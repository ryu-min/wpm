# Tasks

## 1. Зависимость спецификаций и общая сессия

- [x] 1.1 Синхронизировать реализованную capability `spanish-course-app` из `add-spanish-course-app` в основные спецификации и закрепить порядок применения изменений. Проверить наличие исходных требований перед новой дельтой, строгую валидацию и отсутствие возможности позднего возврата устаревших требований из прежнего change.
- [x] 1.2 Подключить формирование текста выбранных пар и общую `TypingSession` к тренировке курса, сохранив полный раздел, порядок и ошибочное подмножество. Проверить модельным тестом текст ru/пробел/es/перенос, языковые политики и отсутствие лимита времени. В этом тесте проверяется внутреннее соответствие политики каждой графеме, которое один снимок терминала не показывает.

## 2. Печать и результат тренировки

- [x] 2.1 Использовать общий виджет и обработчик Translation, обновление статистики и автозавершение в координаторе курса. Обновить E2E для обеих сторон пары, Enter только на границе, Backspace через перенос, цветов ошибки/исправления, курсора и перехода между тремя видимыми строками, включая ширину 40 и изменение размера. Проверить полный ввод и преждевременный Esc через `wpm-es`.
- [x] 2.2 Подключить WPM, точность графем и время к результату тренировки, повтор и возврат с сохранённым выбором. Проверить E2E автоматического завершения с ошибкой и без ошибки, сброса статистики при повторе, составной последней графемы разными событиями и неизменности финализированного времени.
- [x] 2.3 Перевести «Потренировать ошибки» на тот же путь набора пар. Проверить E2E экзамена с двумя ошибками, запуска и повторения тренировки только этих двух пар, затем возврата и полного повторного экзамена.
- [x] 2.4 Уточнить область правил сравнения и документацию режима. Проверить E2E политики раскладки/испанских вариантов и регистра в тренировке, строгой диакритики экзамена и сохранения настроек выбора. Обновить README с клавишами, обеими набираемыми сторонами, точностью печати, завершением и выбранными defaults.

## 3. Интеграционная приёмка

- [x] 3.1 Получить список `cargo test --locked --test e2e -- --list`, запустить новые сценарии по точным именам, существующие группы `spanish_translation::` и `spanish_course::`, затем полный `cargo test --locked --test e2e` и `cargo test --locked --all-features --all-targets`. Зафиксировать результаты обоих бинарников и Unix PTY-платформу.
- [x] 3.2 Выполнить `cargo fmt --all -- --check`, `git diff --check`, `openspec validate align-spanish-training-with-translation --strict` и локальный запуск `cargo run --locked --bin wpm-es`. Проверить каждую изменённую requirement по соответствующему результату теста, после завершения реализации синхронизировать новую дельту поверх исходной capability.


## Acceptance evidence

On macOS arm64 Unix PTY, 22 library tests passed, including shared translation formatting and per-grapheme language policies. The seven existing spanish_translation scenarios passed. All 16 spanish_course scenarios passed as a group. Six new scenarios passed by exact name. Parent additionally exercised live terminal resize, Backspace across a pair boundary and full section completion through the built binary.

The preceding add-spanish-course-app change was archived after applying its specs. Retiring the obsolete capability required retire_capabilities in that change's metadata. The new training delta was then applied to the durable spanish-course-app capability.

Two simultaneous full-suite runs exposed snapshot readiness races under excessive combined test load. New tests now wait on their relevant rendered state and terminal attributes. The final full runs use the suite's ordinary internal parallelism and execute sequentially.

Final sequential runs both passed: all-features/all-targets includes 22 library, 18 architecture, 2 dataset-format and 76 terminal scenarios (64.37s). The separate terminal suite passed all 76 scenarios (see final log). Formatting, diff and strict OpenSpec validation passed. Parent verified local launch, installation and live resize.
