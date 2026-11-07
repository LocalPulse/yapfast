//! A person's Discord profile: who they are across Discord, and in one
//! server. See <https://docs.discord.food/resources/user#get-user-profile>.

use serde_json::Value;

use crate::{
    message::{iso_ms, snowflake_ms},
    ready,
};

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Profile {
    pub user_id: String,
    /// Their nickname in the server, else their display name or username.
    pub name: String,
    pub username: String,
    /// The address of their avatar: the server's, else their own.
    pub avatar: String,
    /// The address of the decoration drawn over their avatar.
    pub avatar_decoration: Option<String>,
    /// Their Discord and server badges, in Discord's order.
    pub badges: Vec<Badge>,
    /// Their public connected accounts.
    pub connections: Vec<ConnectedAccount>,
    /// Their profile's background and accent colours.
    pub theme: Option<(u32, u32)>,
    /// Their server tag and its badge.
    pub server_tag: Option<ServerTag>,
    /// The static nameplate shown behind their name.
    pub nameplate: Option<String>,
    /// Layers of their profile effect, in drawing order.
    pub effects: Vec<EffectLayer>,
    /// The address of their banner: the server's, else their own.
    pub banner: Option<String>,
    /// The banner's colour when they have no banner picture.
    pub accent: Option<u32>,
    pub pronouns: Option<String>,
    /// Their "About me", in Discord Markdown.
    pub bio: Option<String>,
    /// Their roles' ids in the server.
    pub roles: Vec<String>,
    /// When they joined the server, in Unix milliseconds.
    pub joined: Option<u64>,
    /// When their account was made, in Unix milliseconds.
    pub created: u64,
    /// The servers this account shares with them.
    pub mutual_servers: Vec<MutualServer>,
    /// The friends this account shares with them.
    pub mutual_friends: Vec<MutualFriend>,
}

/// A Discord or server badge on someone's profile.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Badge {
    pub id: String,
    pub description: String,
    /// The address of its icon.
    pub icon: String,
    /// What clicking it opens, when Discord gives a link.
    pub link: Option<String>,
}

/// A public account connected to someone's Discord profile.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConnectedAccount {
    /// Discord's connection type, such as `github` or `spotify`.
    pub kind: String,
    pub name: String,
    pub verified: bool,
}

/// A server tag shown beside someone's username.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ServerTag {
    pub tag: String,
    pub badge: String,
}

/// One image layer of a profile effect.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EffectLayer {
    pub image: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub z_index: i32,
}

/// A server this account shares with someone.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MutualServer {
    pub id: String,
    /// Their nickname there, when they set one.
    pub nick: Option<String>,
}

/// A friend this account shares with someone.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MutualFriend {
    pub id: String,
    pub name: String,
    /// The address of their avatar.
    pub avatar: String,
}

/// This account's own profile as Discord's profile editors change it, in
/// a server or its main one: what it set, and what shows where it set
/// nothing. An empty text or no picture goes by `fallback`, as Discord's
/// client shows them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OwnProfile {
    /// The server nickname, or the display name.
    pub name: String,
    pub pronouns: String,
    pub bio: String,
    /// The addresses of the avatar and banner, when set.
    pub avatar: Option<String>,
    pub banner: Option<String>,
    /// The main profile's banner colour.
    pub accent: Option<u32>,
    /// In a server, its profile outside any server; outside, the username
    /// and Discord's default avatar.
    pub fallback: Profile,
}

/// The largest picture Discord's client uploads as an avatar or banner.
pub const PICTURE_LIMIT: u64 = 10 * 1024 * 1024;

/// This account's profile in `guild`, from the profile endpoint's answer
/// for its own id with `guild_id`.
pub fn parse_server(value: &Value, guild: &str) -> Option<OwnProfile> {
    let main = parse(value, None)?;
    let (server, member) = (&value["guild_member_profile"], &value["guild_member"]);
    let owned = |value: &Value| text(value).map(str::to_owned).unwrap_or_default();
    Some(OwnProfile {
        name: owned(&member["nick"]),
        pronouns: owned(&server["pronouns"]),
        bio: text(&server["bio"])
            .or_else(|| text(&member["bio"]))
            .map(str::to_owned)
            .unwrap_or_default(),
        avatar: text(&member["avatar"])
            .map(|hash| ready::member_avatar(guild, &main.user_id, hash)),
        banner: text(&server["banner"])
            .or_else(|| text(&member["banner"]))
            .map(|hash| ready::member_banner(guild, &main.user_id, hash)),
        accent: None,
        fallback: main,
    })
}

/// This account's main profile, from the profile endpoint's answer for its
/// own id without a server.
pub fn parse_main(value: &Value) -> Option<OwnProfile> {
    let profile = parse(value, None)?;
    let (user, own) = (&value["user"], &value["user_profile"]);
    let id = profile.user_id.clone();
    let fallback = Profile {
        name: profile.username.clone(),
        avatar: ready::avatar(&id, None),
        banner: None,
        accent: None,
        pronouns: None,
        bio: None,
        ..profile.clone()
    };
    Some(OwnProfile {
        name: user["global_name"].as_str().unwrap_or_default().to_owned(),
        pronouns: profile.pronouns.unwrap_or_default(),
        bio: profile.bio.unwrap_or_default(),
        avatar: text(&user["avatar"]).map(|hash| ready::avatar(&id, Some(hash))),
        banner: profile.banner,
        accent: color(&own["accent_color"]).or_else(|| color(&user["accent_color"])),
        fallback,
    })
}

/// A colour field that may be missing or null.
fn color(value: &Value) -> Option<u32> {
    value.as_u64().and_then(|color| u32::try_from(color).ok())
}

/// A text field that may be missing, null or empty.
fn text(value: &Value) -> Option<&str> {
    value.as_str().filter(|text| !text.trim().is_empty())
}

fn theme(value: &Value) -> Option<(u32, u32)> {
    let colors = value.as_array()?;
    Some((color(colors.first()?)?, color(colors.get(1)?)?))
}

/// The selected profile effect's collectible SKU, with the server's one winning.
pub fn effect_id<'a>(value: &'a Value, guild: Option<&str>) -> Option<&'a str> {
    let server = guild.map(|_| &value["guild_member_profile"]["profile_effect"]["id"]);
    server
        .and_then(|id| text(id))
        .or_else(|| text(&value["user_profile"]["profile_effect"]["id"]))
}

/// The layers of a collectible product's profile effect.
pub fn parse_effect(product: &Value) -> Vec<EffectLayer> {
    let mut layers: Vec<EffectLayer> = product["items"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|item| item["type"].as_i64() == Some(1))
        .flat_map(|item| item["effects"].as_array().into_iter().flatten())
        .filter_map(|effect| {
            Some(EffectLayer {
                image: text(&effect["src"])?.to_owned(),
                x: effect["position"]["x"].as_i64().unwrap_or(0) as i32,
                y: effect["position"]["y"].as_i64().unwrap_or(0) as i32,
                width: effect["width"].as_i64().unwrap_or(0) as i32,
                height: effect["height"].as_i64().unwrap_or(0) as i32,
                z_index: effect["zIndex"].as_i64().unwrap_or(0) as i32,
            })
        })
        .collect();
    layers.sort_by_key(|layer| layer.z_index);
    layers
}

/// A profile from the profile endpoint's answer. The server's profile, asked
/// for with `guild`, wins over the person's own where they set one.
pub fn parse(value: &Value, guild: Option<&str>) -> Option<Profile> {
    let user = &value["user"];
    let id = user["id"].as_str()?.to_owned();
    let own = &value["user_profile"];
    let (server, member) = match guild {
        Some(_) => (&value["guild_member_profile"], &value["guild_member"]),
        None => (&Value::Null, &Value::Null),
    };
    let name = text(&member["nick"])
        .or_else(|| ready::display_name(user))
        .unwrap_or("Discord user")
        .to_owned();
    let avatar = match (guild, text(&member["avatar"])) {
        (Some(guild), Some(hash)) => ready::member_avatar(guild, &id, hash),
        _ => ready::avatar(&id, user["avatar"].as_str()),
    };
    let banner = match (guild, text(&server["banner"])) {
        (Some(guild), Some(hash)) => Some(ready::member_banner(guild, &id, hash)),
        _ => text(&own["banner"])
            .or_else(|| text(&user["banner"]))
            .map(|hash| ready::banner(&id, hash)),
    };
    Some(Profile {
        name,
        username: user["username"].as_str().unwrap_or_default().to_owned(),
        avatar,
        avatar_decoration: text(&user["avatar_decoration_data"]["asset"]).map(|asset| {
            format!(
                "{}/avatar-decoration-presets/{asset}.png?size=160&passthrough=true",
                ready::cdn()
            )
        }),
        badges: value["badges"]
            .as_array()
            .into_iter()
            .flatten()
            .chain(value["guild_badges"].as_array().into_iter().flatten())
            .filter_map(|badge| {
                let icon = text(&badge["icon"])?;
                Some(Badge {
                    id: badge["id"].as_str().unwrap_or_default().to_owned(),
                    description: badge["description"].as_str().unwrap_or_default().to_owned(),
                    icon: format!("{}/badge-icons/{icon}.png?size=64", ready::cdn()),
                    link: text(&badge["link"]).map(str::to_owned),
                })
            })
            .collect(),
        connections: value["connected_accounts"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|account| {
                Some(ConnectedAccount {
                    kind: text(&account["type"])?.to_owned(),
                    name: text(&account["name"])?.to_owned(),
                    verified: account["verified"].as_bool().unwrap_or(false),
                })
            })
            .collect(),
        theme: theme(&server["theme_colors"]).or_else(|| theme(&own["theme_colors"])),
        server_tag: {
            let tag = &user["primary_guild"];
            if tag["identity_enabled"].as_bool() == Some(true) {
                let text_tag = text(&tag["tag"]);
                let badge = text(&tag["badge"]);
                let guild = text(&tag["identity_guild_id"]);
                match (text_tag, badge, guild) {
                    (Some(text_tag), Some(badge), Some(guild)) => Some(ServerTag {
                        tag: text_tag.to_owned(),
                        badge: format!(
                            "{}/guild-tag-badges/{guild}/{badge}.png?size=64",
                            ready::cdn()
                        ),
                    }),
                    _ => None,
                }
            } else {
                None
            }
        },
        nameplate: text(&user["collectibles"]["nameplate"]["asset"])
            .map(|asset| format!("{}/assets/collectibles/{asset}static.png", ready::cdn())),
        effects: Vec::new(),
        banner,
        accent: color(&server["accent_color"])
            .or_else(|| color(&own["accent_color"]))
            .or_else(|| color(&user["accent_color"])),
        pronouns: text(&server["pronouns"])
            .or_else(|| text(&own["pronouns"]))
            .map(str::to_owned),
        bio: text(&server["bio"])
            .or_else(|| text(&own["bio"]))
            .or_else(|| text(&user["bio"]))
            .map(str::to_owned),
        roles: member["roles"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|role| role.as_str().map(str::to_owned))
            .collect(),
        joined: member["joined_at"].as_str().and_then(iso_ms),
        created: snowflake_ms(&id),
        mutual_servers: value["mutual_guilds"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|guild| {
                Some(MutualServer {
                    id: guild["id"].as_str()?.to_owned(),
                    nick: text(&guild["nick"]).map(str::to_owned),
                })
            })
            .collect(),
        mutual_friends: value["mutual_friends"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|friend| {
                let id = friend["id"].as_str()?;
                Some(MutualFriend {
                    id: id.to_owned(),
                    name: ready::display_name(friend)?.to_owned(),
                    avatar: ready::avatar(id, friend["avatar"].as_str()),
                })
            })
            .collect(),
        user_id: id,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn the_servers_profile_wins_over_their_own() {
        let answer = json!({
            "user": {"id": "1400000000000000200", "username": "ann", "global_name": "Ann",
                     "avatar": "abc", "banner": null, "accent_color": 16711680,
                     "avatar_decoration_data": {"asset": "a_ring", "sku_id": "1", "expires_at": null},
                     "primary_guild": {"identity_guild_id": "10", "identity_enabled": true,
                                       "tag": "TEST", "badge": "guild-icon"},
                     "collectibles": {"nameplate": {"asset": "plate/"}}},
            "user_profile": {"bio": "Hi, I'm **Ann**", "pronouns": "she/her", "banner": "def",
                             "accent_color": null, "theme_colors": [1118481, 16711680],
                             "profile_effect": {"id": "42"}},
            "connected_accounts": [{"type": "github", "id": "123", "name": "ann-dev", "verified": true}],
            "badges": [{"id": "nitro", "description": "Nitro subscriber", "icon": "nitro-icon",
                        "link": "https://discord.com/nitro"}],
            "guild_badges": [{"id": "server", "description": "Server badge", "icon": "server-icon"}],
            "guild_member_profile": {"guild_id": "10", "bio": "", "pronouns": "", "banner": "ghi",
                                     "accent_color": 255},
            "guild_member": {"nick": "Annie", "avatar": null, "roles": ["30", "31"],
                             "joined_at": "2026-09-01T09:00:00.000000+00:00"},
            "mutual_guilds": [{"id": "10", "nick": "Annie"}, {"id": "11", "nick": null}],
            "mutual_friends": [{"id": "3", "username": "bob", "global_name": "Bob"}],
        });
        let profile = parse(&answer, Some("10")).unwrap();
        assert_eq!(profile.name, "Annie");
        assert_eq!(profile.username, "ann");
        assert!(
            profile
                .avatar
                .ends_with("/avatars/1400000000000000200/abc.png?size=128")
        );
        assert!(
            profile
                .banner
                .as_deref()
                .unwrap()
                .ends_with("/guilds/10/users/1400000000000000200/banners/ghi.png?size=600")
        );
        assert_eq!(profile.accent, Some(255));
        assert_eq!(
            profile.avatar_decoration.as_deref(),
            Some(
                "https://cdn.discordapp.com/avatar-decoration-presets/a_ring.png?size=160&passthrough=true"
            )
        );
        assert_eq!(profile.badges.len(), 2);
        assert_eq!(profile.badges[0].id, "nitro");
        assert_eq!(profile.badges[0].description, "Nitro subscriber");
        assert_eq!(
            profile.badges[0].link.as_deref(),
            Some("https://discord.com/nitro")
        );
        assert_eq!(
            profile.badges[1].icon,
            "https://cdn.discordapp.com/badge-icons/server-icon.png?size=64"
        );
        assert_eq!(profile.connections.len(), 1);
        assert_eq!(profile.connections[0].kind, "github");
        assert_eq!(profile.connections[0].name, "ann-dev");
        assert!(profile.connections[0].verified);
        assert_eq!(profile.theme, Some((1118481, 16711680)));
        assert_eq!(
            profile.server_tag.as_ref().map(|tag| tag.tag.as_str()),
            Some("TEST")
        );
        assert_eq!(
            profile.server_tag.as_ref().map(|tag| tag.badge.as_str()),
            Some("https://cdn.discordapp.com/guild-tag-badges/10/guild-icon.png?size=64")
        );
        assert_eq!(
            profile.nameplate.as_deref(),
            Some("https://cdn.discordapp.com/assets/collectibles/plate/static.png")
        );
        assert_eq!(effect_id(&answer, Some("10")), Some("42"));
        assert_eq!(
            profile.pronouns.as_deref(),
            Some("she/her"),
            "an empty server one falls back"
        );
        assert_eq!(profile.bio.as_deref(), Some("Hi, I'm **Ann**"));
        assert_eq!(profile.roles, ["30", "31"]);
        assert_eq!(profile.joined, Some(1_788_253_200_000));
        assert_eq!(profile.created, 1_753_856_410_742);
        assert_eq!(
            profile.mutual_servers,
            [
                MutualServer {
                    id: "10".into(),
                    nick: Some("Annie".into())
                },
                MutualServer {
                    id: "11".into(),
                    nick: None
                }
            ]
        );
        assert_eq!(profile.mutual_friends[0].id, "3");
        assert_eq!(profile.mutual_friends[0].name, "Bob");
        assert!(profile.mutual_friends[0].avatar.contains("/embed/avatars/"));

        let own = parse_server(&answer, "10").unwrap();
        assert_eq!(own.name, "Annie");
        assert_eq!(own.pronouns, "", "the main profile's shows instead");
        assert_eq!(own.bio, "");
        assert_eq!(own.avatar, None);
        assert!(
            own.banner
                .as_deref()
                .unwrap()
                .ends_with("/guilds/10/users/1400000000000000200/banners/ghi.png?size=600")
        );
        assert_eq!(own.fallback.name, "Ann");
        assert_eq!(own.fallback.pronouns.as_deref(), Some("she/her"));

        // Her main profile: what she set, over her username and Discord's
        // default avatar.
        let main = parse_main(&answer).unwrap();
        assert_eq!(main.name, "Ann");
        assert_eq!(main.pronouns, "she/her");
        assert_eq!(main.bio, "Hi, I'm **Ann**");
        assert!(
            main.avatar
                .as_deref()
                .unwrap()
                .contains("/avatars/1400000000000000200/abc.png")
        );
        assert!(
            main.banner
                .as_deref()
                .unwrap()
                .ends_with("/banners/1400000000000000200/def.png?size=600")
        );
        assert_eq!(
            main.accent,
            Some(16_711_680),
            "the user's, as the profile's is null"
        );
        assert_eq!(main.fallback.name, "ann");
        assert!(main.fallback.avatar.contains("/embed/avatars/"));
        assert_eq!((main.fallback.bio, main.fallback.banner), (None, None));

        // Outside a server: their own banner and name.
        let profile = parse(&answer, None).unwrap();
        assert!(
            profile
                .banner
                .as_deref()
                .unwrap()
                .ends_with("/banners/1400000000000000200/def.png?size=600")
        );
        assert_eq!(profile.accent, Some(16_711_680));
    }

    #[test]
    fn an_incomplete_profile_theme_is_ignored() {
        assert_eq!(theme(&json!([16_711_680])), None);
        assert_eq!(theme(&json!([])), None);
    }

    #[test]
    fn profile_effect_layers_come_in_drawing_order() {
        let product = json!({"items": [{"type": 1, "effects": [
            {"src": "top.png", "position": {"x": 4, "y": 5}, "width": 200, "height": 300, "zIndex": 2},
            {"src": "bottom.png", "position": {"x": 0, "y": 1}, "width": 100, "height": 150, "zIndex": 1}
        ]}]});
        let layers = parse_effect(&product);
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[0].image, "bottom.png");
        assert_eq!(layers[1].image, "top.png");
        assert_eq!((layers[1].x, layers[1].y), (4, 5));
        assert_eq!((layers[1].width, layers[1].height), (200, 300));
    }
}
