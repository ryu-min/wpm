# Spec Delta

## MODIFIED Requirements

### Requirement: Install from the source repository
Проект SHALL устанавливаться Cargo из существующего Git-репозитория с командами `wpm` и `wpm-es`.

#### Scenario: Install from Git
- **WHEN** пользователь с Rust и Cargo устанавливает проект из `https://github.com/ryu-min/wmp` документированной командой
- **THEN** Cargo устанавливает команды `wpm` и `wpm-es`, запускающие соответствующие терминальные приложения

### Requirement: Install from a local checkout
Проект SHALL устанавливаться Cargo из локальной рабочей копии с командами `wpm` и `wpm-es`.

#### Scenario: Install from checkout
- **WHEN** пользователь выполняет документированную команду локальной установки Cargo
- **THEN** Cargo устанавливает команды `wpm` и `wpm-es`, запускающие соответствующие терминальные приложения

### Requirement: Verify installation in CI
Проект SHALL проверять установку текущей рабочей копии в изолированный Cargo root на каждом pull request и push. На основной ветке SHALL дополнительно проверяться документированная установка из публичного Git-репозитория. CI SHALL завершаться ошибкой при сбое установки или отсутствии любой команды `wpm` и `wpm-es`.

#### Scenario: Pull request or branch push
- **WHEN** CI выполняется для pull request или push в ветку
- **THEN** Cargo устанавливает рабочую копию, и проверяется наличие обеих команд `wpm` и `wpm-es` с расширением `.exe` на Windows

#### Scenario: Default branch push
- **WHEN** CI выполняется после push в основную ветку
- **THEN** дополнительно выполняется установка из Git с проверкой обеих команд `wpm` и `wpm-es` с расширением `.exe` на Windows

### Requirement: Document installation and purpose
README SHALL описывать практику печати и перевода, назначение курса, команды установки и запуска обоих приложений, необходимость Cargo, настройку PATH и каталог данных `WPM_ES_DATA_DIR`.

#### Scenario: New user follows the README
- **WHEN** новый пользователь читает README
- **THEN** он понимает назначение приложений и условия установки и запуска `wpm` и `wpm-es`

## ADDED Requirements

### Requirement: Изолированные настройки курса

Приложение `wpm-es` SHALL использовать `WPM_ES_DATA_DIR` как необязательный каталог данных и отдельный каталог приложения по умолчанию. Файл настроек курса SHALL сохранять выбор независимо от настроек `wpm`. Обычная команда `cargo run --locked` SHALL продолжать запускать `wpm`, а `cargo run --locked --bin wpm-es` SHALL запускать курс.

#### Scenario: Изоляция настроек
- **WHEN** обе команды меняют свой выбор и перезапускаются с отдельными каталогами
- **THEN** каждая восстанавливает собственные значения

#### Scenario: Локальный запуск курса
- **WHEN** выполнено `cargo run --locked --bin wpm-es`
- **THEN** открыт главный экран «Испанский»
