//! No-inference wire provenance for the approved Sprint 15 live comparison.
//! Loads the real profile and prepares bytes without opening a model or tool.

use std::io::Write;
use std::path::Path;

use kinesin::config::BoundedConfig;
use kinesin::policy::Submission;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 4 {
        return Err("usage: s15-prepare-request CONFIG PROMPT_FILE ADAPTER NEW_OUTPUT_FILE".into());
    }
    let adapter: u64 = args[2].parse()?;
    if !matches!(adapter, 4..=6) {
        return Err("diagnostic adapter must be 4, 5 or 6".into());
    }
    let config = BoundedConfig::load(Path::new(&args[0]))?;
    let authority = config.authorize_local(Submission::Freeform {
        workspace: "storefront".into(),
        model: "local".into(),
        prompt: std::fs::read_to_string(&args[1])?,
        continues: None,
        limits: None,
        capture: None,
    })?;
    let (mut state, _) = kinesin::core::initiate_with_context(
        authority.instructions().into(),
        None,
        None,
        authority.prompt().into(),
        true,
    )?;
    // Core 7 and 8 share the same initial WORK_INSTRUCTION_V7. This helper
    // captures only the first request; actual subsequent requests are journaled.
    state.enable_workflow_recovery_for_tools(
        &authority
            .workspace()
            .tools
            .iter()
            .map(|tool| tool.wire_name())
            .collect::<Vec<_>>(),
    )?;
    let mut options = kinesin::runner::options(&authority);
    options.structured_actions = adapter >= 5;
    let request = kinesin::model::prepare_with_version(state.messages(), &options, adapter)?;
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[3])?;
    output.write_all(request.bytes())?;
    println!("{} {}", request.sha256(), request.bytes().len());
    Ok(())
}
