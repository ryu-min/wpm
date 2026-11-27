# Keyboard Layout Conversion Specification

## Purpose

Allow terminal typing exercises to interpret Russian and English keys by keyboard position when useful, while letting users keep ordinary typing exercises layout sensitive by default.

## Requirements

### Requirement: Convert corresponding keys in Translation
The system SHALL enable Russian/English keyboard-position conversion in every Translation exercise, including starts from the Translation menu item, Quick Start configured for Translation, and Select Mode configured for Translation. Conversion SHALL cover every letter of the Russian alphabet, including `ё` and letters whose English-layout keys are punctuation, and SHALL work in both directions where the displayed target uses either layout. A direct input that already matches the displayed target SHALL remain correct.

#### Scenario: Mac semicolon key produces Russian zhe
- **WHEN** a Translation exercise displays `ж` as the next character and the terminal sends `;`
- **THEN** the input is accepted as `ж` and contributes to a correct result

#### Scenario: Every Russian letter is accepted from the English layout
- **WHEN** a Translation exercise displays each Russian letter as the target and the terminal sends the corresponding English-layout key for each letter
- **THEN** each input is accepted as correct, including `ё`, `х`, `ъ`, `ж`, `э`, `б`, and `ю`

#### Scenario: Cyrillic input maps to an English-layout target
- **WHEN** a Translation exercise displays an English-layout target and the terminal sends its corresponding Russian-layout character
- **THEN** each corresponding key is accepted as correct, including targets that are punctuation

#### Scenario: Correct layout remains usable
- **WHEN** a Translation exercise displays a character and the terminal sends that exact character
- **THEN** the character is accepted without a conversion error

### Requirement: Keep ordinary Typing layout sensitive by default
The system SHALL disable automatic Russian/English keyboard-position conversion by default in Typing exercises started through Quick Start or Select Mode. A key from the other layout SHALL count as incorrect unless the user has enabled conversion for Typing in Settings. This choice SHALL also apply when a Typing exercise is restarted.

#### Scenario: Default Typing rejects another-layout key
- **WHEN** a new or existing user starts a Typing exercise with the default setting and enters a corresponding key from the other layout
- **THEN** the result records that key as incorrect

#### Scenario: Default Typing accepts exact key
- **WHEN** a Typing exercise displays a character and the terminal sends that exact character
- **THEN** the character is accepted as correct

### Requirement: Allow persistent conversion default for Typing
Settings SHALL offer a visible on/off option for layout conversion in Typing exercises. The initial and missing-field value SHALL be off. Saving the option SHALL affect later Typing exercises from Quick Start and Select Mode, including restarts, across application launches. The option SHALL NOT disable conversion in Translation exercises.

#### Scenario: User enables conversion for Typing
- **WHEN** a user enables the Typing conversion option in Settings, saves it, and starts a Typing exercise
- **THEN** corresponding keys from the other layout count as correct

#### Scenario: Saved preference survives restart
- **WHEN** a user saves the Typing conversion option, exits, and relaunches the application
- **THEN** Settings shows the saved value and subsequent Typing exercises use it

#### Scenario: Existing settings file lacks the option
- **WHEN** the application loads a settings file created before this option existed
- **THEN** its other settings remain available and Typing conversion is off

#### Scenario: Translation ignores the Typing option
- **WHEN** the Typing conversion option is off and a user starts or restarts a Translation exercise
- **THEN** corresponding keys from the other layout still count as correct

### Requirement: Общие правила испанского ввода
Все режимы с испанским текстом SHALL применять одинаковые правила допустимости ввода. Помимо прямого совпадения, SHALL приниматься `n/ñ`, `a/á`, `e/é`, `i/í`, `o/ó`, `u/ú/ü` в соответствующем регистре и символы RU-раскладки с тех же физических клавиш, что базовые ENG-буквы. Точное совпадение SHALL иметь приоритет. Прочие буквы, иной регистр, пропуски и лишние символы SHALL оставаться ошибками. Канонически эквивалентные Unicode-представления SHALL давать одинаковую проверку. Эти правила SHALL NOT превращать фонетическую кириллическую запись слова в испанскую транслитерацию.

#### Scenario: Базовые английские буквы вместо испанских
- **WHEN** испанская цель содержит `ñ á é í ó ú ü`, а соответствующий ввод содержит `n a e i o u u`
- **THEN** каждый из этих вводов принимается как правильный

#### Scenario: Русские клавиши вместо испанских букв
- **WHEN** испанская цель содержит `ñ á é í ó ú ü`, а соответствующий ввод содержит `т ф у ш щ г г`
- **THEN** каждый из этих вводов принимается по тем же правилам, что базовые ENG-буквы

#### Scenario: Прямой ввод, регистр и Unicode
- **WHEN** пользователь вводит точные испанские символы либо канонически эквивалентную составную Unicode-форму в правильном регистре
- **THEN** ввод принимается без дополнительной замены

#### Scenario: Исправление составного символа
- **WHEN** последняя введённая испанская буква записана базовой буквой и составным диакритическим знаком, а пользователь нажимает Backspace
- **THEN** вся отображаемая буква удаляется за одно нажатие, без оставленного отдельного диакритического знака

#### Scenario: Неверная буква не исправляется автоматически
- **WHEN** пользователь вводит `e` при цели `á` либо `a` при цели `Á`
- **THEN** ввод считается ошибочным

#### Scenario: Позиции клавиш не являются транслитерацией
- **WHEN** целевое слово равно `hola`
- **THEN** `рщдф` принимается, а `хола` не принимается как правильное испанское слово

### Requirement: Проверка зависит от языка участка и момента отправки
Языковая политика SHALL определяться языком целевого участка или выбранного набора, а не названием режима. Испанские эквивалентности SHALL NOT распространяться на русские/английские участки. `Translation` SHALL проверять видимый образец во время печати.

#### Scenario: Упрощённый ввод сохраняет канонический образец
- **WHEN** для цели `año` пользователь вводит `ano` в `Translation`
- **THEN** ввод принимается, а видимый канонический образец остаётся `año`

#### Scenario: Испанское упрощение не затрагивает английский
- **WHEN** английская цель содержит `n`, а ввод содержит `ñ`
- **THEN** испанская эквивалентность не делает ввод правильным

### Requirement: Явное ограничение упрощённой проверки
Испанский Translation SHALL объяснять, что разрешённые замены не проверяют знание диакритики. Он SHALL принимать одинаковый упрощённый ввод для слов, которые становятся неразличимыми по этим правилам, сохраняя каноническое написание в видимом образце.

#### Scenario: Разные канонические слова имеют общий упрощённый ответ
- **WHEN** в образце Translation показано `si` или `sí`, а пользователь вводит `si`
- **THEN** ввод принимается в обоих случаях, а образец сохраняет именно выбранное каноническое слово

### Requirement: Завершение последней графемы при автоматическом результате
В `Typing` и `Translation` результат SHALL появляться автоматически при достижении длины упражнения без дополнительного Enter или задержки для Unicode-ввода. Если завершение произошло по длине до истечения лимита, последующие последовательные составные знаки последней графемы SHALL учитываться и после показа результата, пока пользователь не нажал другую клавишу. Знак SHALL приниматься в проверяемый текст только как продолжение последней существующей графемы без добавления новой графемы; правильность этого знака SHALL определяться общими правилами, и недопустимый для цели знак SHALL NOT отбрасываться. Точность и WPM SHALL пересчитываться при неизменном времени первоначального завершения. Завершение по таймеру SHALL NOT принимать дальнейший ввод. Прочие клавиши на Result SHALL сохранять прежние действия и закрывать возможность дополнения этой попытки; Backspace SHALL NOT открывать редактирование результата.

#### Scenario: Допустимый знак приходит после показа результата
- **WHEN** в обычном `Typing` с точной целью `café` пользователь вводит `cafe`, видит результат и следующим событием вводит U+0301 без промежуточного нажатия другой клавиши
- **THEN** результат обновляется до точности 100%, как при доставке `cafe` и U+0301 одним фрагментом; время завершения остаётся прежним

#### Scenario: Недопустимый знак не теряется на границе событий
- **WHEN** в испанском `Translation` с целью `кофе café` пользователь вводит `кофе cafe`, а затем отдельным событием U+0308 до другого нажатия клавиши
- **THEN** последняя буква считается ошибочной независимо от того, успел ли появиться Result между базовой буквой и составным знаком

#### Scenario: Другая клавиша завершает возможность дополнения
- **WHEN** после автоматического результата по длине пользователь нажимает стрелку или Backspace, а затем вводит составной знак
- **THEN** результат завершённой попытки не меняется; стрелка выполняет обычную навигацию, а Backspace не возвращает к редактированию

#### Scenario: Лимит времени завершает ввод окончательно
- **WHEN** упражнение закончилось по лимиту времени, после чего приходит составной знак
- **THEN** знак не меняет текст, точность, WPM и время завершённой попытки
