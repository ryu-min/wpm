# Tasks

## 1. Согласование активных спецификаций

- [x] 1.1 Через планировщика согласовать `add-spanish-translation-and-dictation` и `add-spanish-cafe-beginner-set` по таблице design: убрать действующие аудио/dictation deltas и требования, сохранить Translation, корпус и историю; отменённые открытые проверки оставить непройденными с объяснением. Проверить согласованность proposal/design/specs/tasks, отсутствие битых ссылок и `openspec validate <change> --strict` для обоих changes.
- [x] 1.2 Через планировщика согласовать `data-driven-exercise-datasets`, `document-repository-architecture`, `modularize-application-boundaries`, `organize-integration-test-suites` по таблице design. Проверить строгую валидацию каждого и отсутствие в активных deltas требований вернуть audio, Dictation или удаляемые тесты; записать результат в этот tasks, не архивировать changes и не переписывать исторические результаты.

## 2. Удаление режима и совместимость настроек

- [x] 2.1 Удалить аудиомодуль, модель/виджет диктанта, экспорты, варианты режимов/экранов, ветки App и два диктантных метода storage; обновить меню, Select Mode и Settings, сохранив порядок остальных пунктов. Обновить навигационные E2E/helpers, удалить чисто диктантные сценарии и FakeAudio; смешанные сценарии сохранить в части Translation/данных. Проверить сборку и E2E полного обхода меню, режимов и разделов: доступны только два текстовых режима.
- [x] 2.2 Удалить четыре поля настроек; нормализовать прежний Quick Start Dictation в Typing при чтении без записи файла. Добавить E2E со старым JSON, нестандартными значениями всех сохраняемых настроек и произвольными значениями устаревших полей: Quick Start и Settings согласованы, Save/restart убирает старые поля, Escape сохраняет исходный файл. Проверить также новый файл и сохранённый Translation без сброса предпочтений.
- [x] 2.3 Добавить E2E отсутствия вызовов программы из `WPM_TTS_BIN` при меню, настройках и обоих упражнениях с коротким индикатором запуска без аудиопротокола. Обновить README: два режима, сохранённый Spanish Translation и переход старого Quick Start на Typing, без установки eSpeak/настройки звука. Проверить E2E и сверить инструкцию с наблюдаемым интерфейсом.

## 3. Сохранение наборов и текстовых упражнений

- [x] 3.1 Адаптировать смешанные `dataset_profiles`, `spanish_settings`, `spanish_catalog` и межмодульные тесты; сохранить/добавить E2E RU→EN, ES-слов и кафе через оставшиеся пути запуска, строгого регистра, диакритики, RU/ENG-клавиш, Enter/Backspace, результата/restart и отказа пустого/неизвестного набора. Проверить два запуска старой БД с локальными правками и сохранёнными выборами: данные и id не перезаписаны, дублей нет.
- [x] 3.2 Удалить диктантные ветки из `scripts/check_dataset_build.py`, сохранив приёмку добавления/изменения/удаления и ошибок descriptors и запуск вне ресурсов; обновить `docs/datasets.md` и актуальные описания каталога для профилей Translation. Выполнить `python3 scripts/check_dataset_build.py`, `python3 scripts/validate_spanish_catalog.py` и `cargo test --locked --test dataset_format`; подтвердить diff, что все 19 descriptors, схема SQLite и общий формат неизменны.

## 4. Архитектурные границы

- [x] 4.1 Удалить audio из архитектурных разрешений и списка корней, убрать два правила удалённого модуля; сохранить ограничения остальных модулей и согласовать описание с delta `repository-architecture`. Выполнить `cargo test --locked --test architecture`; в временной копии проверить, что UI→storage, app→dataset и новый объявленный исходник `src/audio/` отклоняются конкретными архитектурными правилами. Записать результаты: эти свойства проверяются анализом исходников, поскольку терминальный E2E не проверяет зависимости модулей.

## 5. Интеграционная приёмка

- [x] 5.1 Выполнить `cargo fmt -- --check`, `cargo clippy --locked --all-features --all-targets -- -D warnings`, `cargo doc --locked --no-deps --all-features`, `cargo test --locked --test e2e` и `cargo test --locked --all-features --all-targets`; записать команды, платформу и результаты. Сверить все сценарии exercise-modes с полными именами E2E из `cargo test --locked --test e2e -- --list`, не требуя сохранения числа удалённых тестов. Windows-сборку не выдавать за Unix PTY-приёмку.
- [x] 5.2 Проверить свежую `cargo build --locked` с отдельным target и обычный `cargo run` без SDKROOT/DEVELOPER_DIR в исправленном окружении macOS: изолированные данные, меню из пяти пунктов, штатный Escape. Проверить `cargo install --path . --locked --root <временный-каталог>` и запуск установленного бинарника вне ресурсов. Не создавать SDK override и не менять системный toolchain; записать фактические результаты.
- [x] 5.3 Проверить `git diff --check`, отсутствие продукционного аудиокода и инструкций его настройки; разрешить лишь совместимость старого режима/полей, отрицательные тесты и исторические пояснения. Выполнить `openspec validate remove-dictation --strict` и повторную валидацию шести согласованных changes, убедиться, что будущие deltas не вводят удалённые требования. Зафиксировать результаты и незакрытые ограничения, отмечать задачи только после их выполнения.

## Результаты

На стадии proposal реализация и приёмка не выполнялись; все задачи намеренно оставлены открытыми. Системное исправление SDK выполнено отдельно до этого изменения и не является результатом этих задач.


### Согласование активных изменений (1.1–1.2, 16 ноября 2026)

- Согласованы proposal/design/specs/tasks шести changes из design. Из активного испанского change удалены только dedicated `spanish-audio` и `spanish-dictation` deltas. Дополнительно убраны диктантные требования внутри keyboard-layout-conversion; общий RU/ENG/ES и Unicode-контракт сохранён. Кафе и декларативные профили теперь обещают только Translation.
- Три архитектурные deltas обновлены полными блоками конечного контракта remove-dictation. Поскольку требования уже опубликованы, прежние ADDED в document-repository-architecture и organize-integration-test-suites приведены к MODIFIED. Повторная REMOVED к уже отсутствующему Detect map drift and incomplete analysis исключена; факт прежнего удаления сохранён в истории.
- Проверка текста подтвердила точное сохранение прежних bodies tasks.md и последовательности всех checkbox-отметок во всех шести changes. Открытые аудиопроверки остаются пустыми; добавлены явные объяснения отмены. Исторические команды/результаты не переписаны. Полная таблица 31 пары кафе совпадает с HEAD.
- Сверка будущей архитектурной синхронизации в памяти: каждый полный блок каждой из трёх согласованных deltas совпадает с конечным контрактом (основная спецификация плюс delta remove-dictation), поэтому их применение не возвращает audio или устаревшую структуру. Markdown-ссылки planning-артефактов проверены на существование; активные нормативные требования вернуть диктант отсутствуют. Исторические упоминания сохранены.
- `openspec validate <change> --strict`: PASS для add-spanish-translation-and-dictation, add-spanish-cafe-beginner-set, data-driven-exercise-datasets, document-repository-architecture, modularize-application-boundaries и organize-integration-test-suites. `git diff --check`: PASS.
- На этом этапе изменены только planning-артефакты. Основные specs не синхронизированы, архивирование и кодовые проверки не выполнялись; дальнейшая реализация и приёмка остаются задачами 2.1–5.3.


### Реализация и приёмка (16 ноября 2026, macOS 26.6.1 arm64 / Darwin 25.6.0)

- Удалены `src/audio/`, модель и виджет Dictation, варианты режима/экрана, аудиопереходы App, четыре поля Settings и два диктантных адаптера каталога. Меню содержит пять оставшихся пунктов; Select Mode — два режима; Settings — три раздела. Старый `quick_start_mode: "dictation"` нормализуется только при чтении в `typing`; неизвестные старые поля игнорируются, Save их удаляет, Cancel не переписывает файл.
- Сохранены текстовые части смешанных проверок; удалены отдельный `spanish_dictation` и FakeAudio. Добавлены пять терминальных сценариев `exercise_modes`. Старые поля проверяются со строкой, числом, null, массивом, объектом и boolean; все семь поддерживаемых предпочтений отличаются от defaults, кроме независимых случаев новой установки. Дополнительно сохранена проверка отсутствующего выбранного каталога вместе с пустым и повреждённым.
- `cargo clippy --locked --all-features --all-targets -- -D warnings` — PASS. Для обязательной чистой проверки минимально исправлены прежние замечания: let-chain и `push(']')` в build.rs; alias типа ответа экрана и лишний lifetime в E2E. Семантика формата/загрузки не изменена. `cargo doc --locked --no-deps --all-features` — PASS.
- `cargo test --locked --test architecture` — PASS, 16 правил; `cargo test --locked --test dataset_format` — PASS, 2 теста. `python3 scripts/validate_spanish_catalog.py` — PASS, 5675 уникальных лемм в шести уровнях; допустимые коллизии упрощённого ввода сохранены.
- `python3 scripts/check_dataset_build.py` — PASS: JSON-only добавление, изменение и удаление при повторной сборке, standalone Translation через PTY из пустого каталога, ошибки профиля, записи и конфликтующих id. Из скрипта убраны только диктантный проход и FakeAudio; сообщение результата теперь точно указывает Translation. Проверка нового descriptor сохранена.
- Сверка с HEAD: все 19 `resources/datasets/*.json`, `src/dataset/`, Cargo.toml и Cargo.lock не изменены; тело `init_tables` и SQL-схема побайтово совпадают. E2E `dataset_profiles::every_discovered_descriptor_is_seeded_once_and_preserves_existing_data` проходит по каждому descriptor, проверяет канонические строки, локальные изменения и id после двух запусков, отсутствие дублей и сохранённый выбор.
- Негативные архитектурные проверки выполнены в отдельной временной копии со свежим target. Внесённая UI→storage ссылка отклонена `internal_dependencies::ui_respects_module_boundaries` с `src/ui/mod.rs -> src/storage/mod.rs`; app→dataset — `internal_dependencies::app_respects_module_boundaries` с `src/app/mod.rs -> src/dataset/mod.rs`; новый объявленный `src/audio/mod.rs` — `source_layout::discovered_sources_belong_to_a_documented_module` с указанием недокументированного корня. Все три бинарника тестов скомпилированы, отказ получен от архитектурных утверждений. Это анализ исходников, поскольку терминальный E2E не устанавливает зависимости модулей. Первая проба с общим target отброшена из-за переиспользованного Cargo artifact; в доказательство включён только независимый target.
- Свежая `cargo build --locked` выполнена с отдельным `CARGO_TARGET_DIR`; затем обычная `cargo run --locked` из checkout под PTY показала пять пунктов и штатно завершилась по Escape. `cargo install --path . --locked --root <временный-каталог>` — PASS; установленный бинарник запущен под PTY из пустого каталога вне ресурсов, показал те же пять пунктов и завершился по Escape. SDKROOT и DEVELOPER_DIR отсутствовали; overrides и системные изменения не создавались. Все данные запусков изолированы через WPM_DATA_DIR.

### Соответствие сценариев exercise-modes терминальным тестам

Полные имена сверены с `cargo test --locked --test e2e -- --list` (50 тестов). Сокращение числа прежних диктантных тестов не служит критерием приёмки.

| Сценарий спецификации | Полные имена E2E / дополнительная проверка |
| --- | --- |
| Пользователь обходит все пути выбора | `menu::menu_shows_all_actions_and_exits_on_escape`, `menu::menu_selection_moves_in_both_directions_and_wraps`, `exercise_modes::all_mode_and_settings_choices_contain_only_text_exercises` |
| Оставшиеся режимы завершаются и перезапускаются | `result::completed_typing_shows_result_and_returns_to_menu`, `result::completed_translation_shows_accurate_result`, `result::result_can_restart_test`, `result::translation_result_can_restart`, `result::time_limit_finishes_an_incomplete_attempt`, `layout_policy::typing_defaults_to_layout_sensitive_in_both_start_paths_and_restart`, `layout_policy::translation_restart_keeps_conversion_and_direct_input_works`, `menu::control_c_closes_application` |
| Обновление с выбранным Dictation | `exercise_modes::legacy_dictation_normalizes_without_writing_and_cancel_keeps_all_preferences` |
| Сохранение и повторный запуск | `exercise_modes::legacy_save_and_restart_remove_obsolete_fields_and_preserve_preferences`, `settings::saved_quick_start_translation_is_used`, `exercise_modes::new_settings_save_only_supported_defaults` |
| Отмена без перезаписи старого файла | `exercise_modes::legacy_dictation_normalizes_without_writing_and_cancel_keeps_all_preferences`, `settings::escape_discards_unsaved_settings` |
| Неподдерживаемые поля не повреждают оставшиеся настройки | `exercise_modes::legacy_dictation_normalizes_without_writing_and_cancel_keeps_all_preferences` (шесть JSON-типов), `exercise_modes::legacy_save_and_restart_remove_obsolete_fields_and_preserve_preferences` |
| Отсутствующий синтезатор не влияет на работу | `spanish_translation::spanish_translation_is_available_without_audio_dependency`, `dataset_profiles::translation_types_complete_pairs_without_audio_and_retains_case`; обычная свежая сборка/установка и PTY без аудионастройки |
| Старое окружение не запускает процесс | `exercise_modes::obsolete_tts_environment_never_starts_a_program` (меню, Settings, Typing, Translation, результаты и restart; исполняемый индикатор не вызван) |
| Испанский перевод сохраняет правила | `spanish_translation::spanish_translation_accepts_eng_ru_and_canonical_input_from_all_start_paths`, `spanish_translation::spanish_translation_supports_all_seven_diacritics_and_restart`, `spanish_translation::spanish_translation_enter_and_backspace_cross_pair_boundary`, `spanish_translation::spanish_translation_keeps_wrong_letters_as_errors`, `dataset_profiles::translation_types_complete_pairs_without_audio_and_retains_case` |
| Существующая база переживает два запуска | `dataset_profiles::every_discovered_descriptor_is_seeded_once_and_preserves_existing_data`, `dataset_profiles::five_profiles_route_settings_select_mode_quick_start_and_persist`, `spanish_settings::spanish_translation_setting_persists_and_unsaved_dataset_is_discarded` |
| Новый набор не требует аудио | `python3 scripts/check_dataset_build.py`; `dataset_profiles::five_profiles_route_settings_select_mode_quick_start_and_persist`, `dataset_profiles::descriptor_checks_handle_long_ids_targets_and_russian_translations` |
| Повреждённый выбранный набор | `spanish_catalog::empty_damaged_or_unknown_selected_catalog_cannot_start_spanish_translation`, `spanish_catalog::corrupted_legacy_translation_json_is_an_error_instead_of_an_empty_success` |


### Итог обязательных проверок (5.1–5.3)

- `cargo fmt -- --check` и `git diff --check` — PASS.
- `cargo test --locked --test e2e` — PASS, 50/50 (64,97 s). После дополнения неизвестного выбранного набора `cargo test --locked --test e2e spanish_catalog::` — PASS, 2/2, затем финальный `cargo test --locked --all-features --all-targets` — PASS: unit 12/12, architecture 16/16, dataset_format 2/2, e2e 50/50 (терминальная часть 62,85 s). Проверки выполнялись с обычной параллельностью, изолированными каталогами WPM_DATA_DIR, без исключённых тестов.
- Список `cargo test --locked --test e2e -- --list` содержит 50 полных имён; таблица выше сверена с ним. E2E-приёмка выполнена на Unix/macOS; Windows здесь не проверялась и успешным терминальным результатом не объявляется.
- `openspec validate remove-dictation --strict` и строгая повторная валидация `add-spanish-translation-and-dictation`, `add-spanish-cafe-beginner-set`, `data-driven-exercise-datasets`, `document-repository-architecture`, `modularize-application-boundaries`, `organize-integration-test-suites` — PASS. Проверены актуальные deltas: будущая синхронизация не вводит audio-корень, диктантные разрешения, UI или аудиообязательства. Исторические результаты и отменённые непроверенные аудиозадачи сохранены.
- Поиск по src, README, docs, scripts, .github и AGENTS.md не обнаружил продукционного аудиокода или действующих инструкций настройки синтезатора: в src остаётся только сравнение старой строки `dictation` при загрузке настроек. Упоминания WPM_TTS_BIN и старых полей в E2E являются отрицательными проверками/проверками совместимости. README соответствует двум режимам и объясняет миграцию Quick Start.
- Snapshot исходников перед заключительным прогоном и после него совпал; после прогона изменён только этот отчёт tasks.md. Нерешённых вопросов реализации нет. Синхронизация основных спецификаций, архивирование и коммит этим исполнителем не выполнялись.

### Дополнительная сверка документации каталога (3.2, 16 ноября 2026)

- В `resources/ru_es_catalog.md` текущее описание использования исправлено на Translation; отменены действующие указания отдельной приёмки слышимости, описание проверки приведено к текстовому упражнению. Путь сценариев обновлён на `tests/e2e/dataset_profiles.rs`. Дата подготовки, происхождение, таблицы корпуса и исторические результаты сохранены. Это последующая правка только документации; исходники и наборы JSON после полного прогона не менялись.
- `git diff --check`, строгая валидация `remove-dictation` и сверка ссылок/отсутствия действующих аудиообязательств в документе — PASS. Повторный запуск Rust-тестов для этой правки текста не требуется.
