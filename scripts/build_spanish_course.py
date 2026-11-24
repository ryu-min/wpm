#!/usr/bin/env python3
"""Rebuild course resources from preserved sources and editorial decisions."""
import html
import json
import re
from pathlib import Path
import unicodedata

ROOT = Path(__file__).resolve().parent.parent
COURSE = ROOT / 'resources/course'
COUNTS = [10, 31, 30, 60, 50, 50, 36, 36]


def read(path):
    return json.loads(path.read_text())


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')


def key(value):
    return unicodedata.normalize('NFC', value).casefold()


def generate():
    source_html = (COURSE / 'sources/structure.html').read_text()
    introductions = re.findall(r'<b>Модуль (\d+) .*?\n<span class=c19>(.*?)</span>', source_html)
    assert [int(n) for n, _ in introductions] == list(range(1, 9)), 'invalid module introductions'
    titles = [re.split('[А-Яа-яЁё]', html.unescape(re.sub('<[^>]*>', '', text)), 1)[0].strip() for _, text in introductions]
    assert all(titles), 'empty module introduction'
    vocabulary = read(COURSE / 'sources/vocabulary.json')
    legacy = read(ROOT / 'resources/provenance/ru_es_duolingo_m1.json')['entries']
    legacy_ids = {r['section']: r['entry_id'].split(':')[0] for r in legacy if f"_s{r['section']:02}_" in r['entry_id']}
    decisions = read(COURSE / 'normalization.json')
    structure = []
    for line in (COURSE / 'sources/structure-details.html').read_text().splitlines():
        if line.startswith('<tr><td>') and 'span' not in line:
            title, debug = line.removeprefix('<tr><td>').split('<td>')
            structure.append({'title': html.unescape(title), 'debug_name': html.unescape(debug)})
    outputs = {}
    modules, evidence, owners = [], [], {}
    index = cursor = 0
    for number, count in enumerate(COUNTS, 1):
        module = {'id': f'module-{number}', 'title': titles[number - 1], 'topics': []}
        for position in range(count):
            source = vocabulary[index]
            width = 1 if index < 131 else 5
            block = structure[cursor:cursor + width]
            expected = [source['name']] if width == 1 else [f"{source['name']}, Lesson {n}" for n in range(1, 6)]
            assert [key(x['debug_name']) for x in block] == [key(x) for x in expected], source['id']
            topic = {'id': source['id'], 'title': source['title'], 'start': position * width + 1, 'end': (position + 1) * width, 'source_name': source['name'], 'source_sections': block, 'dataset_id': None, 'alternatives': {}}
            dataset_id = legacy_ids.get(index + 1, source['id'].replace('-', '_'))
            entries = []
            for row in source['rows']:
                decision = decisions[row['form']]
                target, ru = decision['target'], decision['ru']
                target_key = key(target)
                duplicate = target_key in owners
                if not duplicate:
                    entry_id = dataset_id + ':' + target
                    owners[target_key] = (topic['id'], entry_id, ru, target)
                    entries.append({'id': entry_id, 'ru': ru, 'target': target})
                    if decision.get('alternatives'):
                        topic['alternatives'][target] = decision['alternatives']
                owner, entry_id, owner_ru, owner_target = owners[target_key]
                evidence.append({**row, **decision, 'topic_id': topic['id'], 'owner_topic_id': owner, 'entry_id': entry_id, 'ru': owner_ru, 'target': owner_target, 'decision': 'repeat' if duplicate else 'included', 'source_url': f"https://duome.eu/vocabulary/ru/es/skills#s{index + 1}"})
            if entries:
                topic['dataset_id'] = dataset_id
                descriptor = {'schema_version': 1, 'id': dataset_id, 'description': topic['title'], 'profile': 'spanish-phrases', 'level': 'course', 'entries': entries}
                outputs[COURSE / 'datasets' / (dataset_id + '.json')] = descriptor
            module['topics'].append(topic)
            index += 1
            cursor += width
        modules.append(module)
    assert cursor == len(structure) == 991
    for path, descriptor in outputs.items():
        write(path, descriptor)
    for path in (COURSE / 'datasets').glob('*.json'):
        if path not in outputs:
            path.unlink()
    write(COURSE / 'manifest.json', {'schema_version': 1, 'modules': modules})
    write(COURSE / 'provenance.json', {'source_rows': evidence, 'unprocessed': [], 'rejected': []})
    coverage = []
    for module in modules:
        topic_ids = {t['id'] for t in module['topics']}
        coverage.append({'module_id': module['id'], 'topics': len(topic_ids), 'source_rows': sum(r['topic_id'] in topic_ids for r in evidence), 'canonical_units': sum(len(d['entries']) for path, d in outputs.items() if d['id'] in {t['dataset_id'] for t in module['topics']}), 'unprocessed': 0})
    write(COURSE / 'coverage.json', {'modules': coverage, 'legacy_first_module': {'source_rows': 208, 'canonical_units': 177}, 'total_source_rows': len(evidence), 'unprocessed': 0, 'linguistic_review': 'Canonical dictionary candidates and ambiguous homographs were checked. Independent expert editorial review remains pending.'})
    print(f'{len(evidence)} source rows, {len(owners)} canonical units, {index} topics, {cursor} sections')


if __name__ == '__main__':
    generate()
