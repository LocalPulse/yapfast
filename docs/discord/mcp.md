# MCP server for Discord

`yapfast-discord-mcp` (`crates/yapfast-discord/src/bin/yapfast-discord-mcp.rs`)
is a [Model Context Protocol](https://modelcontextprotocol.io) server for
the Discord account linked to Yapfast. An assistant such as Claude Code
reads and writes in its servers and DMs, runs its servers and changes its
profile through it.

It speaks MCP over stdio, one JSON-RPC message a line, and asks Discord
with the token Yapfast keeps in the OS credential store (`token::load`), so
it works once Discord is linked in Yapfast, whether Yapfast is running or
not. Its requests go through `Api` as Yapfast's do: the web client's
headers and super properties, and Discord's rate limits (`limits`). It
keeps no gateway connection: everything is the REST API.

Everything it does, Discord sees the account do. Discord's terms forbid
automating a user account; each call should be one the person asked for,
and its instructions to the assistant say to confirm before sending,
changing a server or a profile, or deleting anything.

## Tools

- `whoami`, `list_servers`, `list_channels` (a server's), `list_dms`.
- `read_messages`: a channel's or DM's messages, newest first, up to 100,
  `before` a message to read further back.
- `send_message` (with `reply_to` and `silent`), `dm_user`.
- `search_messages`: a server's search, by text, author or channel.
- `get_profile`: a user's profile, with their server profile given a
  server.
- `edit_profile` (display name, bio, pronouns, accent colour),
  `set_avatar` and `set_banner` from a picture file, or cleared,
  `set_custom_status`.
- `discord_request`: any request to `https://discord.com/api/v9`, for the
  rest: channels, roles, members, invites, server settings, reactions,
  pins, threads. Its path must be a path, starting with `/`.

Ids are checked to be digits before they go into a path.

## Setting it up

The Windows build puts `yapfast-discord-mcp.exe` beside `yapfast.exe`. For
Claude Code:

```bash
claude mcp add discord -- "C:\Users\<you>\AppData\Local\Programs\Yapfast-fork-…\yapfast-discord-mcp.exe"
```

or in a project's `.mcp.json`:

```json
{"mcpServers": {"discord": {"command": "C:\\…\\yapfast-discord-mcp.exe"}}}
```

`YAPFAST_LOG` sets what it logs, to stderr; stdout carries the protocol.
