#!/usr/bin/env python3
"""Required Unix acceptance: new data file only, incremental embedding, standalone PTY."""
import hashlib
import json
import os
from pathlib import Path
import re
import select
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]


def snapshot(root):
    return {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in root.rglob('*') if p.is_file()}


def screen(output, columns=80):
    # Ratatui emits cursor-positioned diffs, so searching a raw transcript can
    # mistake an old frame for the current one or miss words separated by CSI.
    grid = [[' '] * columns for _ in range(24)]
    row = col = 0
    tokens = re.split(r'(\x1b\[[0-?]*[ -/]*[@-~])', output.decode(errors='replace'))
    for token in tokens:
        if token.startswith('\x1b['):
            final = token[-1]
            params = token[2:-1]
            if final in ('H', 'f'):
                parts = [int(x or '1') for x in params.split(';')]
                row = parts[0] - 1
                col = (parts[1] if len(parts)>1 else 1) - 1
            elif final == 'J' and params in ('2', '3'):
                grid = [[' '] * columns for _ in range(24)]
            continue
        for char in token:
            if char == '\r': col = 0
            elif char == '\n': row += 1
            elif char.isprintable():
                if 0 <= row < 24 and 0 <= col < columns: grid[row][col] = char
                col += 1
    return '\n'.join(''.join(line) for line in grid)


def exercise(binary, empty_cwd, descriptor, mode):
    with tempfile.TemporaryDirectory(prefix='wpm-dataset-data-') as directory:
        data = Path(directory)
        settings = dict(quick_start_time=15, quick_start_wordset='en_1000',
                        translation_set=descriptor['id'], translation_time=60,
                        dictation_set=descriptor['id'], dictation_length=20)
        (data / 'settings.json').write_text(json.dumps(settings))
        fake = data / 'speech'
        fake.write_text('#!/usr/bin/env python3\nimport sys\n'
                        'if "--voices=es" in sys.argv: print("5 es Spanish")\n'
                        'elif "--stdin" in sys.argv: sys.stdin.read()\n')
        fake.chmod(0o755)
        env = os.environ.copy()
        env.update(TERM='xterm-256color', WPM_BIN=str(binary), WPM_DATA_DIR=str(data), WPM_TTS_BIN=str(fake))
        columns = max(80, 2 * (len(descriptor['id']) + len(descriptor['entries'][0]['ru']) + len(descriptor['entries'][0]['target']) + 20))
        command = ['script', '-q']
        shell = f'stty rows 24 cols {columns}; exec "$WPM_BIN"'
        if sys.platform == 'darwin':
            command += ['/dev/null', 'sh', '-c', shell]
        else:
            command += ['-c', shell, '/dev/null']
        process = subprocess.Popen(command, cwd=empty_cwd, env=env, stdin=subprocess.PIPE,
                                   stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        output = bytearray()

        def wait(label):
            deadline = time.monotonic() + 8
            while label not in screen(output, columns):
                if time.monotonic() > deadline or process.poll() is not None:
                    raise AssertionError(f'missing {label}: {output.decode(errors="replace")}')
                if select.select([process.stdout], [], [], .05)[0]:
                    output.extend(os.read(process.stdout.fileno(), 65536))

        def key(value):
            process.stdin.write(value)
            process.stdin.flush()
            time.sleep(.12)

        try:
            wait('> Quick Start')
            # Select Mode starts with descriptor defaults; its mode/list is metadata driven.
            key(b'\x1b[B'); key(b'\x1b[B'); key(b'\r'); key(b'\x1b[C')
            if mode == 'dictation': key(b'\x1b[C')
            wait(descriptor['id'])
            key(b'\r')
            pair = descriptor['entries'][0]
            if mode == 'dictation':
                wait('Audio ready')
                key(pair['target'].lower().encode()); key(b'\r')
                wait('Answer: ' + pair['target']); wait('Checked: 1  Correct: 1  Errors: 0')
                key(b'\r'); wait('Dictation result')
            else:
                wait(pair['target'])
                key((pair['ru'] + ' ' + pair['target']).encode())
                wait('Accuracy: 100%')
            key(b'\x1b'); key(b'\x1b')
            process.communicate(timeout=5)
            assert process.returncode == 0
        finally:
            if process.poll() is None:
                process.kill(); process.communicate()


def main():
    assert os.name == 'posix', 'PTY acceptance requires Unix'
    with tempfile.TemporaryDirectory(prefix='wpm-dataset-build-') as directory:
        temp = Path(directory)
        project = temp / 'project'
        shutil.copytree(ROOT, project, ignore=shutil.ignore_patterns('.git', 'target', '__pycache__'))
        env = os.environ.copy()
        env.pop('SDKROOT', None)
        env['CARGO_TARGET_DIR'] = str(temp / 'target')
        empty = temp / 'empty'
        empty.mkdir()

        def build(ok=True, labels=()):
            result = subprocess.run(['cargo', 'build', '--locked'], cwd=project, env=env,
                                    stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
            assert (result.returncode == 0) == ok, result.stdout
            for label in labels: assert label in result.stdout, result.stdout

        def seed_and_read(name):
            with tempfile.TemporaryDirectory(prefix='wpm-dataset-inspect-') as data:
                # Exercise startup through a PTY, with Escape, to inspect resulting persistent seed.
                run_env = env.copy()
                run_env.update(WPM_BIN=str(temp / 'target/debug/wpm'), WPM_DATA_DIR=data, TERM='xterm-256color')
                shell = 'stty rows 24 cols 80; exec "$WPM_BIN"'
                args = ['script', '-q', '/dev/null', 'sh', '-c', shell] if sys.platform == 'darwin' else ['script', '-q', '-c', shell, '/dev/null']
                process = subprocess.Popen(args, cwd=empty, env=run_env, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
                output = bytearray(); deadline = time.monotonic()+8
                try:
                    while b'> Quick Start' not in output:
                        assert time.monotonic()<deadline, output
                        if select.select([process.stdout],[],[],.05)[0]: output.extend(os.read(process.stdout.fileno(),65536))
                    process.communicate(b'\x1b',timeout=5); assert process.returncode==0
                finally:
                    if process.poll() is None: process.kill(); process.communicate()
                conn = sqlite3.connect(str(Path(data)/'wordset.db'))
                return conn.execute('SELECT es FROM vocabulary_entries WHERE set_name=? ORDER BY id',(name,)).fetchall()

        build()
        before = snapshot(project)
        name = 'practice_' + 'long_' * 20 + 'new'
        original = 'Quisiera pedir un cafe con leche sin azucar y dos vasos de agua para mis amigos'
        russian = 'Я хотел бы заказать кофе с молоком без сахара и два стакана воды для моих друзей за соседним столиком'
        descriptor = dict(schema_version=1, id=name, description='Long data build acceptance',
                          profile='spanish-phrases', entries=[dict(id=name + ':one', ru=russian, target=original)])
        path = project / 'resources/datasets' / (name + '.json')
        path.write_text(json.dumps(descriptor,ensure_ascii=False))
        after = snapshot(project)
        assert set(after)-set(before) == {str(path.relative_to(project))}
        assert all(after[k] == v for k,v in before.items()), 'Adding data modified existing files'
        build()
        binary = temp / 'target/debug/wpm'
        for mode in ['translation', 'dictation']: exercise(binary, empty, descriptor, mode)
        assert seed_and_read(name) == [(original,)]
        descriptor['entries'][0]['target'] = 'Buenos dias'
        path.write_text(json.dumps(descriptor,ensure_ascii=False)); build()
        assert seed_and_read(name) == [('Buenos dias',)]
        path.unlink(); build(); assert seed_and_read(name) == []
        for change, labels in [({'profile':'unsupported'}, [path.name,'unsupported']),
                               ({'entries':[dict(id=name + ':one',ru='привет',target='')]}, [path.name,'entry 1'])]:
            bad = dict(descriptor,**change); path.write_text(json.dumps(bad)); build(False,labels); path.unlink()
        # Collision with an existing migrated descriptor names both JSON sources.
        conflict = dict(descriptor,id='ru_en_a1')
        collision = path.with_name('conflicting.json'); collision.write_text(json.dumps(conflict))
        build(False,['conflicting.json','duplicate dataset id','resources/datasets/ru_en_a1.json']); collision.unlink()
        path.write_text(json.dumps(descriptor)); other=path.with_name('other.json'); other.write_text(json.dumps(descriptor))
        build(False,['other.json',path.name,'duplicate dataset id']); other.unlink(); path.unlink()
        build()
        assert snapshot(project)==before, 'Acceptance left modified project files'
        print('PASS: JSON-only incremental add/change/delete; standalone PTY both modes; metadata/entry/collision diagnostics')


if __name__ == '__main__':
    main()
