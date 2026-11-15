# Tasks

## 1. Локальная настройка и документация

- [x] 1.1 Добавить в `.gitignore` узкое исключение `/.cargo/config.toml` и компактное описание локального выбора совместимого macOS SDK в README; проверить diff, сохранность существующих правок и отсутствие требования SDK 15.4 для других машин.
- [x] 1.2 Создать локальный `.cargo/config.toml` с проверенным путём SDK 15.4 и `force = false`, сохранив другие ключи при их появлении; проверить существование SDK, корректность TOML, `git check-ignore -v .cargo/config.toml` и отсутствие этого файла в `git ls-files`.

## 2. Проверка исходной команды

- [x] 2.1 В процессе без внешнего `SDKROOT` выполнить свежую сборку с отдельным пустым `CARGO_TARGET_DIR`; подтвердить успешную линковку без ошибки `arm64e.x1-macos` и записать команду/результат, не очищая текущий target.
- [x] 2.2 Запустить обычный `cargo run` из корня checkout без внешнего `SDKROOT` в псевдотерминале с изолированным `WPM_DATA_DIR`; подтвердить появление меню и штатный выход, записать результат. Это проверка локального инструментария; новые продуктовые E2E не требуются.
- [x] 2.3 Выполнить `cargo test --locked --all-features --all-targets` без внешнего `SDKROOT`, записать результат и выполнить `openspec validate fix-macos-cargo-run-sdk --strict`; проверить, что отслеживаемые изменения этого исправления ограничены ignore, документацией и его OpenSpec-артефактами.

## Результаты реализации и проверки

- В `.gitignore` добавлено только `/.cargo/config.toml`. В README добавлен раздел `Local macOS SDK override`: заменяемый путь к совместимому установленному SDK, запуск из корня checkout, приоритет внешнего SDKROOT и удаление обхода после исправления toolchain. SDK 15.4 не объявляется общей зависимостью проекта.
- Создан только локальный `.cargo/config.toml`: `[env]` с SDKROOT `/Library/Developer/CommandLineTools/SDKs/MacOSX15.4.sdk`, `force = false`. Перед созданием проверено отсутствие старого `.cargo/config` и нового конфига; существующих ключей для переноса не было. Каталог SDK существует.
- `cargo metadata --locked --no-deps --format-version 1` успешно прочитал TOML. `git check-ignore -v .cargo/config.toml` показал `.gitignore:3:/.cargo/config.toml`; `git ls-files -- .cargo/config.toml` не вернул файлов. Глобальные настройки, developer directory и shell profile не менялись.
- Свежая сборка `cargo build --locked` выполнена с новым пустым `CARGO_TARGET_DIR=/var/folders/r4/020lyrb12_g29s296t0jwqv40000gp/T/wpm-fresh-sdk-yy169ugo`. SDKROOT удалён из окружения непосредственно перед запуском Cargo. Весь граф зависимостей и wpm слинкованы успешно, `Finished dev ... in 11.22s`; ошибки arm64e.x1-macos нет. Существующий target не очищался.
- Обычный `cargo run` запущен из корня checkout в PTY через `script`, без SDKROOT в окружении Cargo. Изолированные данные: `/var/folders/r4/020lyrb12_g29s296t0jwqv40000gp/T/wpm-cargo-run-sdk-cwl9jgfb`. Наблюдалось главное меню с `> Quick Start`; Escape завершил процесс с кодом 0. Терминальное свидетельство: `/tmp/wpm-sdk-cargo-run-evidence.txt`.
- Для свежей сборки и PTY окружение дочернего Cargo формировалось копированием окружения с явным удалением SDKROOT. Это существенно, потому что системный `/usr/bin/python3` на этом Mac сам добавляет SDKROOT при запуске; это значение не передавалось проверяемому Cargo. Команда полного прогона запускается напрямую как `env -u SDKROOT cargo test --locked --all-features --all-targets`.
- Изменения реализации этого change ограничены `.gitignore`, новым разделом README и локальным ignored-конфигом. Все прежние правки испанских режимов, корпуса и других файлов сохранены; новые тесты для простой локальной настройки не добавлялись.
- `env -u SDKROOT cargo test --locked --all-features --all-targets`: PASS, 17 модульных/архитектурных и 55 терминальных E2E, всего 72. `openspec validate fix-macos-cargo-run-sdk --strict`: PASS; отсутствие delta specs принято по `skip_specs: true`.

## Повторное восстановление (15 ноября 2026)

При повторном обращении пользователя каталог `.cargo` в checkout отсутствовал.
Причина его отсутствия не установлена. Системный выбор SDK по-прежнему указывал
на несовместимый MacOSX27.0.sdk; совместимый MacOSX15.4.sdk существует.

По прежнему дизайну восстановлен только локальный ignored `.cargo/config.toml`
с SDKROOT `/Library/Developer/CommandLineTools/SDKs/MacOSX15.4.sdk` и
`force = false`. Системные настройки, исходники приложения, README и изменения
`add-spanish-cafe-beginner-set` при восстановлении не менялись. Никаких
предположений о том, почему локальный конфиг исчез, не делалось.

Повторные проверки:

- `env -u SDKROOT cargo metadata --locked --no-deps --format-version 1`: PASS,
  конфигурация TOML читается Cargo.
- `git check-ignore -v .cargo/config.toml`: `.gitignore:3:/.cargo/config.toml`;
  `git ls-files -- .cargo/config.toml` пуст, локальный путь не отслеживается.
- Свежий `cargo build --locked` с пустым временным
  `CARGO_TARGET_DIR=/var/folders/r4/020lyrb12_g29s296t0jwqv40000gp/T/wpm-sdk-restored-fresh-hig93gd5`:
  PASS, полная сборка и линковка за 12.30s. SDKROOT явно удалён из окружения
  дочернего Cargo после запуска Python; существующий target не очищался.
  Лог: `/tmp/wpm-sdk-restored-fresh-build.log`.
- Обычный `cargo run` из корня checkout через PTY `script`, без SDKROOT в
  окружении дочернего Cargo: меню `> Quick Start` появилось; Escape завершил
  процесс с кодом 0. Данные изолированы в
  `/var/folders/r4/020lyrb12_g29s296t0jwqv40000gp/T/wpm-sdk-restored-data-628hbfkh`.
  Терминальное свидетельство: `/tmp/wpm-sdk-restored-cargo-run-evidence.txt`.
- `env -u SDKROOT cargo test --locked --all-features --all-targets`: PASS,
  79 тестов в 12 бинарниках на macOS / Darwin arm64; лог
  `/tmp/wpm-sdk-restored-tests.log`.
- `openspec validate fix-macos-cargo-run-sdk --strict`: PASS; `skip_specs: true`
  сохранён. `git diff --check`: PASS. Отслеживаемый diff повторного восстановления
  ограничен этим разделом результатов; игнорирование и инструкция уже существовали.
