fn main() -> std::process::ExitCode {
    match kinesin::cli::parse(std::env::args_os().skip(1)).and_then(kinesin::cli::execute) {
        Ok(code) => std::process::ExitCode::from(code),
        Err(error) => {
            eprintln!("Kinesin: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
