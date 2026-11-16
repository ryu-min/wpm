# Tasks

## Согласование с remove-dictation — 16 ноября 2026

Активные proposal/design/specs согласованы с [remove-dictation](../remove-dictation/proposal.md). Ниже сохранены исторические задачи, их фактические отметки и результаты прежних раундов; они не являются поручением повторно вводить отменённое поведение. Текущая реализация и регрессионная приёмка выполняются по [задачам remove-dictation](../remove-dictation/tasks.md). Упоминания прежних записей об удалённых аудиоспецификациях и старых путях кода относятся к тому историческому состоянию; актуальные требования доступны в оставшихся specs этого change и remove-dictation.

Разделы 1–3 фиксируют выполненный первый раунд с частичной миграцией. Его результаты ниже исторические: они не подтверждают уточнённый пользователем объём «все наборы в одной форме». Актуальная доработка и повторная приёмка — открытые задачи раздела 4; решения о legacy-ресурсах заменены текущими proposal/spec/design.

## 1. Формат и автоматическая поставка

- [x] 1.1 Добавить общий parser/validator JSON и пять профилей из spec; проверить общими тестами допустимые данные, неизвестные поля/профили, пустые значения, дубли id/точных NFC-target и ограничения испанского контента. Написать `docs/datasets.md` с форматом, примерами и границами v1.
- [x] 1.2 Реализовать discovery/embedding в `build.rs` и общий источник legacy-регистрации для диагностики коллизий без дублирующего списка id; проверить корректную сборку, адресные ошибки и присутствие новых ресурсов в `cargo package --list --allow-dirty`.

## 2. Использование профиля и миграция кафе

- [x] 2.1 Подключить декларативные наборы к существующим таблицам без изменения схемы, добавляя отсутствующие id транзакционно; общими интеграционными/E2E-проверками подтвердить два запуска старой БД, сохранность данных/выбора и добавление Typing при непустой таблице.
- [x] 2.2 Получать маршрутизацию и списки режимов из descriptor/legacy-адаптера, передавать существующую политику диктанта из профиля вместо id; параметризованными E2E пяти профилей проверить допустимые режимы, Settings/Select Mode/Quick Start, сохранение выбора, ввод и выход. Проверить id без языкового префикса и прежние RU/EN/ES наборы.
- [x] 2.3 Перевести кафе в один descriptor, удалить старый ресурс/ручную регистрацию/ветку по id; сравнить все 31 тройки id/ru/target с прежним ресурсом и зафиксировать равенство в результате задачи. Обновить пути в документации, сохранить пользовательское поведение и SDK evidence; поиск в производственном Rust не должен находить специальную обработку id кафе.
- [x] 2.4 Обобщить полезное покрытие `e2e_spanish_cafe.rs` в тесты каталога/профилей с автоматическим обнаружением файлов, убрать дублирование персональных сценариев и копию 31 пары; сохранить проверки case/акцентов/RU-клавиш, полных и неполных фраз, скрытого ответа, TTS/F2/restart, ошибки/выхода и количества записей. Записать краткое соответствие прежних сценариев новым проверкам и выполнить эти E2E.

## 3. Проверка процесса добавления и интеграция

- [x] 3.1 Добавить общую acceptance-проверку временной копии проекта: исходная сборка → только новый JSON → повторная сборка в том же target → PTY-запуск бинарника из пустого каталога → набор доступен/запускается в обоих испанских режимах. Проверить также изменение/удаление файла на свежем WPM_DATA_DIR и build-ошибки неверной metadata/дубликата id. Проверка должна сохранять исходники/manifest/тесты копии неизменными при добавлении набора и не загрязнять рабочее дерево.
- [x] 3.2 Завершить инструкцию «добавить файл → проверить → пересобрать/установить», ссылку из README и оговорку об отсутствии перезаписи существующей БД; проверить инструкцию результатом 3.1. Выполнить `cargo test --locked --all-features --all-targets`, `python3 scripts/validate_spanish_catalog.py`, `openspec validate data-driven-exercise-datasets --strict` и `git diff --check`, записать платформу/результаты. Общую build-acceptance из 3.1 включить в документированный обязательный прогон, если она не входит в cargo test; Unix PTY не считать Windows-покрытием.

## 4. Единая форма всех встроенных наборов

- [x] 4.1 Преобразовать оставшиеся 18 ресурсов в descriptors той же формы, что кафе; одноразово сравнить с исходниками HEAD все 19 последовательностей, количества, тексты/id и существующие повторы. Сохранить `ru_5000` = 4977, RU→EN id `name:<индекс с нуля>`, испанские id и 31 запись кафе; записать результат равенства без постоянной копии данных в тестах.
- [x] 4.2 Удалить старые TXT/JSON, `src/legacy_datasets.rs`, ручную регистрацию и отдельные seed-процедуры; оставить общий discovery/validator/seed для всех descriptors. Адаптировать общие тесты и build-acceptance к коллизиям между descriptors; проверить отсутствие старых загрузочных путей, seed всех 19 наборов, повторный запуск/непустую БД и сохранение локальных данных/выбора.
- [x] 4.3 Перевести `scripts/validate_spanish_catalog.py`, каталог, README и `docs/datasets.md` на единый формат/папку; сохранить содержательную проверку шести уровней и общие автоматические проверки всех descriptors. Проверить 5675 ES-лемм, существующие профили и отсутствие требований добавлять ручные списки/индивидуальные E2E для нового файла.
- [x] 4.4 Повторить `cargo test --locked --all-features --all-targets`, `python3 scripts/check_dataset_build.py`, `python3 scripts/validate_spanish_catalog.py`, `cargo package --list --allow-dirty`, `openspec validate data-driven-exercise-datasets --strict` и `git diff --check`. Записать новую evidence отдельно от истории: package содержит 19 descriptors и не содержит прежних ресурсов/legacy-модуля; standalone binary, новое добавление только JSON и существующие сохранённые наборы работают.

## Исторические результаты первого раунда (15 ноября 2026)

- Общий формат `src/dataset.rs` используется build и runtime; ровно пять профилей.
  `tests/dataset_format.rs` проверяет все профили, неизвестные metadata/поля,
  пустой/null level, пустые/неверные entries, адресный индекс записи, NFC,
  точные дубли id/target, пробелы и испанские ограничения. Разные регистр и
  диакритика остаются различными каноническими target.
- `build.rs` обнаруживает отсортированные JSON напрямую в resources/datasets,
  отслеживает и каталог, и файлы, встраивает сгенерированный каталог из OUT_DIR.
  `src/legacy_datasets.rs` — единственный список прежних ресурсов для seed и
  build collision diagnostics. Новые descriptors не регистрируются в Rust.
- Адаптер SQLite добавляет descriptors транзакционно по отсутствующему id.
  Маршрутизация/готовые mode lists берутся из profile либо фактических
  таблицы/языка legacy БД; несовместимые сохранённые данные не переписываются.
  Общий unit `declarative_profiles_add_to_nonempty_storage_and_preserve_incompatible_rows`
  проверяет пять адаптеров, непустой Typing, идемпотентность и несовместимость.
- Кафе перенесено в `resources/datasets/ru_es_cafe_beginner.json`.
  Одноразовое сравнение старого JSON с новым descriptor подтвердило равенство
  всех 31 тройки id/ru/es→target, включая порядок; основной агент независимо
  подтвердил это сравнением с HEAD. Старый ресурс и персональная Rust-ветка
  удалены. В производственном Rust специального id кафе больше нет.
- `tests/e2e_dataset_profiles.rs` автоматически обнаруживает все descriptors:
  точный seed, два запуска прежней БД, сохранённые локальные edits/выбор,
  Settings/Select Mode/запуск во всех допустимых режимах. Полное прохождение
  испанских descriptors динамически проверяет размер, отсутствие повторов,
  канонические ответы, переводы и TTS (при длине 50 — до 50 записей).
- Сценарии прежнего cafe E2E обобщены без копии 31 пары: hidden/F2/TTS/restart
  сохранены в `phrase_is_hidden_repeated_canonically_checked_once_and_keeps_policy_on_restart`;
  case/accents/RU/пробелы/неполная фраза/пропуск Yo/артикля/Escape — в
  `spanish_profiles_check_whole_phrases_spaces_layout_and_optional_accents`;
  Translation, Enter, 100%/94% и отсутствие аудио — в
  `translation_types_complete_pairs_without_audio_and_retains_case`.
  Параметризованный `five_profiles_route_settings_select_mode_quick_start_and_persist`
  проверяет пять профилей, непредфиксные id, настройки, Quick Start и прямой
  запуск; навигация вычисляется из содержимого БД вместо постоянного числа клавиш.
- `scripts/check_dataset_build.py`: PASS. Во временной копии baseline build →
  добавлен только practice_new.json → повторная сборка в том же target →
  standalone PTY из пустого cwd, оба испанских режима. Также PASS изменение,
  удаление на свежих WPM_DATA_DIR, неизвестный profile, пустой target,
  коллизия с legacy и два файла одного id с указанием обоих источников.
  Snapshot подтверждает отсутствие других изменений в копии; исходный checkout
  этой проверкой не изменяется. Это отдельная обязательная Unix-проверка в docs.
- `cargo package --list --allow-dirty`: PASS, descriptor/build.rs/parser/legacy
  присутствуют; локальный SDK config не попадает в package.
- Платформа: macOS / Darwin arm64. Локальный ignored .cargo/config.toml сохранён,
  cargo работает без внешнего SDKROOT. SDK evidence и открытые ручные аудио/Windows
  задачи прежних изменений не менялись и не считаются выполненными этим apply.

### Итоговый обязательный прогон

- `cargo test --locked --all-features --all-targets`: PASS, 81 тест в 13
  тестовых бинарниках. Финальный лог: `/tmp/wpm-datasets-final-tests.log`.
  Один промежуточный прогон одновременно со свежей сборкой временной копии
  получил Timeout в двух прежних audio unit с лимитом 1s; без этой параллельной
  сборки повторный полный прогон прошёл. Audio production/tests не менялись.
- `python3 scripts/check_dataset_build.py`: PASS повторно на финальном build
  механизме; `/tmp/wpm-dataset-final-build-acceptance.log`.
- `python3 scripts/validate_spanish_catalog.py`: PASS, 5675 прежних лемм.
- `cargo package --list --allow-dirty`: PASS; дополнительно подтверждены
  docs/datasets.md и acceptance script в package; .cargo/config.toml исключён.
- `openspec validate data-driven-exercise-datasets --strict`: PASS.
- `git diff --check`: PASS. Никакие прежние change tasks не закрывались,
  системные настройки, SDK config и пользовательская БД не менялись.

## Исправление общего E2E для длинных данных

Общие descriptor-проверки раньше использовали 80 колонок и требовали полный
Answer/Translation либо id на одном снимке. Это ошибочно отклоняло допустимый
новый JSON с длинными фразами. Производственный код и лимиты формата не менялись.

- Общий PTY helper теперь принимает geometry с прежним default 80×24. Реальный
  `script`/stty и восстановленный Surface используют одинаковую геометрию.
- Автоматические descriptor-проверки вычисляют ширину по экранной Unicode-ширине
  id, слов либо полной пары ru/target с запасом под 70%-ную область Translation
  и метки Dictation. Полные канонические текстовые проверки сохранены.
  Аналогичные полные id assertions пяти-профильного теста используют подходящую
  ширину. Ограничений длины данных ради теста не добавлено.
- Общий `descriptor_checks_handle_long_ids_targets_and_russian_translations`:
  PASS. Он использует id длиннее 80 колонок, испанскую фразу и русский перевод
  из воспроизведения ревью и те же helpers доступности/прохождения каталога.
- `scripts/check_dataset_build.py` повторён с добавлением только временного
  длинного descriptor (длинные id/target/ru), без постоянного нового ресурса:
  PASS, в том числе standalone PTY обоих режимов, incremental change/delete
  и отрицательные build fixtures. В Dictation acceptance проверяется точная
  статистика `Checked: 1  Correct: 1  Errors: 0`, вместо подстроки `Correct`.
  Лог: `/tmp/wpm-datasets-long-build-acceptance.log`.

После исправления длинных данных обязательный
`cargo test --locked --all-features --all-targets`: PASS, 82 теста в 13
бинарниках, без одновременной build-acceptance нагрузки. Финальный лог:
`/tmp/wpm-datasets-review-fix-tests.log`. Strict OpenSpec и diff check: PASS.

## Единая форма всех встроенных ресурсов — новый раунд приёмки

Предыдущая evidence описывает историю первоначального варианта и его ревью.
Новый объём по разделу 4 проверяется отдельно; прежние результаты не считаются
доказательством завершения миграции всех наборов.

### Механическое равенство данных

Одноразово прочитаны исходные ресурсы `git show HEAD:resources/<file>` и
сравнены с новыми упорядоченными entries. Все 19 последовательностей совпали:
Typing — точные непустые строки, RU→EN — точные пары с `name:<индекс с нуля>`,
ES и кафе — точные id/ru/es→target. Все повторы и порядок сохранены, редактура
данных не выполнялась. Основной агент независимо повторил сравнение: PASS.
Хеши канонических entries и количества записаны как локальная evidence в
`/tmp/wpm-all-descriptors-migration-equality.json`; постоянной копии данных в
тестах нет.

| Группа | Количества записей |
| --- | --- |
| en_1000, en_10000 | 1000, 10000 |
| ru_5000, ru_10000, ru_50000 | 4977, 10000, 50000 |
| ru_en_a1, a2, b1, b2, c1, c2, basic | 996, 1000, 1000, 1000, 944, 896, 61 |
| ru_es_a1, a2, b1, b2, c1, c2 | 955, 919, 935, 945, 951, 970 (всего5675) |
| ru_es_cafe_beginner | 31 |

### Общий механизм и содержательные проверки

- Все ресурсы теперь descriptors в `resources/datasets/`. Удалены старые пять
  TXT, тринадцать JSON A1–C2/RU→EN (кафе уже перенесено), legacy_datasets.rs,
  ручной registry и отдельные seed_data/seed_spanish. Один descriptor-seed
  транзакционно добавляет отсутствующие id; SQLite fallback сохраняется только
  для локальных строк, не имеющих descriptor.
- `cargo check --locked`: PASS; общий parser принял все19 файлов без исключений
  и изменения правил. Focused Rust unit+format:20 PASS. Проверка rollback теперь
  подтверждает отсутствие вставок во всех четырёх таблицах после ошибки seed.
- ES validator читает `resources/datasets/*.json` → entries/target, сохраняя
  содержательные ограничения шести уровней: PASS5675. На временных descriptors
  повторены11 отрицательных content mutations (пустой уровень, дубли target/id,
  NFD, заглавные/фраза, пустой перевод/заглушка, кириллица в target, неверный id,
  неверный тип): все отклонены. Исходные данные не изменялись проверкой.
- `python3 scripts/check_dataset_build.py`: PASS; standalone binary и JSON-only
  incremental add/change/delete работают поверх19 descriptors. Диагностика
  коллизии с перенесённым RU→EN называет два файла descriptors. Лог:
  `/tmp/wpm-unified-build-acceptance.log`.
- `cargo package --list --allow-dirty`: PASS, ровно19 descriptors; старых TXT/JSON,
  legacy-модуля и локального SDK config в package нет. Лог:
  `/tmp/wpm-unified-package-list.txt`.

- Автоматический обход всех19 descriptors через бинарник: PASS (77.19s),
  точный seed и два повторных запуска/локальные edits/сохранение выбора,
  Settings/Select Mode и запуск каждого набора в допустимых режимах.
  Первый прогон на больших Typing-словарях проверял содержимое до отрисовки
  первого кадра; helper теперь явно ждёт `0 wpm` перед снимком, аналогично
  ожиданию Audio ready. Проверки полного текста и descriptor geometry сохранены.
  Лог focused: `/tmp/wpm-unified-every-descriptor.log`.

### Финальный прогон единого формата

- `cargo test --locked --all-features --all-targets`: PASS, 82 теста в13
  бинарниках, включая автоматический обход всех19 descriptors и полное
  прохождение семи испанских наборов (до50 записей каждого). Прогон выполнялся
  без параллельной свежей build-acceptance; лог `/tmp/wpm-unified-final-tests.log`.
- `python3 scripts/check_dataset_build.py`: PASS на едином формате19 наборов;
  `python3 scripts/validate_spanish_catalog.py`: PASS5675; дополнительно11
  отрицательных content cases отвергнуты после смены чтения entries/target.
- `cargo package --list --allow-dirty`: PASS19 descriptors и общий механизм,
  без прежних ресурсов/legacy-модуля и без `.cargo/config.toml`.
- `openspec validate data-driven-exercise-datasets --strict` и
  `git diff --check`: PASS. Производственный поиск не обнаруживает LEGACY,
  legacy_datasets, include_str старых ресурсов или отдельных старых seeds.
- Платформа macOS/Darwin arm64. Локальный SDK override сохранён; обычные cargo
  команды работают без внешнего SDKROOT. Старые незавершённые проверки звука
  и Windows не менялись. Commit/push/archive не выполнялись.
