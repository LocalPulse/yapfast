# LocalPulse fork: plan

This fork adds to Yapfast the Vencord plugins its owner has enabled, with
their Vencord settings imported, plus fixes and features found while using it.
Only plugins enabled in `%APPDATA%\Vencord\settings\settings.json` (87 on
2026-11-07) are in scope.

Status: `[ ]` to do, `[~]` partly there in upstream Yapfast (check before
writing), `[x]` done, `n/a` nothing to port.

## Fixes and features outside Vencord

- [x] Bot buttons send `POST /interactions` (type 3).
- [ ] Bot select menus open and send their values; `INTERACTION_FAILURE`
      shows "This interaction failed" on the message.
- [ ] Bot modals (`INTERACTION_MODAL_CREATE`): text inputs, labels, selects,
      submitted as type 5.
- [x] Shortcuts (Ctrl+A/C/V/X/Z/Y and Yapfast's own) work on the Russian
      and Ukrainian layouts: with Ctrl held, a Cyrillic letter becomes the
      Latin one on the same key (`vendor/i-slint-core/window.rs`).
- [x] `<t:…:STYLE>` timestamps honour their style (t, T, d, D, f, F, R) and
      the 12-hour clock setting (`crates/yapfast-discord/src/message.rs`).
- [ ] Profile: avatar decorations, profile effects, badges, connected
      accounts, theme colours, server tag.
- [ ] Friends page: "Active Now" column (games, voice channels, grouped).
- [x] Authors' role colours follow once a member list or profile teaches
      their roles (they stayed white until something else re-rendered).
- [ ] Smooth scrolling and animations at the display's refresh rate (240 Hz):
      the channel list scrolls at 40–50 fps today. Check the renderer in use
      (Skia GPU or the software fallback), frame pacing and per-frame work.
- [ ] Import Vencord settings (plugin options, PinDMs categories).
- [ ] MCP server for Discord, through the running Yapfast session.
- [x] Skia Windows builder lockfile: prettyplease 0.3 so bindgen 0.73 sees
      one `syn` (upstream's Windows build fails without it).

## Vencord plugins

### Nothing to port (Vencord's own plumbing or the web client's)

n/a: ChatInputButtonAPI, CommandsAPI, ContextMenuAPI, DynamicImageModalAPI,
MemberListDecoratorsAPI, MessageAccessoriesAPI, MessageDecorationsAPI,
MessageEventsAPI, MessagePopoverAPI, MessageUpdaterAPI, NoticesAPI,
ServerListAPI, UserSettingsAPI, BadgeAPI, Settings, SupportHelper,
ConcatenatedComponentExtractor, ConsoleJanitor, CrashHandler,
ReactErrorDecoder, FixDiscordCss, FixCodeblockGap, VencordToolbox,
BetterSettings, NoTrack (Yapfast sends no analytics), YoutubeAdblock and
SpotifyCrack (Yapfast plays neither inside the app).

### To port

Messages and chat
- [ ] MessageLogger: keep deleted and edited messages locally, show them.
- [ ] SilentTyping: don't send typing, toggle in the composer.
- [ ] ClearURLs: strip tracking parameters from sent links.
- [ ] QuickReply: Ctrl+Up/Down to reply to messages.
- [ ] QuickMention: mention button in the message hover bar.
- [ ] MessageClickActions: double-click to edit / reply, backspace-click
      to delete.
- [ ] ReplyTimestamp: time of the replied-to message.
- [ ] ValidReply: fetch replied-to messages Yapfast doesn't have.
- [ ] ValidUser: fetch mentioned users Yapfast doesn't know.
- [ ] PreviewMessage: preview a message before sending.
- [ ] Translate: translate messages and drafts.
- [ ] ShikiCodeblocks: syntax highlighting in code blocks.
- [ ] MessageLatency: show when a message arrived late.
- [ ] Summaries: Discord's conversation summaries.
- [ ] TypingIndicator: typing dots on channels and DMs in the sidebar.
- [ ] TypingTweaks: avatars and colours in "is typing".
- [ ] NoTypingAnimation: still typing indicator.
- [ ] UserMessagesPronouns: pronouns beside names in messages.
- [ ] ForceOwnerCrown: crown for the server owner even when offline.
- [ ] petpet: /petpet command.
- [ ] SpotifyShareCommands: /track, /album, /artist.
- [ ] CopyFileContents: copy a text attachment's contents.

Media, files and emoji
- [~] ImageZoom: magnifier on images.
- [~] PictureInPicture: pop out videos.
- [ ] ViewIcons: open avatars, banners and icons full size.
- [ ] UnlockedAvatarZoom: zoom avatars further.
- [ ] FixImagesQuality: load images at full quality.
- [ ] ImageFilename: show image file names.
- [ ] AnonymiseFileNames: random names for uploads.
- [ ] BetterUploadButton: click uploads, right-click opens the menu.
- [ ] AddAttachments / GifPaste / StickerPaste: insert into the draft
      instead of sending.
- [ ] BetterGifPicker: open GIF picker on favourites.
- [ ] FavoriteGifSearch: search favourite GIFs.
- [ ] FavoriteEmojiFirst: favourite emoji first in suggestions.
- [~] VoiceMessages: record and send voice messages.
- [ ] VoiceDownload: download voice messages.

People, DMs and servers
- [ ] PinDMs: pinned DM categories (import Pin, Work, Increase, Studio,
      Anime Paradise with their colours and order).
- [~] PlatformIndicators: desktop / web / mobile icons.
- [~] PermissionsViewer: members' and roles' permissions.
- [ ] BetterRoleContext: role menu (copy colour, edit, view as).
- [ ] BetterNotesBox: notes box tweaks.
- [ ] BetterSessions: name devices, notify of new sessions.
- [ ] RelationshipNotifier: notify when removed as a friend or from a
      server / group.
- [ ] SortFriendRequests: sort by date.
- [ ] NoPendingCount: no count for friend requests / message requests.
- [~] MemberCount: online and total members.
- [ ] ServerInfo: server info dialog.
- [ ] NewGuildSettings: defaults for newly joined servers.
- [ ] ReadAllNotificationsButton: mark every server read.
- [ ] OnePingPerDM: one notification per unread DM.
- [ ] NotificationVolume: notification sound volume.
- [ ] AccountPanelServerProfile: server profile from the account panel.
- [ ] UserVoiceShow: voice channel a user is in, on their profile.
- [~] GameActivityToggle: toggle game activity from the account panel.

Voice, calls and streaming
- [~] CallTimer: call duration.
- [ ] VolumeBooster: user volume above 200%.
- [ ] StreamerModeOnStream: streamer mode while streaming.
- [ ] SpotifyControls: Spotify player above the account panel.

Other
- [ ] LoadingQuotes: quotes on the loading screen.
