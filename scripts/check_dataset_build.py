#!/usr/bin/env python3
"""Required Unix acceptance: new data file only, incremental embedding, standalone PTY."""
from contextlib import contextmanager
import fcntl
import hashlib
import json
import os
from pathlib import Path
import pty
import re
import select
import shutil
import sqlite3
import signal
import struct
import subprocess
import sys
import tempfile
import termios
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


def exercise(binary, empty_cwd, descriptor):
    with tempfile.TemporaryDirectory(prefix='wpm-dataset-data-') as directory:
        data = Path(directory)
        settings = dict(quick_start_time=15, quick_start_wordset='en_1000',
                        translation_set=descriptor['id'], translation_time=60)
        (data / 'settings.json').write_text(json.dumps(settings))
        env = os.environ.copy()
        env.update(TERM='xterm-256color', WPM_BIN=str(binary), WPM_DATA_DIR=str(data))
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
            wait(descriptor['id'])
            key(b'\r')
            pair = descriptor['entries'][0]
            wait(pair['target'])
            key((pair['ru'] + ' ' + pair['target']).encode())
            wait('Accuracy: 100%')
            key(b'\x1b'); key(b'\x1b')
            process.communicate(timeout=5)
            assert process.returncode == 0
        finally:
            if process.poll() is None:
                process.kill(); process.communicate()


@contextmanager
def course_terminal(binary, cwd, data):
    master, slave = pty.openpty()
    columns = 80
    fcntl.ioctl(master, termios.TIOCSWINSZ, struct.pack('HHHH', 24, columns, 0, 0))
    env = dict(os.environ, TERM='xterm-256color', WPM_ES_DATA_DIR=str(data))
    proc = subprocess.Popen([str(binary)], cwd=cwd, env=env, stdin=slave, stdout=slave, stderr=slave)
    os.close(slave)
    output = bytearray()
    def resize(rows, cols):
        nonlocal columns
        columns = cols
        fcntl.ioctl(master, termios.TIOCSWINSZ, struct.pack('HHHH', rows, cols, 0, 0))
        proc.send_signal(signal.SIGWINCH)

    def wait_for(label, centered=False):
        deadline = time.monotonic() + 8
        while True:
            lines = screen(output, columns).splitlines()
            for line in lines:
                if centered:
                    if line.strip() == label and abs(2 * line.index(label) + len(label) - columns) <= 1:
                        return lines
                elif label in line:
                    return lines
            if proc.poll() is not None or time.monotonic() > deadline:
                raise AssertionError(f'{label!r} missing from course screen: {screen(output, columns)}')
            if select.select([master], [], [], .05)[0]:
                output.extend(os.read(master, 65536))
    try:
        yield lambda key: os.write(master, key), wait_for, resize
        os.write(master, b'\x1b')
        assert proc.wait(timeout=5) == 0
    finally:
        if proc.poll() is None:
            proc.kill()
            proc.wait()
        os.close(master)


def course_home(binary, cwd, data, modules, first_title, empty_reason=None):
    """Check an offline standalone course, all module selectors and live resize."""
    with course_terminal(binary, cwd, data) as (key, wait_for, resize):
        wait_for(first_title)
        for index, module in enumerate(modules, 1):
            if index > 1:
                key(b'\x1b[C')
            label = f"{index}. {module['title']}"
            if len(label) > 76:
                label = label[:75] + '…'
            wait_for(f'< {label} >', centered=True)
            if index == 1 and empty_reason:
                key(b'\r')
                wait_for(empty_reason)
        resize(10, 80)
        wait_for('Увеличьте окно терминала')
        resize(24, 80)
        wait_for(f'< {label} >', centered=True)


def exam_resize(binary, cwd, data):
    """Keep the cursor and full submitted answer across real window changes."""
    data.mkdir()
    choice = dict(module='module-5', topics={'module-5': 'topic-138'}, mode='Exam')
    (data / 'course-settings.json').write_text(json.dumps(choice))
    answer = '1234567890' * 40 + 'fin'
    with course_terminal(binary, cwd, data) as (key, wait_for, resize):
        wait_for('Расскажи о медицине')
        key(b'\r')
        wait_for('_', centered=True)
        key(answer.encode())
        for width in (80, 40, 120):
            if width != 80:
                resize(24, width)
            tail = (answer + '_')[(len(answer) // (width - 4)) * (width - 4):]
            wait_for(tail, centered=True)
            if width == 40:
                key(b'\x1b[5~' * 3)
                wait_for('По-русски', centered=True)
                key(b'\x1b[6~' * 3)
                wait_for(tail, centered=True)
                key(b'\x1b[5~' * 3)
                wait_for('По-русски', centered=True)
                key(b'\x7f')
                wait_for(tail[:-2] + '_', centered=True)
                key(b'n')
                wait_for(tail, centered=True)
        resize(10, 40)
        wait_for('Увеличьте окно терминала')
        resize(24, 120)
        wait_for(tail, centered=True)
        key(b'\r\r')
        last = wait_for('3 из 3')
        targets = {'анализ крови': 'análisis de sangre', 'заболевать': 'enfermarse', 'сгореть': 'quemarse'}
        last_target = next(targets[row.strip()] for row in last if row.strip() in targets)
        key(b'\r')
        result = wait_for('Посмотреть ошибки')
        assert not any('Верный ответ' in row for row in result)
        key(b'\r')
        wait_for('Ошибки экзамена')
        full_line = 'Ваш ответ: ' + answer
        for start in range(0, len(full_line), 116):
            wait_for(full_line[start:start + 116])
        resize(24, 40)
        wait_for('Ошибки экзамена', centered=True)
        key(b'\x1b[6~' * 6)
        wait_for('Верный ответ: ' + last_target)
        key(b'\x1b')
        wait_for('Правильно 0 из 3')
        key(b'\x1b')
        wait_for('Enter начать')
        key(b'\r')
        wait_for('_', centered=True)
        resize(10, 40)
        wait_for('Увеличьте окно терминала')
        key(b'\x1b')
        time.sleep(.15)
        resize(24, 80)
        wait_for('Enter начать')


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
        manifest_path = project / 'resources/course/manifest.json'
        saved_manifest = manifest_path.read_bytes()
        manifest = json.loads(saved_manifest)
        first_topic = manifest['modules'][0]['topics'][0]
        course_home(temp / 'target/debug/wpm-es', empty, temp / 'course-settings', manifest['modules'], first_topic['title'])
        exam_resize(temp / 'target/debug/wpm-es', empty, temp / 'exam-resize')
        module = manifest['modules'][0]
        title = module.pop('title')
        manifest_path.write_text(json.dumps(manifest, ensure_ascii=False))
        build(False, ['manifest.json', 'missing field', 'title'])
        module['title'] = ' '
        manifest_path.write_text(json.dumps(manifest, ensure_ascii=False))
        build(False, ['invalid module', module['id']])
        module['title'] = title
        first_topic['title'] = 'Проверка обновления курса'
        manifest_path.write_text(json.dumps(manifest, ensure_ascii=False))
        build()
        course_home(temp / 'target/debug/wpm-es', empty, temp / 'updated-course', manifest['modules'], first_topic['title'])
        manifest_path.write_bytes(saved_manifest)
        topic_path = project / 'resources/course/datasets' / (first_topic['dataset_id'] + '.json')
        saved_topic = topic_path.read_bytes()
        topic_path.unlink()
        build(False, ['missing or repeated dataset', first_topic['dataset_id']])
        # Preserve a selectable topic when every source word belongs to an earlier one.
        first_topic['dataset_id'] = None
        first_topic['alternatives'] = {}
        for unavailable, reason in [(False, 'Новых слов нет'), (True, 'Словарь недоступен')]:
            first_topic['unavailable'] = unavailable
            manifest_path.write_text(json.dumps(manifest, ensure_ascii=False))
            build()
            course_home(temp / 'target/debug/wpm-es', empty, temp / reason, manifest['modules'], first_topic['title'], reason)
        manifest_path.write_bytes(saved_manifest)
        topic_path.write_bytes(saved_topic)
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
        exercise(binary, empty, descriptor)
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
        print('PASS: incremental dataset and course builds; standalone Translation and Spanish course PTYs; resize, empty/unavailable topics, and invalid descriptors')


if __name__ == '__main__':
    main()
