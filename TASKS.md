# Задачи форка LocalPulse/yapfast

Статусы: ✅ готово · 🔨 в работе · ⏳ ждёт · ➖ переносить нечего.
«Готово» значит: код в `main` форка, тесты прошли, сборка для Windows
собрана. Что проверено только тестами, а не руками в приложении, помечено.

Кто где работает: **Claude** — в `main`, папка `C:\Users\z1vas\yapfast`;
**Kimi** — в ветке `profile-cosmetics`, папка
`C:\Users\z1vas\yapfast-profile-cosmetics`. Ветку Kimi сливаем в `main`,
когда она готова.

## Сейчас

| Задача | Кто | Статус | Заметки |
|---|---|---|---|
| Меню (выпадающие списки) ботов | Claude | 🔨 | Все слои написаны, идёт проверка |
| Формы (модалки) ботов | Claude | 🔨 | Все слои написаны, идёт проверка |
| Профиль: украшения аватара и бейджи | Kimi | 🔨 | Ветка `profile-cosmetics` |
| Профиль: привязанные аккаунты | Kimi | 🔨 | Ветка `profile-cosmetics` |

## Исправления и функции

| Задача | Статус | Заметки |
|---|---|---|
| Сборка Windows-версии в WSL (Nix) | ✅ | Чинилась ошибка `syn` у автора; Skia собирается в 12 потоков |
| Кнопки ботов (`POST /interactions`, тип 3) | ✅ | Тесты прошли; руками в приложении не проверено |
| Ctrl+A/C/V/X/Z на русской раскладке | ✅ | Руками не проверено |
| Метки времени `<t:…:t/T/d/D/f/F/R>` и 12/24 часа | ✅ | Тест прошёл |
| Цвета ролей в чате после загрузки ролей | ✅ | Руками не проверено |
| «Это взаимодействие не удалось» при отказе бота | ⏳ | Событие `INTERACTION_FAILURE` уже приходит |
| PinDMs: закреплённые ЛС по категориям + импорт из Vencord | ⏳ | Категории Pin, Work, Increase, Studio, Anime Paradise с цветами |
| Импорт настроек плагинов из Vencord | ⏳ | Из `%APPDATA%\Vencord\settings\settings.json` |
| Если нужно — смотреть, как это делает сам Discord (его веб-клиент) | — | Принцип работы: повторять поведение официального клиента |
| Профиль: украшения аватара | 🔨 | Kimi; `avatar_decoration_data`, CDN `avatar-decoration-presets` |
| Профиль: бейджи | 🔨 | Kimi; `badges`, CDN `badge-icons` |
| Профиль: привязанные аккаунты | 🔨 | Kimi; `connected_accounts` |
| Профиль: цвета темы, эффекты профиля, тег сервера, nameplate | ⏳ | `theme_colors`, `profile_effect`, `primary_guild`, `collectibles` |
| Профиль: тег сервера (MCS, AC, INCR…) рядом с именами в ЛС, участниках и чате | ⏳ | `primary_guild`, как на скриншоте закреплённых ЛС |
| Плавная прокрутка и анимации под 240 Гц | ⏳ | Сейчас ~40–50 fps в списке каналов сервера; Windows выдаёт 164 Гц — проверить и отрисовщик, и кадры |
| Колонка «Active Now» на странице друзей | ⏳ | Кто во что играет и сколько (Dota 2 – 13m), кто в каком голосовом канале с аватарками, группировка, Rich Presence (Medal: «Clipping Dota 2», время) |
| MCP-сервер для Discord через запущенный Yapfast | ⏳ | Читать и писать в каналах и ЛС; заходить на серверы; управлять сервером (каналы, роли, настройки); менять свой профиль: описание, аватар, баннер, статус |
| Одно приложение на всё | ⏳ | Цель: Discord с плагинами + MCP, потом Telegram |
| Telegram | ⏳ | Отложено по просьбе |

## Плагины Vencord (только включённые у тебя)

### Переносить нечего ➖

Внутренности Vencord или веб-клиента: ChatInputButtonAPI, CommandsAPI,
ContextMenuAPI, DynamicImageModalAPI, MemberListDecoratorsAPI,
MessageAccessoriesAPI, MessageDecorationsAPI, MessageEventsAPI,
MessagePopoverAPI, MessageUpdaterAPI, NoticesAPI, ServerListAPI,
UserSettingsAPI, BadgeAPI, Settings, SupportHelper,
ConcatenatedComponentExtractor, ConsoleJanitor, CrashHandler,
ReactErrorDecoder, FixDiscordCss, FixCodeblockGap, VencordToolbox,
BetterSettings, NoTrack (Yapfast и так не шлёт аналитику), YoutubeAdblock и
SpotifyCrack (Yapfast не проигрывает их внутри себя).

### Сообщения и чат

| Плагин | Статус | Заметки |
|---|---|---|
| ClearURLs | ✅ | Уже есть в Yapfast: «вырезать трекинг из ссылок» |
| MessageLogger | ⏳ | |
| SilentTyping | ⏳ | |
| QuickReply | ⏳ | |
| QuickMention | ⏳ | |
| MessageClickActions | ⏳ | |
| ReplyTimestamp | ⏳ | |
| ValidReply | ⏳ | |
| ValidUser | ⏳ | |
| PreviewMessage | ⏳ | |
| Translate | ⏳ | |
| ShikiCodeblocks | ⏳ | |
| MessageLatency | ⏳ | |
| Summaries | ⏳ | |
| TypingIndicator | ⏳ | |
| TypingTweaks | ⏳ | |
| NoTypingAnimation | ⏳ | |
| UserMessagesPronouns | ⏳ | |
| ForceOwnerCrown | ⏳ | |
| petpet | ⏳ | |
| SpotifyShareCommands | ⏳ | |
| CopyFileContents | ⏳ | |

### Медиа, файлы, эмодзи

| Плагин | Статус | Заметки |
|---|---|---|
| ImageZoom | ⏳ | Проверить, что уже есть |
| PictureInPicture | ⏳ | Проверить, что уже есть |
| ViewIcons | ⏳ | |
| UnlockedAvatarZoom | ⏳ | |
| FixImagesQuality | ⏳ | |
| ImageFilename | ⏳ | |
| AnonymiseFileNames | ⏳ | |
| BetterUploadButton | ⏳ | |
| AddAttachments / GifPaste / StickerPaste | ⏳ | |
| BetterGifPicker | ⏳ | |
| FavoriteGifSearch | ⏳ | |
| FavoriteEmojiFirst | ⏳ | |
| VoiceMessages | ⏳ | Проверить, что уже есть |
| VoiceDownload | ⏳ | |

### Люди, ЛС, серверы

| Плагин | Статус | Заметки |
|---|---|---|
| PinDMs | ⏳ | См. выше |
| PlatformIndicators | ⏳ | Проверить, что уже есть |
| PermissionsViewer | ⏳ | Проверить, что уже есть |
| BetterRoleContext | ⏳ | |
| BetterNotesBox | ⏳ | |
| BetterSessions | ⏳ | |
| RelationshipNotifier | ⏳ | |
| SortFriendRequests | ⏳ | |
| NoPendingCount | ⏳ | |
| MemberCount | ⏳ | Проверить, что уже есть |
| ServerInfo | ⏳ | |
| NewGuildSettings | ⏳ | |
| ReadAllNotificationsButton | ⏳ | |
| OnePingPerDM | ⏳ | |
| NotificationVolume | ⏳ | |
| AccountPanelServerProfile | ⏳ | |
| UserVoiceShow | ⏳ | |
| GameActivityToggle | ⏳ | Проверить, что уже есть |

### Голос, звонки, стримы

| Плагин | Статус | Заметки |
|---|---|---|
| CallTimer | ⏳ | Проверить, что уже есть |
| VolumeBooster | ⏳ | |
| StreamerModeOnStream | ⏳ | |
| SpotifyControls | ⏳ | |

### Прочее

| Плагин | Статус | Заметки |
|---|---|---|
| LoadingQuotes | ⏳ | |

## Справка для разработки

Данные профиля в API Discord (по docs.discord.food), для задач профиля:

- Украшение аватара: `user.avatar_decoration_data.asset` →
  `https://cdn.discordapp.com/avatar-decoration-presets/{asset}.png?size=…&passthrough=true`
  (у `a_…` анимированная APNG; `passthrough=false` — статичная).
- Эффект профиля: `user_profile.profile_effect.id`; картинки эффекта —
  в списке эффектов (путь эндпоинта проверить по трафику клиента).
- Бейджи: `badges[]` (`id`, `description`, `icon`, `link`) →
  `https://cdn.discordapp.com/badge-icons/{icon}.png`; плюс `guild_badges`.
- Привязки: `connected_accounts[]` (`type`, `id`, `name`, `verified`);
  иконки поставляются с клиентом.
- Цвета темы: `user_profile.theme_colors` [основной, акцент], запасной —
  `accent_color`.
- Тег сервера: `user.primary_guild` → `https://cdn.discordapp.com/guild-tag-badges/{identity_guild_id}/{badge}.png`.
- Nameplate: `user.collectibles.nameplate.asset` →
  `https://cdn.discordapp.com/assets/collectibles/{asset}static.png` (анимация — `asset.webm`).
