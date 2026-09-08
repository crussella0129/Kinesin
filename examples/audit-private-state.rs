//! Read-only operator audit. This example never changes permissions.

use kinesin::private_state::{PrivateStatePolicy, validate_private_tree};
use std::path::PathBuf;

fn main() -> std::process::ExitCode {
    let result = audit();
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(reason) => {
            eprintln!("Private state rejected: {reason}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn audit() -> Result<(), String> {
    let mut args = std::env::args_os().skip(1);
    let path = PathBuf::from(
        args.next()
            .ok_or("usage: audit-private-state ABSOLUTE_PATH [--trust-sid OPERATOR_SID]")?,
    );
    let mut trusted = Vec::new();
    while let Some(flag) = args.next() {
        if flag != "--trust-sid" {
            return Err("unknown audit option".into());
        }
        trusted.push(
            args.next()
                .ok_or("missing operator SID")?
                .into_string()
                .map_err(|_| "SID must be text")?,
        );
    }
    let policy = PrivateStatePolicy::current(&trusted)?;
    let report = validate_private_tree(&path, &policy)?;
    println!(
        "Private descriptor checks passed for {} entries; root owner {}.",
        report.checked_entries, report.owner
    );
    println!(
        "This audit does not verify trusted ancestors, service identity isolation, or network policy."
    );
    Ok(())
}
