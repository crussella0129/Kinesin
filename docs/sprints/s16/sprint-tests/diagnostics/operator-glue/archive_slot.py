"""Archive settled diagnostic evidence and independently compare file bytes."""
import hashlib
import json
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path

lab = Path(__file__).resolve().parent
repo = lab.parent.parent
number = int(sys.argv[1])
call = lab / f'call-{number:02d}'
ev = call / 'evidence'
assert (ev / 'finished.json').exists()
freeze = json.loads((ev / 'freeze.json').read_text())
capture = json.loads((ev / 'capture.json').read_text())
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
final_files = {str(p.relative_to(call / 'app')).replace('\\', '/'): sha(p) for p in sorted((call / 'app').rglob('*')) if p.is_file()}
events = capture['events']
terminal = next(e for e in reversed(events) if e['kind'] == 'run_finished')
wire = sorted(ev.glob('wire-*-request.json'))
assert sha(wire[0]) == freeze['expected_first_sha256']
tools = [e for e in events if e['kind'].startswith('tool_')]
observation = {'slot': number, 'branch': freeze['branch'], 'observer': 'Codex operator; direct journal and filesystem observation', 'observed_utc': datetime.now(timezone.utc).isoformat(), 'first_wire_matches': True, 'counters': terminal['data']['counters'], 'final_files': final_files, 'changed_paths': sorted(p for p in set(final_files) | set(freeze['seed']) if final_files.get(p) != freeze['seed'].get(p)), 'tool_events': tools, 'terminal_reason': capture['run']['terminal_reason'], 'acceptance_status': capture['run']['acceptance_status'], 'candidate_claim': capture['run']['result'], 'request_elapsed_s': json.loads((ev / 'terminal.json').read_text())['request_elapsed_s'], 'session_closed': True, 'operator_assistance': freeze['operator_assistance'], 'corrective_messages': 0, 'manual_candidate_patches': 0, 'context_resets': 0, 'tool_forcing': 0}
if not tools:
    observation.update({'diagnostic_result': 'FAIL', 'reason': 'No model-selected tool operation, no changed files and no owned returned preview; defect behavior remains unqualified.', 'candidate_browser': 'not run: no actual preview created', 'preview_shutdown': 'not applicable: no preview created'})
(ev / 'observation.json').write_text(json.dumps(observation, indent=2), encoding='utf-8')
dest = repo / 'docs/sprints/s16/sprint-tests/diagnostics/evidence' / f'call-{number:02d}'
dest.mkdir(parents=True, exist_ok=False)
shutil.copytree(ev, dest / 'evidence')
shutil.copytree(call / 'app', dest / 'final-app')
shutil.copyfile(call / 'control/prompt.txt', dest / 'prompt.txt')
shutil.copyfile(call / 'control/kinesin.toml', dest / 'kinesin.toml')
print(json.dumps(observation, indent=2))
