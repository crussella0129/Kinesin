//! A tiny, cross-platform, argv-driven fixture for the `run_command` tests.
//!
//! It never invokes a shell and its behavior is fully determined by its argv, so
//! the same binary drives the command-tool tests identically on Windows and Linux
//! (reached through `CARGO_BIN_EXE_cmd-fixture`). Directives are applied in order:
//! `--print <text>` writes text to stdout verbatim; `--emit-stdout <n>` /
//! `--emit-stderr <n>` write n bytes; `--print-cwd` writes the working directory;
//! `--print-env <name>` writes an env var's value (empty if unset); `--sleep-ms
//! <ms>` sleeps; `--spawn-grandchild <marker>` spawns a detached grandchild that
//! appends to the marker file forever (to prove whole-tree cleanup);
//! `--grandchild-loop <marker>` is that grandchild's internal mode;
//! `--read-file <path>` prints `READ_OK`/`READ_DENIED` and exits 21 if the read
//! was denied (filesystem-sandbox probe); `--open-socket` prints
//! `SOCKET_OK`/`SOCKET_DENIED` and exits 22 if socket creation was denied
//! (network-sandbox probe); and `--exit <code>` sets the process exit code
//! (default 0).

use std::io::Write;
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut exit_code = 0;
    let mut i = 0;
    while i < args.len() {
        let flag = args[i].as_str();
        let value = |idx: &mut usize| -> String {
            *idx += 1;
            args.get(*idx).cloned().unwrap_or_default()
        };
        match flag {
            "--print" => {
                let text = value(&mut i);
                let mut out = std::io::stdout();
                let _ = out.write_all(text.as_bytes());
                let _ = out.flush();
            }
            "--emit-stdout" => {
                let n: usize = value(&mut i).parse().unwrap_or(0);
                let mut out = std::io::stdout();
                let _ = out.write_all(&vec![b'a'; n]);
                let _ = out.flush();
            }
            "--emit-stderr" => {
                let n: usize = value(&mut i).parse().unwrap_or(0);
                let mut err = std::io::stderr();
                let _ = err.write_all(&vec![b'e'; n]);
                let _ = err.flush();
            }
            "--print-cwd" => {
                let cwd = std::env::current_dir().unwrap_or_default();
                let mut out = std::io::stdout();
                let _ = out.write_all(cwd.to_string_lossy().as_bytes());
                let _ = out.flush();
            }
            "--print-env" => {
                let name = value(&mut i);
                let val = std::env::var(&name).unwrap_or_default();
                let mut out = std::io::stdout();
                let _ = out.write_all(val.as_bytes());
                let _ = out.flush();
            }
            "--sleep-ms" => {
                let ms: u64 = value(&mut i).parse().unwrap_or(0);
                std::thread::sleep(Duration::from_millis(ms));
            }
            "--spawn-grandchild" => {
                let marker = value(&mut i);
                // Re-exec ourselves as a detached grandchild that keeps writing.
                // group_spawn places it in the same group/job, so killing the
                // group must stop it too.
                let exe = std::env::current_exe().expect("current exe");
                let _ = std::process::Command::new(exe)
                    .arg("--grandchild-loop")
                    .arg(&marker)
                    .spawn();
            }
            "--grandchild-loop" => {
                let marker = value(&mut i);
                loop {
                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&marker)
                    {
                        let _ = file.write_all(b"x");
                        let _ = file.flush();
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
            "--read-file" => {
                // Probe filesystem confinement: try to read a path, report the
                // outcome. Denied (e.g. by Landlock outside the workspace) => 21.
                let path = value(&mut i);
                let mut out = std::io::stdout();
                match std::fs::read(&path) {
                    Ok(_) => {
                        let _ = out.write_all(b"READ_OK");
                    }
                    Err(_) => {
                        let _ = out.write_all(b"READ_DENIED");
                        exit_code = 21;
                    }
                }
                let _ = out.flush();
            }
            "--open-socket" => {
                // Probe network confinement: creating a UDP socket exercises the
                // `socket` syscall. Denied (e.g. by seccomp) => 22.
                let mut out = std::io::stdout();
                match std::net::UdpSocket::bind("127.0.0.1:0") {
                    Ok(_) => {
                        let _ = out.write_all(b"SOCKET_OK");
                    }
                    Err(_) => {
                        let _ = out.write_all(b"SOCKET_DENIED");
                        exit_code = 22;
                    }
                }
                let _ = out.flush();
            }
            "--exit" => {
                exit_code = value(&mut i).parse().unwrap_or(0);
            }
            _ => {}
        }
        i += 1;
    }
    std::process::exit(exit_code);
}
