"""Prepare a selected frozen branch without model inference."""
import hashlib
import json
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

lab = Path(__file__).resolve().parent
repo = lab.parent.parent
preparation = repo / 'docs/sprints/s16/sprint-tests/diagnostics/preparation'
manifest = json.loads((preparation / 'manifest.json').read_text())
number, branch, fixture, prompt_name = sys.argv[1:]
call = lab / f'call-{int(number):02d}'
call.mkdir(exist_ok=False)
control, evidence = call / 'control', call / 'evidence'
control.mkdir()
evidence.mkdir()
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
for name, digest in manifest['files'].items():
    assert sha(preparation / name) == digest, name
for name, digest in manifest['source_files'].items():
    assert sha(repo / name) == digest, name
for key in ('runtime', 'model'):
    assert sha(Path(manifest['runtime'][key + '_path'])) == manifest['runtime'][key + '_sha256']
binary = repo / 'target/debug/kinesin.exe'
assert sha(binary) == manifest['binary_sha256']
shutil.copytree(preparation / 'fixtures' / fixture, call / 'app')
shutil.copyfile(preparation / 'kinesin.toml', control / 'kinesin.toml')
shutil.copyfile(preparation / 'prompts' / prompt_name, control / 'prompt.txt')
output = subprocess.check_output([str(repo / 'target/debug/examples/s15-prepare-request.exe'), str(control / 'kinesin.toml'), str(control / 'prompt.txt'), '6', str(evidence / 'first-request-expected.json')], cwd=repo, text=True).strip()
freeze = {'slot': int(number), 'branch': branch, 'fixture': fixture, 'frozen_utc': datetime.now(timezone.utc).isoformat(), 'adapter': 6, 'source_commit': manifest['source_commit'], 'executable': str(binary), 'binary_sha256': sha(binary), 'config': str(control / 'kinesin.toml'), 'config_sha256': sha(control / 'kinesin.toml'), 'prompt': str(control / 'prompt.txt'), 'prompt_sha256': sha(control / 'prompt.txt'), 'expected_first_sha256': sha(evidence / 'first-request-expected.json'), 'first_request_bytes': len((evidence / 'first-request-expected.json').read_bytes()), 'runtime': manifest['runtime'], 'seed': {str(p.relative_to(call / 'app')).replace('\\', '/'): sha(p) for p in sorted((call / 'app').rglob('*')) if p.is_file()}, 'current_source_files': manifest['source_files'], 'operator_assistance': branch in ('B', 'C', 'source-heldout'), 'correction_messages': 0}
(evidence / 'freeze.json').write_text(json.dumps(freeze, indent=2), encoding='utf-8')
print(output)
print(json.dumps({key: freeze[key] for key in ('slot', 'branch', 'prompt_sha256', 'expected_first_sha256', 'first_request_bytes')}))
