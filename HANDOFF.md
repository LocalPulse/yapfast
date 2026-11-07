# Передача работы: от Claude к Kimi

Всё, что дальше делалось Claude, передаётся Kimi. Здесь — что готово, что
начато, как собирать и проверять, и на чём легко споткнуться. Список задач
со статусами — `TASKS.md`; правила проекта — `CLAUDE.md` и
`docs/discord/adding-a-feature.md` (их стоит прочитать первыми).

## Где что лежит

- GitHub: https://github.com/LocalPulse/yapfast (форк `yapfast/yapfast`,
  `upstream` — оригинал).
- Основная копия: `C:\Users\z1vas\yapfast`. Ветки:
  - `main` — готовое и проверенное (до коммита `aab8a0af` включительно, плюс
    правки `TASKS.md`).
  - `wip/pindms-active-now` — **начатое Claude, не проверенное**: PinDMs и
    Active Now (коммит `610378bd`). Сейчас эта ветка открыта в основной копии.
- Твоя копия: `C:\Users\z1vas\yapfast-profile-cosmetics`, ветка
  `profile-cosmetics`.
- Копия для сборки в WSL: `/home/kali/yapfast` (Kali, Nix из пакетов Kali).
  Её перезаписывает синхронизация из `C:\Users\z1vas\yapfast` — не правь там
  руками.
- Установленная программа: `C:\Users\z1vas\AppData\Local\Programs\Yapfast-fork-20261107-0318`
  (меню и формы ботов уже внутри); ярлыки «Yapfast» на рабочем столе и в
  «Пуске» ведут туда. Версия автора — `...\Programs\Yapfast`, на откат.

## Что готово (в `main`)

- Сборка Windows-версии в WSL: исправлен lock-файл сборщика Skia
  (`prettyplease 0.3`), `YAPFAST_SKIA_JOBS` задаёт число потоков Skia.
- Кнопки, меню и формы (модалки) ботов: `POST /interactions` типы 3 и 5,
  `INTERACTION_MODAL_CREATE`, `INTERACTION_FAILURE` («This interaction
  failed.»). Код: `crates/yapfast-discord/src/components.rs` (модели `Menu`,
  `Modal`, `modal_answer`), `service.rs` (`press_button`, `submit_modal`),
  `crates/yapfast-core/src/application/modals.rs`, `src/ui/app_forms.rs`,
  `ui/app_modal.slint`. Тесты: `presses_an_apps_buttons` (fake Discord),
  `an_apps_buttons_press_unless_disabled`, `an_apps_menus_…`,
  `an_apps_form_…`. Не умеют: меню людей/ролей/каналов, загрузку файлов в
  формах.
- Ctrl+A/C/V/X/Z и свои сочетания Yapfast на русской/украинской раскладке:
  `vendor/i-slint-core/window.rs` + `input.rs` (`latin_on_same_key`).
- Метки времени `<t:…:STYLE>` по стилю и с 12/24 часами:
  `crates/yapfast-discord/src/message.rs` (`timestamp_at`).
- Цвета ролей в чате обновляются, когда роли приходят из списка участников
  или профиля.

Руками в приложении из этого ничего не проверено — только тестами.

## Что начато (ветка `wip/pindms-active-now`)

### PinDMs (`src/ui/pin_dms.rs`)

- Категории хранятся в `Layout` (`pin_categories`, `vencord_pins_for`), не в
  core. Импорт один раз на аккаунт из
  `%APPDATA%\Vencord\settings\settings.json`,
  `plugins.PinDMs.userBasedCategoryList[<discord user id>]`. У владельца там 5
  категорий: Pin, Work, Increase, Studio, Anime Paradise.
- В `src/ui/refresh.rs` (`show_sidebar`): порядок ЛС — старые плоские пины,
  потом категории, потом остальные под «DIRECT MESSAGES». Заголовки цветные
  (`RoomRow.heading-color/heading-tinted`, `CategoryHeading.tint` в
  `ui/rail.slint`), сворачиваются (`layout.collapsed["@me"]`,
  `on_toggle_category` в `callbacks.rs`).
- Есть юнит-тест импорта. **Нет**: управления категориями (создать,
  переименовать, перекрасить, переложить ЛС через меню) — только импорт и
  показ.
- Документация: абзац в `docs/chat-ui.md`.

### Active Now (`src/ui/active_now.rs`, `ui/active_now.slint`)

- Колонка 360px справа от списка друзей, когда ширина ≥ 1000px
  (`ui/yapfast.slint`, рядом с `FriendsScreen`). Карточки: друзья в одном
  голосовом канале сервера (с аватарками всех, кто там, и «+N»), или друзья в
  одной игре; строки Rich Presence (детали, состояние, «01:19:45 elapsed»).
  Таймеры — через `ActivityClock`, свой `active_now_clock` в `Adapter`.
- Последнее состояние: `cargo check` проходит; Clippy ругался на
  `items_after_test_module` — исправлено (impl перенесён выше тестов), но
  повторная проверка не завершилась. **Тесты не прогнаны.**

Что сделать с веткой: прогнать проверку (ниже), поправить, что упадёт,
посмотреть руками, затем слить в `main` и обновить `TASKS.md`.

## Как собирать и проверять

Все команды — в WSL через Nix. **Запускай WSL из PowerShell или файлом-
скриптом**, не из Git Bash: Git Bash переписывает пути `/mnt/...` в
`C:/Program Files/Git/mnt/...` и теряет переменные `$…`. Из-за этого однажды
синхронизация скопировала весь корень WSL в репозиторий (остатки вынесены в
`C:\Users\z1vas\yapfast-stray-wsl-copy`, папка `home/` в репозитории исключена
через `.git/info/exclude` — не коммить её).

Готовые скрипты Claude (можно взять за основу) лежат в
`C:\Users\z1vas\AppData\Local\Temp\claude\29abb69cf93759cc\b8e55b27-666b-44bc-a4ef-b67b47461cb5\scratchpad\`:

- `ui-check.sh` — синхронизирует копию в `/home/kali/yapfast` и делает
  `cargo check -p yapfast` (быстро ловит ошибки Slint).
- `check-main.sh` — синхронизация, `cargo fmt` (отформатированные `.rs`
  копируются обратно поштучно), Clippy с `-D warnings`, все тесты с
  `--no-fail-fast`, тест кнопок на fake Discord.
- `build-windows.sh` — Windows-сборка (`scripts/build-windows discord
  --no-installer`, `YAPFAST_SKIA_JOBS=12`), ZIP и SHA256SUMS кладёт в
  `scratchpad\fork-build\`.

Запуск: `wsl -d kali-linux -u kali -- bash /mnt/c/.../ui-check.sh` из
PowerShell.

Полный набор тестов fake Discord: `scripts/test-fake-discord` (нужен
`ffprobe` — установлен через apt). В WSL apt настроен на мёртвый прокси
`socks5h://172.18.112.1:2080` (`/etc/apt/apt.conf.d/95proxies`): ставь пакеты
с `-o Acquire::http::Proxy=DIRECT -o Acquire::https::Proxy=DIRECT`.

Время: `cargo check` после правки Slint ~3–4 мин; Clippy ~6 мин; тесты ~1
мин; Windows-сборка ~6 мин (Skia уже в кеше).

### Память

В WSL ~15 ГБ, одна компиляция `yapfast-ui` берёт 6–8 ГБ. Две сборки
одновременно уронили WSL (`Catastrophic failure`, `Wsl/Service/E_UNEXPECTED`).
Собирать по одной. Если нужно параллельно — дать WSL больше памяти в
`%USERPROFILE%\.wslconfig` (у машины 32 ГБ).

### Установка новой сборки

Проверить хеш ZIP по `SHA256SUMS.txt`, распаковать в новую папку
`...\Programs\Yapfast-fork-<дата>`, `Unblock-File` на `yapfast.exe`,
перенаправить оба ярлыка. Старые папки не трогать — это откат. Работающий
Yapfast не закрывать без спроса.

## Что осталось (из `TASKS.md`)

- Доделать и слить PinDMs и Active Now (выше).
- Плавная прокрутка и анимации на частоте экрана: цель — ровные 164 кадра
  (монитор нарочно на 164 Гц). Сейчас ~40–50 fps в списке каналов сервера.
  Проверить: какой отрисовщик реально работает (Skia GPU или программный
  запасной — `--renderer`), ограничение кадров/vsync, перестройку строк
  списка на каждый кадр. Предлагалось добавить показ fps по переменной
  окружения и попросить владельца полистать.
- Импорт настроек плагинов из Vencord (по мере переноса плагинов).
- MCP-сервер для Discord через запущенный Yapfast: читать и писать в каналах
  и ЛС, заходить на серверы, управлять сервером, менять свой профиль
  (описание, аватар, баннер, статус). Владельца предупредили: автоматические
  действия от имени пользователя Discord считает self-bot, риск бана выше.
- Около 60 плагинов Vencord — список по группам в `TASKS.md`.
- Профиль (у тебя уже): украшения, бейджи, привязки; свободно — цвета темы,
  эффекты профиля, nameplate, теги серверов рядом с именами.
- Меню людей/ролей/каналов и загрузка файлов в формах ботов.
- Telegram — отложен.

## Договорённости с владельцем

- Только плагины, **включённые** у него в Vencord; их настройки переносить.
- Если непонятно, как сделать, — повторять поведение официального клиента
  Discord.
- Ничего не удалять без спроса; коммиты с понятными сообщениями; `TASKS.md`
  держать в актуальном состоянии.
- Отвечает он по-русски.
