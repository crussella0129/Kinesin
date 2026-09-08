PRAGMA foreign_keys = ON;

CREATE TABLE runs (
    run_id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL,
    workspace_id TEXT NOT NULL,
    model_profile_id TEXT NOT NULL,
    task_mode TEXT NOT NULL CHECK (task_mode IN ('freeform', 'checked')),
    task_profile_id TEXT,
    task_profile_version TEXT,
    task_spec_sha256 TEXT,
    checker_id TEXT,
    checker_version TEXT,
    capture TEXT NOT NULL CHECK (capture IN ('metadata', 'replay')),
    phase TEXT NOT NULL CHECK (phase IN (
        'queued', 'running', 'cancelling', 'completed',
        'stopped', 'failed', 'cancelled', 'interrupted'
    )),
    acceptance_status TEXT NOT NULL CHECK (acceptance_status IN (
        'unchecked', 'pending', 'passed', 'failed', 'inconclusive'
    )),
    created_unix_ms INTEGER NOT NULL,
    policy_version TEXT NOT NULL,
    submission_sha256 TEXT NOT NULL,
    idempotency_key TEXT,
    terminal_reason TEXT,
    result_json TEXT,
    result_sha256 TEXT,
    acceptance_json TEXT,
    CHECK (
        (task_mode = 'freeform' AND acceptance_status = 'unchecked'
            AND task_profile_id IS NULL AND task_profile_version IS NULL
            AND task_spec_sha256 IS NULL
            AND checker_id IS NULL AND checker_version IS NULL)
        OR
        (task_mode = 'checked' AND acceptance_status != 'unchecked'
            AND task_profile_id IS NOT NULL AND task_profile_version IS NOT NULL
            AND task_spec_sha256 IS NOT NULL
            AND checker_id IS NOT NULL AND checker_version IS NOT NULL)
    ),
    CHECK (task_spec_sha256 IS NULL OR length(task_spec_sha256) = 64),
    CHECK (result_sha256 IS NULL OR length(result_sha256) = 64),
    CHECK (
        (result_json IS NULL AND result_sha256 IS NULL)
        OR (result_json IS NOT NULL AND result_sha256 IS NOT NULL)
    ),
    CHECK (
        phase IN ('queued', 'running', 'cancelling')
        OR (acceptance_status != 'pending' AND acceptance_json IS NOT NULL)
    ),
    CHECK (
        phase NOT IN ('queued', 'running', 'cancelling')
        OR acceptance_status IN ('unchecked', 'pending')
    ),
    CHECK (acceptance_status != 'passed' OR phase = 'completed'),
    CHECK (
        phase != 'completed'
        OR (result_json IS NOT NULL AND result_sha256 IS NOT NULL
            AND acceptance_json IS NOT NULL)
    ),
    UNIQUE (owner_id, idempotency_key)
);

CREATE TABLE events (
    run_id TEXT NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    seq INTEGER NOT NULL CHECK (seq >= 0),
    schema_version INTEGER NOT NULL,
    kind TEXT NOT NULL,
    elapsed_ms INTEGER NOT NULL CHECK (elapsed_ms >= 0),
    data_json TEXT NOT NULL,
    PRIMARY KEY (run_id, seq)
);

CREATE INDEX runs_owner_created
    ON runs(owner_id, created_unix_ms, run_id);

PRAGMA user_version = 2;
