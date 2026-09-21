"""Keep the normal CLI session alive for browser inspection, with no added prompt."""
import hashlib
import json
import queue
import subprocess
import sys
import threading
import time
from datetime import datetime, timezone
from pathlib import Path

lab = Path(__file__).resolve().parent
number = int(sys.argv[1])
call = lab / f'call-{number:02d}'
evidence = call / 'evidence'
freeze = json.loads((evidence / 'freeze.json').read_text(encoding='utf-8'))
prompt = Path(freeze['prompt']).read_bytes()
assert prompt.count(b'\n') == 1 and prompt.endswith(b'\n')
assert hashlib.sha256(prompt).hexdigest() == freeze['prompt_sha256']
assert hashlib.sha256(Path(freeze['executable']).read_bytes()).hexdigest() == freeze['binary_sha256']
started = {'slot': number, 'started_utc': datetime.now(timezone.utc).isoformat(), 'consumed': True, 'operator_correction_messages': 0}
with (evidence / 'started.json').open('x', encoding='utf-8') as out:
    json.dump(started, out, indent=2)
(lab / 'active.json').write_text(json.dumps({'evidence': str(evidence), 'expected_first_sha256': freeze['expected_first_sha256']}), encoding='utf-8')
start = time.monotonic()
rows = []
lines = queue.Queue()
with (evidence / 'stderr.txt').open('wb') as stderr, (evidence / 'stdout.jsonl').open('wb') as stdout:
    process = subprocess.Popen([freeze['executable'], '--config', freeze['config'], '--json'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr, creationflags=getattr(subprocess, 'CREATE_NO_WINDOW', 0))
    (evidence / 'process.json').write_text(json.dumps({'pid': process.pid}), encoding='utf-8')
    def read_stdout():
        for line in process.stdout:
            stdout.write(line)
            stdout.flush()
            lines.put(line)
        lines.put(None)
    reader = threading.Thread(target=read_stdout, daemon=True)
    reader.start()
    process.stdin.write(prompt)
    process.stdin.flush()
    terminal = None
    while time.monotonic() - start < 290:
        try:
            line = lines.get(timeout=1)
        except queue.Empty:
            continue
        if line is None:
            break
        row = json.loads(line)
        if row.get('kind') == 'run':
            rows.append(row)
            terminal = {'slot': number, 'terminal_utc': datetime.now(timezone.utc).isoformat(), 'request_elapsed_s': time.monotonic()-start, 'run': row}
            (evidence / 'terminal.json').write_text(json.dumps(terminal, indent=2), encoding='utf-8')
            print(json.dumps(terminal), flush=True)
            break
        if row.get('kind') == 'error':
            print(json.dumps(row), flush=True)
            break
    if terminal:
        deadline = time.monotonic() + 1200
        while not (evidence / 'close-session').exists() and time.monotonic() < deadline:
            if process.poll() is not None:
                break
            time.sleep(0.5)
    process.stdin.close()
    try:
        code = process.wait(timeout=30)
    except subprocess.TimeoutExpired:
        process.terminate()
        code = process.wait(timeout=10)
    reader.join(timeout=5)
for row in rows:
    run_id = row.get('record', row)['run_id']
    result = subprocess.run([freeze['executable'], 'export', '--config', freeze['config'], '--run', run_id, '--output', str(evidence / 'capture.json')], capture_output=True, timeout=30)
    (evidence / 'export.jsonl').write_bytes(result.stdout)
    if result.returncode:
        (evidence / 'export-error.txt').write_bytes(result.stderr)
(evidence / 'finished.json').write_text(json.dumps({'slot': number, 'finished_utc': datetime.now(timezone.utc).isoformat(), 'session_elapsed_s': time.monotonic()-start, 'exit_code': code, 'runs': rows}, indent=2), encoding='utf-8')
print('Session closed; real capture exported.', flush=True)
