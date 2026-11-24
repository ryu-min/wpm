#!/usr/bin/env python3
"""Validate the full course against independent preserved vocabulary and structure."""
import argparse
from copy import deepcopy
import hashlib
import html
import json
from pathlib import Path
import re
import unicodedata

ROOT = Path(__file__).resolve().parent.parent
COURSE = ROOT / 'resources/course'
COUNTS = [10,31,30,60,50,50,36,36]


def load(path):
    return json.loads(path.read_text())


def key(text):
    return unicodedata.normalize('NFC', text).casefold()


def validate(manifest=None, evidence=None):
    manifest = manifest or load(COURSE / 'manifest.json')
    evidence = evidence or load(COURSE / 'provenance.json')
    metadata = load(COURSE / 'sources/metadata.json')
    for item in metadata['files']:
        assert hashlib.sha256((COURSE / 'sources' / item['path']).read_bytes()).hexdigest() == item['sha256'], 'snapshot hash mismatch'
    source = load(COURSE / 'sources/vocabulary.json')
    raw = (COURSE / 'sources/duome.html').read_text()
    actual_source = []
    for n, part in enumerate(re.split('class="path-section-delimiter"', raw)[1:], 1):
        pairs = re.findall('class="_blue  wA">(.*?)</span><span class="cCCC wT">(.*?)</span>', part)
        for j, (form, translation) in enumerate(pairs, 1):
            actual_source.append((f'source-{n:03}-{j:03}',html.unescape(form),html.unescape(translation).removeprefix(' - ').strip()))
    saved_source = [(r['id'],r['form'],r['translation']) for t in source for r in t['rows']]
    assert actual_source == saved_source, 'source inventory mismatch'
    rows = evidence['source_rows']
    assert [(r['id'],r['form'],r['translation']) for r in rows] == saved_source, 'missing source row'
    assert not evidence['unprocessed'] and not evidence['rejected'], 'unprocessed/rejected rows'
    modules = manifest['modules']
    assert [len(m['topics']) for m in modules] == COUNTS, 'module boundary mismatch'
    structure_html = (COURSE / 'sources/structure.html').read_text()
    introductions = re.findall(r'<b>Модуль (\d+) .*?\n<span class=c19>(.*?)</span>', structure_html)
    assert [int(n) for n, _ in introductions] == list(range(1, 9)), 'module introduction order'
    for module, (_, text) in zip(modules, introductions):
        expected_title = re.split('[А-Яа-яЁё]', html.unescape(re.sub('<[^>]*>', '', text)), 1)[0].strip()
        assert module.get('title') == expected_title and expected_title, f'module {module.get("id", "unknown")}: introduction missing, empty, or differs from source'
    topics = [t for m in modules for t in m['topics']]
    assert len(topics) == 303
    ids = [t['dataset_id'] for t in topics if t['dataset_id']]
    assert len(ids) == len(set(ids)), 'duplicate dataset link'
    editorial = load(COURSE / 'normalization.json')
    for form, expected in load(COURSE / 'editorial-regressions.json').items():
        assert all(editorial[form][k] == v for k,v in expected.items()), 'editorial regression: ' + form
    owner = {}
    pairs = {}
    for r in rows:
        assert r['source_url'].startswith('https://duome.eu/vocabulary/ru/es/skills#s'), 'missing source'
        assert r['normalization'], 'missing normalization'
        source_topic = 'topic-' + r['id'].split('-')[1]
        assert r['topic_id'] == source_topic, 'source row assigned to wrong topic'
        assert key(r['target']) == key(editorial[r['form']]['target']), 'row normalization mismatch'
        assert r['ru'].count('(') == r['ru'].count(')'), 'truncated parenthesis in cue'
        first = owner.setdefault(key(r['target']), (r['topic_id'],r['entry_id']))
        assert first == (r['owner_topic_id'],r['entry_id']), 'wrong first owner'
        pair = (r['ru'],r['target'])
        assert r['entry_id'] not in pairs or pairs[r['entry_id']] == pair, 'conflicting pair'
        pairs[r['entry_id']] = pair
    actual = {}
    targets = set()
    cursor = 0
    details = []
    for line in (COURSE / 'sources/structure-details.html').read_text().splitlines():
        if line.startswith('<tr><td>') and 'span' not in line:
            details.append(html.unescape(line.split('<td>')[-1]))
    structure = (COURSE / 'sources/structure.html').read_text()
    structure_rows = [(int(a), int(b)) for a,b in re.findall(r'<span class=c1>([1-8])</span>.*?<span class=[^>]+>([0-9]+) ', structure)]
    assert len(structure_rows) == 991, 'incomplete independent structure'
    for module in modules:
        next_section = 1
        for t in module['topics']:
            assert t['start'] == next_section, 'range boundary mismatch'
            width = t['end'] - t['start'] + 1
            expected = [t['source_name']] if width == 1 else [f"{t['source_name']}, Lesson {i}" for i in range(1,6)]
            assert [key(x) for x in details[cursor:cursor+width]] == [key(x) for x in expected], 'lesson/name mismatch'
            assert [x['debug_name'] for x in t['source_sections']] == details[cursor:cursor+width], 'missing lesson'
            module_number = int(module['id'].split('-')[-1])
            assert structure_rows[cursor:cursor+width] == [(module_number, n) for n in range(t['start'],t['end']+1)], 'independent module boundary mismatch'
            cursor += width
            next_section = t['end'] + 1
            if t['dataset_id'] is None:
                continue
            d = load(COURSE / 'datasets' / (t['dataset_id']+'.json'))
            assert d['id'] == t['dataset_id'] and d['entries'], 'invalid descriptor link'
            for entry in d['entries']:
                same_cue = {e['target'] for e in d['entries'] if e['ru'] == entry['ru'] and e['target'] != entry['target']}
                assert same_cue <= set(t['alternatives'].get(entry['target'], [])), 'ambiguous cue without verified alternatives'
            for e in d['entries']:
                assert key(e['target']) not in targets, 'duplicate target'
                targets.add(key(e['target']))
                assert e['id'] not in actual, 'duplicate entry id'
                actual[e['id']] = (e['ru'],e['target'])
                allowed = {a for d in editorial.values() if d['target'] == e['target'] for a in d.get('alternatives',[])}
                assert set(t['alternatives'].get(e['target'],[])) <= allowed, 'unverified answer alternative'
    assert cursor == len(details) == 991, 'missing course section'
    assert actual == pairs, 'descriptor/evidence pair mismatch'
    legacy = load(ROOT / 'resources/provenance/ru_es_duolingo_m1.json')['entries']
    legacy_targets = {r['target'] for r in legacy}
    first_targets = {r['target'] for r in rows if r['topic_id'] in {t['id'] for t in modules[0]['topics']}}
    assert legacy_targets == first_targets and len(legacy) == 208, 'legacy first module loss'
    assert [t['source_name'] for t in topics] == [t['name'] for t in source], 'wrong topic name'
    return len(rows),len(actual)


def self_test():
    m = load(COURSE / 'manifest.json')
    p = load(COURSE / 'provenance.json')
    variants=[]
    q=deepcopy(p);q['source_rows'].pop();variants.append((m,q))
    q=deepcopy(p);q['source_rows'][0]['source_url']='';variants.append((m,q))
    q=deepcopy(p);q['source_rows'][0]['ru']='ошибка';variants.append((m,q))
    q=deepcopy(p);q['source_rows'][0]['owner_topic_id']='invalid';variants.append((m,q))
    q=deepcopy(m);q['modules'][4]['topics'][0]['source_sections'].pop();variants.append((q,p))
    q=deepcopy(m);q['modules'][0]['topics'][0]['source_name']='invalid';variants.append((q,p))
    q=deepcopy(m);q['modules'][0]['topics'][0]['end']=2;variants.append((q,p))
    q=deepcopy(m);q['modules'][0]['topics'][1]['dataset_id']=q['modules'][0]['topics'][0]['dataset_id'];variants.append((q,p))
    q=deepcopy(p);q['source_rows'][0]['topic_id']='topic-999';q['source_rows'][0]['owner_topic_id']='topic-999';variants.append((m,q))
    q=deepcopy(m);q['modules'][1]['id']='module-3';variants.append((q,p))
    q=deepcopy(m);q['modules'][0]['topics'][0]['alternatives']['agua']=['café'];variants.append((q,p))
    for kind in ['missing', 'empty', 'wrong']:
        q=deepcopy(m)
        if kind == 'missing': q['modules'][0].pop('title')
        else: q['modules'][0]['title'] = '' if kind == 'empty' else m['modules'][1]['title']
        variants.append((q,p))
    for a,b in variants:
        try:validate(a,b)
        except (AssertionError,KeyError):continue
        raise AssertionError('invalid mutation accepted')
    print(f'{len(variants)} invalid mutations rejected')


if __name__ == '__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--self-test',action='store_true');args=parser.parse_args()
    print('PASS: source rows / canonical pairs',validate())
    if args.self_test:self_test()
