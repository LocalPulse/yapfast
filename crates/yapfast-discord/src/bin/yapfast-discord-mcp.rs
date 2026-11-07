//! A Model Context Protocol server for the Discord account linked to
//! Yapfast, so an assistant can read and write in its servers and DMs, run
//! its servers and change its profile. It speaks MCP over stdio (one JSON-RPC
//! message a line) and asks Discord with the token Yapfast keeps in the OS
//! credential store, through the same client as Yapfast: the web client's
//! headers and Discord's rate limits. See docs/discord/mcp.md.
//!
//! Everything it does, Discord sees the account do. Discord's terms forbid
//! automating a user account, so each call should be one the person asked for.

use std::io::{BufRead, Write};

use anyhow::{Context, Result, anyhow, bail};
use base64::Engine;
use serde_json::{Value, json};
use yapfast_discord::{api::Api, token};

/// The MCP version this server speaks, and answers with when asked for
/// another.
const PROTOCOL: &str = "2025-06-18";

/// The most messages one read returns, as Discord's API allows.
const MOST_MESSAGES: u64 = 100;

/// The largest file `set_avatar` and `set_banner` read.
const MOST_PICTURE_BYTES: u64 = 10 * 1024 * 1024;

fn tools() -> Value {
    let string = |description: &str| json!({"type": "string", "description": description});
    json!([
        {
            "name": "whoami",
            "description": "The Discord account Yapfast is linked to: id, username, display name.",
            "inputSchema": {"type": "object", "properties": {}},
        },
        {
            "name": "list_servers",
            "description": "The servers the account is in, with their ids.",
            "inputSchema": {"type": "object", "properties": {}},
        },
        {
            "name": "list_channels",
            "description": "A server's channels in Discord's order, each under its category: id, name, type (0 text, 2 voice, 4 category, 5 announcement, 13 stage, 15 forum), category and topic.",
            "inputSchema": {"type": "object", "properties": {"server_id": string("The server's id.")}, "required": ["server_id"]},
        },
        {
            "name": "list_dms",
            "description": "The account's DMs and group DMs, the one talked in last first: channel id, name, the people in each, and the last message's id.",
            "inputSchema": {"type": "object", "properties": {}},
        },
        {
            "name": "read_messages",
            "description": "A channel's or DM's messages, newest first: id, author, time, content, attachments' names, and what each replies to.",
            "inputSchema": {"type": "object", "properties": {
                "channel_id": string("The channel's or DM's id."),
                "limit": {"type": "integer", "description": "How many, 1 to 100; 50 when left out."},
                "before": string("Only messages older than this message id, to read further back."),
            }, "required": ["channel_id"]},
        },
        {
            "name": "send_message",
            "description": "Sends a message in a channel or DM, as the account. Discord Markdown works.",
            "inputSchema": {"type": "object", "properties": {
                "channel_id": string("The channel's or DM's id."),
                "content": string("The message, up to 2000 characters (4000 with Nitro)."),
                "reply_to": string("A message id to reply to."),
                "silent": {"type": "boolean", "description": "Send without notifying anyone, as @silent does."},
            }, "required": ["channel_id", "content"]},
        },
        {
            "name": "dm_user",
            "description": "Opens (or finds) the DM with a user and sends them a message.",
            "inputSchema": {"type": "object", "properties": {
                "user_id": string("Their user id."),
                "content": string("The message."),
            }, "required": ["user_id", "content"]},
        },
        {
            "name": "get_profile",
            "description": "A user's Discord profile: names, bio, pronouns, badges, connected accounts; with server_id, their roles and nickname there.",
            "inputSchema": {"type": "object", "properties": {
                "user_id": string("Their user id, or @me for the account itself."),
                "server_id": string("A server you share, for their server profile."),
            }, "required": ["user_id"]},
        },
        {
            "name": "edit_profile",
            "description": "Changes the account's own profile: display name, bio (About Me), pronouns, accent colour. Only the fields given change.",
            "inputSchema": {"type": "object", "properties": {
                "global_name": string("Display name; an empty string clears it."),
                "bio": string("About Me, up to 190 characters."),
                "pronouns": string("Pronouns, up to 40 characters."),
                "accent_color": {"type": "integer", "description": "Banner colour as 0xRRGGBB, without a banner picture."},
            }},
        },
        {
            "name": "set_avatar",
            "description": "Sets the account's avatar from a PNG, JPEG, GIF or WebP file on this computer; with clear, removes it.",
            "inputSchema": {"type": "object", "properties": {
                "path": string("The picture's path."),
                "clear": {"type": "boolean", "description": "Remove the avatar instead."},
            }},
        },
        {
            "name": "set_banner",
            "description": "Sets the account's profile banner from a picture file (Nitro); with clear, removes it.",
            "inputSchema": {"type": "object", "properties": {
                "path": string("The picture's path."),
                "clear": {"type": "boolean", "description": "Remove the banner instead."},
            }},
        },
        {
            "name": "set_custom_status",
            "description": "Sets the account's custom status text, with an optional emoji; an empty text clears it.",
            "inputSchema": {"type": "object", "properties": {
                "text": string("The status text."),
                "emoji": string("A Unicode emoji to show with it."),
            }, "required": ["text"]},
        },
        {
            "name": "search_messages",
            "description": "Searches a server's messages, as Discord's search does.",
            "inputSchema": {"type": "object", "properties": {
                "server_id": string("The server's id."),
                "query": string("What to search for."),
                "author_id": string("Only messages by this user."),
                "channel_id": string("Only in this channel."),
            }, "required": ["server_id", "query"]},
        },
        {
            "name": "discord_request",
            "description": "Any request to Discord's user API (https://discord.com/api/v9), for what the other tools don't cover: running a server (channels, roles, members, invites, settings), reactions, pins, threads. See https://docs.discord.food for endpoints and bodies. Returns Discord's JSON answer.",
            "inputSchema": {"type": "object", "properties": {
                "method": {"type": "string", "enum": ["GET", "POST", "PUT", "PATCH", "DELETE"]},
                "path": string("The path after /api/v9, such as /guilds/123/channels."),
                "body": {"type": "object", "description": "The JSON body, for POST, PUT and PATCH."},
                "query": {"type": "object", "description": "Query parameters, added to the path."},
            }, "required": ["method", "path"]},
        },
    ])
}

/// The string `key` of the arguments, required.
fn text<'a>(arguments: &'a Value, key: &str) -> Result<&'a str> {
    arguments[key]
        .as_str()
        .ok_or_else(|| anyhow!("`{key}` is required"))
}

/// An id Discord gave: digits only, so it can go in a path as it is.
fn snowflake<'a>(arguments: &'a Value, key: &str) -> Result<&'a str> {
    let id = text(arguments, key)?;
    if id.is_empty() || !id.bytes().all(|byte| byte.is_ascii_digit()) {
        bail!("`{key}` must be a Discord id, digits only");
    }
    Ok(id)
}

/// A picture file as Discord takes one: a data URI.
fn picture_uri(path: &str) -> Result<String> {
    let path = std::path::Path::new(path);
    let size = std::fs::metadata(path)
        .with_context(|| format!("could not read {}", path.display()))?
        .len();
    if size > MOST_PICTURE_BYTES {
        bail!("the picture is over 10 MB");
    }
    let bytes =
        std::fs::read(path).with_context(|| format!("could not read {}", path.display()))?;
    let kind = match bytes.get(..4) {
        Some([0x89, b'P', b'N', b'G']) => "image/png",
        Some([0xff, 0xd8, 0xff, _]) => "image/jpeg",
        Some([b'G', b'I', b'F', b'8']) => "image/gif",
        Some([b'R', b'I', b'F', b'F']) => "image/webp",
        _ => bail!("the file isn't a PNG, JPEG, GIF or WebP picture"),
    };
    Ok(format!(
        "data:{kind};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

/// A message as the tools give it: what an assistant reads, without the
/// rest of Discord's object.
fn message(value: &Value) -> Value {
    let author = &value["author"];
    json!({
        "id": value["id"],
        "author": {
            "id": author["id"],
            "username": author["username"],
            "display_name": author["global_name"],
            "bot": author["bot"].as_bool().unwrap_or(false),
        },
        "time": value["timestamp"],
        "edited": value["edited_timestamp"],
        "content": value["content"],
        "attachments": value["attachments"].as_array().map(|files| {
            files.iter().map(|file| file["filename"].clone()).collect::<Vec<_>>()
        }),
        "embeds": value["embeds"].as_array().map(Vec::len).unwrap_or(0),
        "reply_to": value["message_reference"]["message_id"],
    })
}

async fn call(api: &Api, name: &str, arguments: &Value) -> Result<Value> {
    Ok(match name {
        "whoami" => {
            let me = api.raw("GET", "/users/@me", None).await?;
            json!({"id": me["id"], "username": me["username"], "display_name": me["global_name"]})
        }
        "list_servers" => {
            let servers = api.raw("GET", "/users/@me/guilds", None).await?;
            Value::Array(
                servers
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|server| json!({"id": server["id"], "name": server["name"], "owner": server["owner"]}))
                    .collect(),
            )
        }
        "list_channels" => {
            let server = snowflake(arguments, "server_id")?;
            let channels = api
                .raw("GET", &format!("/guilds/{server}/channels"), None)
                .await?;
            Value::Array(in_discords_order(
                channels.as_array().map_or(&[][..], Vec::as_slice),
            ))
        }
        "list_dms" => {
            let channels = api.raw("GET", "/users/@me/channels", None).await?;
            let mut channels: Vec<&Value> = channels.as_array().into_iter().flatten().collect();
            // The one talked in last first, as Discord's DM list.
            channels.sort_by_key(|channel| {
                std::cmp::Reverse(
                    channel["last_message_id"]
                        .as_str()
                        .and_then(|id| id.parse::<u64>().ok())
                        .unwrap_or_default(),
                )
            });
            Value::Array(
                channels
                    .into_iter()
                    .map(|channel| {
                        let people: Vec<Value> = channel["recipients"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .map(|user| json!({"id": user["id"], "username": user["username"], "display_name": user["global_name"]}))
                            .collect();
                        json!({"channel_id": channel["id"], "name": channel["name"], "people": people, "last_message_id": channel["last_message_id"]})
                    })
                    .collect(),
            )
        }
        "read_messages" => {
            let channel = snowflake(arguments, "channel_id")?;
            let limit = arguments["limit"]
                .as_u64()
                .unwrap_or(50)
                .clamp(1, MOST_MESSAGES);
            let mut path = format!("/channels/{channel}/messages?limit={limit}");
            if arguments["before"].is_string() {
                path.push_str(&format!("&before={}", snowflake(arguments, "before")?));
            }
            let messages = api.raw("GET", &path, None).await?;
            Value::Array(
                messages
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(message)
                    .collect(),
            )
        }
        "send_message" => {
            let channel = snowflake(arguments, "channel_id")?;
            send(api, channel, arguments).await?
        }
        "dm_user" => {
            let user = snowflake(arguments, "user_id")?;
            let dm = api
                .raw(
                    "POST",
                    "/users/@me/channels",
                    Some(&json!({"recipients": [user]})),
                )
                .await?;
            let channel = dm["id"]
                .as_str()
                .ok_or_else(|| anyhow!("Discord didn't open the DM"))?;
            send(api, channel, arguments).await?
        }
        "get_profile" => {
            let user = match text(arguments, "user_id")? {
                "@me" => api.raw("GET", "/users/@me", None).await?["id"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Discord didn't say who the account is"))?
                    .to_owned(),
                _ => snowflake(arguments, "user_id")?.to_owned(),
            };
            let mut path = format!("/users/{user}/profile?with_mutual_guilds=true");
            if arguments["server_id"].is_string() {
                path.push_str(&format!("&guild_id={}", snowflake(arguments, "server_id")?));
            }
            api.raw("GET", &path, None).await?
        }
        "edit_profile" => {
            let mut user = serde_json::Map::new();
            if let Some(name) = arguments["global_name"].as_str() {
                user.insert(
                    "global_name".into(),
                    if name.is_empty() {
                        Value::Null
                    } else {
                        name.into()
                    },
                );
            }
            let mut profile = serde_json::Map::new();
            for key in ["bio", "pronouns", "accent_color"] {
                if !arguments[key].is_null() {
                    profile.insert(key.into(), arguments[key].clone());
                }
            }
            if user.is_empty() && profile.is_empty() {
                bail!("give at least one of global_name, bio, pronouns or accent_color");
            }
            if !user.is_empty() {
                api.raw("PATCH", "/users/@me", Some(&Value::Object(user)))
                    .await?;
            }
            if !profile.is_empty() {
                api.raw("PATCH", "/users/@me/profile", Some(&Value::Object(profile)))
                    .await?;
            }
            json!({"changed": true})
        }
        "set_avatar" | "set_banner" => {
            let field = if name == "set_avatar" {
                "avatar"
            } else {
                "banner"
            };
            let picture = if arguments["clear"].as_bool().unwrap_or(false) {
                Value::Null
            } else {
                picture_uri(text(arguments, "path")?)?.into()
            };
            let changed = api
                .raw("PATCH", "/users/@me", Some(&json!({ field: picture })))
                .await?;
            json!({ field: changed[field] })
        }
        "set_custom_status" => {
            let status = yapfast_discord::settings::CustomStatus {
                text: text(arguments, "text")?.to_owned(),
                emoji_name: arguments["emoji"].as_str().map(str::to_owned),
                emoji_id: None,
                expires_at_ms: None,
            };
            api.set_custom_status((!status.is_empty()).then_some(&status))
                .await?;
            json!({"changed": true})
        }
        "search_messages" => {
            let server = snowflake(arguments, "server_id")?;
            let query = text(arguments, "query")?;
            let mut path = format!(
                "/guilds/{server}/messages/search?content={}",
                urlencoding(query)
            );
            for (key, parameter) in [("author_id", "author_id"), ("channel_id", "channel_id")] {
                if arguments[key].is_string() {
                    path.push_str(&format!("&{parameter}={}", snowflake(arguments, key)?));
                }
            }
            let found = api.raw("GET", &path, None).await?;
            json!({
                "total": found["total_results"],
                "messages": found["messages"].as_array().into_iter().flatten()
                    .filter_map(|group| group.get(0)).map(message).collect::<Vec<_>>(),
            })
        }
        "discord_request" => {
            let method = text(arguments, "method")?;
            if !["GET", "POST", "PUT", "PATCH", "DELETE"].contains(&method) {
                bail!("`method` must be GET, POST, PUT, PATCH or DELETE");
            }
            let mut path = text(arguments, "path")?.to_owned();
            if !path.starts_with('/') || path.contains("://") || path.contains("..") {
                bail!("`path` must be a path under /api/v9, starting with /");
            }
            if let Some(query) = arguments["query"].as_object() {
                for (index, (key, value)) in query.iter().enumerate() {
                    let value = value
                        .as_str()
                        .map_or_else(|| value.to_string(), str::to_owned);
                    let join = if index == 0 && !path.contains('?') {
                        '?'
                    } else {
                        '&'
                    };
                    path.push_str(&format!(
                        "{join}{}={}",
                        urlencoding(key),
                        urlencoding(&value)
                    ));
                }
            }
            let body = arguments.get("body").filter(|body| !body.is_null());
            api.raw(method, &path, body).await?
        }
        other => bail!("no tool called {other}"),
    })
}

/// A server's channels as Discord lists them: those in no category first,
/// then each category by its position with its channels; within each, text
/// channels (and forums and announcements) before voice and stage
/// channels, each by position. Each channel names its category.
fn in_discords_order(channels: &[Value]) -> Vec<Value> {
    const CATEGORY: u64 = 4;
    let voice = |channel: &Value| matches!(channel["type"].as_u64(), Some(2 | 13));
    let position = |channel: &Value| channel["position"].as_i64().unwrap_or_default();
    let id = |channel: &Value| {
        channel["id"]
            .as_str()
            .and_then(|id| id.parse::<u64>().ok())
            .unwrap_or_default()
    };
    let mut categories: Vec<&Value> = channels
        .iter()
        .filter(|channel| channel["type"].as_u64() == Some(CATEGORY))
        .collect();
    categories.sort_by_key(|category| (position(category), id(category)));
    let row = |channel: &Value, category: Option<&Value>| {
        json!({
            "id": channel["id"], "name": channel["name"], "type": channel["type"],
            "category": category.map(|category| category["name"].clone()),
            "topic": channel["topic"],
        })
    };
    let under = |parent: Option<&str>| {
        let mut children: Vec<&Value> = channels
            .iter()
            .filter(|channel| {
                channel["type"].as_u64() != Some(CATEGORY)
                    && channel["parent_id"].as_str() == parent
            })
            .collect();
        children.sort_by_key(|channel| (voice(channel), position(channel), id(channel)));
        children
    };
    let mut ordered: Vec<Value> = under(None)
        .into_iter()
        .map(|channel| row(channel, None))
        .collect();
    for category in categories {
        ordered.push(row(category, None));
        ordered.extend(
            under(category["id"].as_str())
                .into_iter()
                .map(|channel| row(channel, Some(category))),
        );
    }
    ordered
}

async fn send(api: &Api, channel: &str, arguments: &Value) -> Result<Value> {
    let content = text(arguments, "content")?;
    if content.trim().is_empty() {
        bail!("`content` is empty");
    }
    let mut body = json!({
        "content": content,
        "nonce": yapfast_discord::message::nonce(),
        "tts": false,
    });
    if arguments["silent"].as_bool().unwrap_or(false) {
        body["flags"] = json!(1 << 12);
    }
    if arguments["reply_to"].is_string() {
        body["message_reference"] = json!({"message_id": snowflake(arguments, "reply_to")?});
    }
    let sent = api
        .raw(
            "POST",
            &format!("/channels/{channel}/messages"),
            Some(&body),
        )
        .await?;
    Ok(message(&sent))
}

/// Percent-encodes a query value.
fn urlencoding(text: &str) -> String {
    let mut encoded = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.~".contains(&byte) {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn answer(id: &Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn failure(id: &Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

async fn handle(api: &Option<Api>, request: &Value) -> Option<Value> {
    let id = request.get("id")?.clone();
    let method = request["method"].as_str().unwrap_or_default();
    Some(match method {
        "initialize" => answer(
            &id,
            json!({
                "protocolVersion": PROTOCOL,
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "yapfast-discord", "version": env!("CARGO_PKG_VERSION")},
                "instructions": "Acts as the Discord account linked to Yapfast. Every call is the account acting: do only what the person asked for, and confirm before sending messages, changing a server or a profile, or deleting anything.",
            }),
        ),
        "ping" => answer(&id, json!({})),
        "tools/list" => answer(&id, json!({"tools": tools()})),
        "tools/call" => {
            let params = &request["params"];
            let name = params["name"].as_str().unwrap_or_default();
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
            let outcome = match api {
                Some(api) => call(api, name, &arguments).await,
                None => Err(anyhow!(
                    "no Discord account is linked: sign in to Discord in Yapfast first"
                )),
            };
            let (text, error) = match outcome {
                Ok(value) => (
                    serde_json::to_string_pretty(&value).unwrap_or_default(),
                    false,
                ),
                Err(error) => (format!("{error:#}"), true),
            };
            answer(
                &id,
                json!({"content": [{"type": "text", "text": text}], "isError": error}),
            )
        }
        _ => failure(&id, -32601, "method not found"),
    })
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Logs go to stderr: stdout carries the protocol alone.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("YAPFAST_LOG")
                .unwrap_or_else(|_| "warn".into()),
        )
        .init();
    let api = match token::load() {
        Ok(token) => Some(Api::new(token)?),
        Err(error) => {
            tracing::warn!(%error, "no Discord token");
            None
        }
    };
    if let Some(api) = &api {
        api.learn_build_number().await;
    }
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line.context("could not read stdin")?;
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(request) => handle(&api, &request).await,
            Err(_) => Some(failure(&Value::Null, -32700, "parse error")),
        };
        if let Some(reply) = reply {
            writeln!(stdout, "{reply}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_and_paths_are_checked_before_they_go_to_discord() {
        assert!(snowflake(&json!({"id": "123"}), "id").is_ok());
        assert!(snowflake(&json!({"id": "12/../3"}), "id").is_err());
        assert!(snowflake(&json!({}), "id").is_err());
        assert_eq!(urlencoding("a b&c"), "a%20b%26c");
    }

    #[test]
    fn channels_come_in_discords_order() {
        let channels = json!([
            {"id": "5", "name": "voice", "type": 2, "parent_id": "1", "position": 0},
            {"id": "4", "name": "chat", "type": 0, "parent_id": "1", "position": 1},
            {"id": "1", "name": "Talk", "type": 4, "position": 1},
            {"id": "2", "name": "Info", "type": 4, "position": 0},
            {"id": "3", "name": "rules", "type": 0, "parent_id": "2", "position": 0},
            {"id": "6", "name": "lobby", "type": 0, "position": 5},
        ]);
        let names: Vec<_> = in_discords_order(channels.as_array().unwrap())
            .iter()
            .map(|channel| channel["name"].as_str().unwrap().to_owned())
            .collect();
        assert_eq!(names, ["lobby", "Info", "rules", "Talk", "chat", "voice"]);
    }

    #[tokio::test]
    async fn speaks_mcp_without_a_linked_account() {
        let init = handle(
            &None,
            &json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}),
        )
        .await
        .unwrap();
        assert_eq!(init["result"]["protocolVersion"], PROTOCOL);
        let listed = handle(
            &None,
            &json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
        )
        .await
        .unwrap();
        assert!(listed["result"]["tools"].as_array().unwrap().len() >= 10);
        let called = handle(
            &None,
            &json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "whoami", "arguments": {}}}),
        )
        .await
        .unwrap();
        assert_eq!(called["result"]["isError"], true);
        // Notifications get no answer.
        assert!(
            handle(
                &None,
                &json!({"jsonrpc": "2.0", "method": "notifications/initialized"})
            )
            .await
            .is_none()
        );
    }
}
