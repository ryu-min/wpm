#!/usr/bin/env python3
"""Download a new snapshot on request, or reproduce vocabulary extraction offline."""
import argparse
from datetime import date
import hashlib
import html
import json
from pathlib import Path
import re
from urllib.request import urlopen

SOURCE = Path(__file__).resolve().parent.parent / 'resources/course/sources'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--download', action='store_true', help='replace the three source HTML snapshots before extraction')
    args = parser.parse_args()
    metadata = json.loads((SOURCE / 'metadata.json').read_text())
    if args.download:
        downloaded = {}
        for item in metadata['files']:
            with urlopen(item['url'], timeout=30) as response:
                downloaded[item['path']] = response.read()
        for name, data in downloaded.items():
            (SOURCE / name).write_bytes(data)
        for item in metadata['files']:
            item['sha256'] = hashlib.sha256(downloaded[item['path']]).hexdigest()
        metadata['accessed'] = date.today().isoformat()
        metadata['vocabulary_version'] = 'requires review after download'
        metadata['structure_version'] = 'requires review after download'
        (SOURCE / 'metadata.json').write_text(json.dumps(metadata, ensure_ascii=False, indent=2) + '\n')
    topics = []
    raw = (SOURCE / 'duome.html').read_text()
    for n, part in enumerate(re.split('class="path-section-delimiter"', raw)[1:], 1):
        name = html.unescape(re.search('class="small-label">([^<]+)', part)[1])
        title = html.unescape(re.sub('<[^>]+>', '', part[:part.index('</li>')])).strip('> ').strip()
        title = re.sub(r'^\d+\s*' + re.escape(name) + r'\s*', '', title)
        pairs = re.findall('class="_blue  wA">(.*?)</span><span class="cCCC wT">(.*?)</span>', part)
        rows = [{'id': f'source-{n:03}-{j:03}', 'form': html.unescape(a), 'translation': html.unescape(b).removeprefix(' - ').strip()} for j, (a,b) in enumerate(pairs, 1)]
        topics.append({'id': f'topic-{n:03}', 'name': name, 'title': title, 'rows': rows})
    assert topics and all(t['rows'] for t in topics), 'empty vocabulary extraction'
    (SOURCE / 'vocabulary.json').write_text(json.dumps(topics, ensure_ascii=False, indent=2) + '\n')
    print(f"Extracted {len(topics)} topics and {sum(len(t['rows']) for t in topics)} source rows")


if __name__ == '__main__':
    main()
