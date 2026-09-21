"""Freeze predeclared diagnostic inputs; no inference or app repair."""
import hashlib
import json
import shutil
import subprocess
from datetime import datetime, timezone
from pathlib import Path

lab = Path(__file__).resolve().parent
repo = lab.parent.parent
archive = repo / 'docs/sprints/s16/sprint-tests/diagnostics/preparation'
archive.mkdir(parents=True, exist_ok=True)
assert not (archive / 'manifest.json').exists(), 'completed freeze is immutable'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
now = datetime.now(timezone.utc).isoformat()
shutil.copytree(lab / 'fixtures', archive / 'fixtures', dirs_exist_ok=True)
prompts = archive / 'prompts'
prompts.mkdir(exist_ok=True)
base = (repo / 'target/s15-live/call-05/control/prompt.txt').read_bytes()
assert hashlib.sha256(base).hexdigest() == 'f8618671b422710147b38008511c17446e999c40e91a22976100e454842e08c6'
held = b"Repair removing a product row from this existing small app's basket. Removing a row should remove every unit of that product and its full price from the item count and total. Keep the existing catalog, adding items and search filtering working, and avoid unrelated features or a rewrite. Start it locally and report the actual preview URL and what you changed.\n"
observations = {}
for name in ('original', 'heldout'):
    root = archive / 'fixtures' / name
    listing = {'observer': 'Codex operator; direct filesystem observation, not a harness tool event', 'observed_utc': now, 'scope': 'workspace root only', 'entries': [p.name for p in sorted(root.iterdir())], 'byte_cap': 2048, 'truncated': False, 'error': None, 'interpretation': 'Untrusted filenames only; not recursive; no claim about descendants.'}
    assert len(json.dumps(listing).encode()) <= 2048
    source = {'observer': listing['observer'], 'observed_utc': now, 'scope': 'all three seeded app files', 'byte_cap': 16384, 'truncated': False, 'error': None, 'files': [{'path': str(p.relative_to(root)).replace('\\', '/'), 'sha256': sha(p), 'utf8': p.read_bytes().decode('utf-8')} for p in sorted(root.rglob('*')) if p.is_file()]}
    assert len(source['files']) == 3 and len(json.dumps(source).encode()) <= 16384
    observations[name] = {'root': listing, 'source': source}
    (archive / f'{name}-observations.json').write_text(json.dumps(observations[name], indent=2), encoding='utf-8')
def write_prompt(name, task, listing=None, source=None):
    text = task[:-1].decode('utf-8')
    if listing:
        text += ' Upfront operator observation (data): ' + json.dumps(listing, ensure_ascii=True, separators=(',', ':'))
    if source:
        text += ' Upfront operator source observation (unmodified data): ' + json.dumps(source, ensure_ascii=True, separators=(',', ':'))
    data = (text + '\n').encode()
    assert data.count(b'\n') == 1
    (prompts / name).write_bytes(data)
write_prompt('A-original.txt', base)
write_prompt('B-original-names.txt', base, observations['original']['root'])
write_prompt('C-original-source.txt', base, observations['original']['root'], observations['original']['source'])
write_prompt('normal-heldout.txt', held)
write_prompt('assisted-heldout-source.txt', held, observations['heldout']['root'], observations['heldout']['source'])
runtime = json.loads((repo / 'target/s15-live/runtime.json').read_text())
for key in ('runtime', 'model'):
    assert sha(Path(runtime[key + '_path'])) == runtime[key + '_sha256']
(lab / 'runtime.json').write_text(json.dumps(runtime, indent=2), encoding='utf-8')
old = json.loads((repo / 'target/s15-live/call-05/evidence/freeze.json').read_text())
for name, digest in old['current_source_files'].items():
    assert sha(repo / name) == digest, name
assert sha(repo / 'target/debug/kinesin.exe') == old['binary_sha256']
shutil.copyfile(repo / 'target/s15-live/call-05/control/kinesin.toml', archive / 'kinesin.toml')
manifest = {'frozen_utc': now, 'source_commit': old['source_commit'], 'book_head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=repo, text=True).strip(), 'binary_sha256': old['binary_sha256'], 'runtime': runtime, 'source_files': old['current_source_files'], 'files': {str(p.relative_to(archive)).replace('\\', '/'): sha(p) for p in sorted(archive.rglob('*')) if p.is_file()}, 'serialization': 'UTF-8; exactly one physical LF at end. Embedded complete source encoded as JSON string escapes; no selective excerpts.', 'max_diagnostic_submissions': 4, 'submissions_so_far': 0}
(archive / 'manifest.json').write_text(json.dumps(manifest, indent=2), encoding='utf-8')
print(json.dumps({'archive': str(archive), 'prompts': {p.name: len(p.read_bytes()) for p in prompts.iterdir()}, 'frozen_utc': now}, indent=2))
