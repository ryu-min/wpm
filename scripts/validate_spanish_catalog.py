#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Проверка встроенного ES-каталога; не является лингвистической экспертизой."""
import argparse
import json
from pathlib import Path
import re
import sys
import unicodedata

LEVELS = ('a1', 'a2', 'b1', 'b2', 'c1', 'c2')
SPANISH_WORD = re.compile(r'[a-záéíóúüñ]+\Z')
PLACEHOLDER = re.compile(r'\b(?:todo|tbd|placeholder|заглушка|перевод\s+\d+)\b', re.I)
FOLD = str.maketrans('áéíóúüñ', 'aeiouun')


def validate(resources):
    errors, counts = [], {}
    ids, words, folded = {}, {}, {}
    for level in LEVELS:
        set_id = 'ru_es_' + level
        path = resources / (set_id + '.json')
        try:
            rows = json.loads(path.read_text(encoding='utf-8'))
        except (OSError, UnicodeError, json.JSONDecodeError) as exc:
            errors.append(f'{path.name}: {exc}')
            continue
        if not isinstance(rows, list):
            errors.append(f'{path.name}: ожидается массив записей')
            continue
        counts[set_id] = len(rows)
        if not 900 <= len(rows) <= 1100:
            errors.append(f'{set_id}: {len(rows)} записей вместо 900–1100')
        for index, row in enumerate(rows, 1):
            location = f'{set_id}[{index}]'
            if not isinstance(row, dict) or set(row) != {'id', 'es', 'ru'}:
                errors.append(f'{location}: обязательны ровно поля id/es/ru')
                continue
            if any(not isinstance(row[key], str) for key in ('id', 'es', 'ru')):
                errors.append(f'{location}: значения id/es/ru должны быть строками')
                continue
            lemma, translation, entry_id = row['es'], row['ru'], row['id']
            canonical = unicodedata.normalize('NFC', lemma)
            if canonical != lemma:
                errors.append(f'{location}: лемма не в NFC: {lemma!r}')
            if not SPANISH_WORD.fullmatch(lemma):
                errors.append(f'{location}: ожидается одно строчное испанское слово: {lemma!r}')
            if lemma != lemma.lower():
                errors.append(f'{location}: лемма должна быть строчной: {lemma!r}')
            if entry_id != set_id + ':' + lemma:
                errors.append(f'{location}: нестабильный id {entry_id!r}')
            if entry_id in ids:
                errors.append(f'{location}: повтор id из {ids[entry_id]}')
            ids[entry_id] = location
            key = canonical.casefold()
            if key in words:
                errors.append(f'{location}: повтор канонической леммы {lemma!r} из {words[key]}')
            words[key] = location
            if not translation.strip() or translation != translation.strip():
                errors.append(f'{location}: пустой перевод или внешние пробелы')
            if not re.search('[А-Яа-яЁё]', translation) or PLACEHOLDER.search(translation):
                errors.append(f'{location}: отсутствует русский перевод или обнаружена заглушка')
            if unicodedata.normalize('NFC', translation) != translation:
                errors.append(f'{location}: перевод не в NFC')
            folded.setdefault(canonical.translate(FOLD), set()).add(lemma)
    total = sum(counts.values())
    if not 5400 <= total <= 6600:
        errors.append(f'Всего {total} записей вместо 5400–6600')
    collisions = {key: sorted(value) for key, value in sorted(folded.items()) if len(value) > 1}
    return {'counts': counts, 'total': total, 'unique_lemmas': len(words),
            'simplified_collisions': collisions, 'errors': errors}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--resources-dir', type=Path,
                        default=Path(__file__).resolve().parent.parent / 'resources')
    parser.add_argument('--json', action='store_true', help='Машиночитаемый отчёт')
    args = parser.parse_args()
    report = validate(args.resources_dir)
    if args.json:
        print(json.dumps(report, ensure_ascii=False, indent=2))
    else:
        for name, count in report['counts'].items():
            print(f'{name}: {count}')
        print(f"Всего: {report['total']}; уникальных лемм: {report['unique_lemmas']}")
        print('Коллизии упрощённого ввода (допустимы, записи не объединяются):')
        for key, values in report['simplified_collisions'].items():
            print(f"  {key}: {', '.join(values)}")
        for error in report['errors']:
            print('ERROR: ' + error, file=sys.stderr)
        print('Проверка пройдена' if not report['errors'] else 'Проверка не пройдена')
    return 1 if report['errors'] else 0


if __name__ == '__main__':
    sys.exit(main())
