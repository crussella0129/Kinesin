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
            "--emit-invalid-utf8" => {
                let n: usize = value(&mut i).parse().unwrap_or(0);
                let mut out = std::io::stdout();
                let _ = out.write_all(&vec![0xff; n]);
                let _ = out.flush();
            }
            "--write-file" => {
                let path = value(&mut i);
                let content = value(&mut i);
                if std::fs::write(path, content).is_err() {
                    exit_code = 23;
                }
            }
            "--wait-file" => {
                let path = value(&mut i);
                let deadline = std::time::Instant::now() + Duration::from_secs(5);
                while std::fs::metadata(&path).is_err() {
                    if std::time::Instant::now() >= deadline {
                        exit_code = 24;
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
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
                // Re-exec ourselves as a grandchild that keeps writing.
                // It inherits the process group/job, so killing the
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
            #[cfg(target_os = "linux")]
            "--truncate-file" => {
                let path = std::ffi::CString::new(value(&mut i)).unwrap();
                // SAFETY: fixture-owned terminated path; mutation targets only
                // the explicit synthetic test path supplied by the test.
                let denied = unsafe { libc::truncate(path.as_ptr(), 0) } != 0;
                println!(
                    "{}",
                    if denied {
                        "TRUNCATE_DENIED"
                    } else {
                        "TRUNCATE_OK"
                    }
                );
                if denied {
                    exit_code = 25;
                }
            }
            "--rename-file" => {
                let from = value(&mut i);
                let to = value(&mut i);
                if std::fs::rename(from, to).is_err() {
                    exit_code = 26;
                }
            }
            #[cfg(target_os = "linux")]
            "--read-fd" => {
                let fd: i32 = value(&mut i).parse().unwrap();
                let mut byte = [0_u8; 1];
                // SAFETY: bounded initialized output and a numeric descriptor
                // explicitly supplied by the synthetic inheritance test.
                let read = unsafe { libc::pread(fd, byte.as_mut_ptr().cast(), 1, 0) };
                println!(
                    "{}",
                    if read > 0 {
                        "FD_READ_OK"
                    } else {
                        "FD_READ_DENIED"
                    }
                );
            }
            #[cfg(target_os = "linux")]
            "--probe-syscall" => {
                let name = value(&mut i);
                let syscall = match name.as_str() {
                    "uring-setup" => libc::SYS_io_uring_setup,
                    "uring-enter" => libc::SYS_io_uring_enter,
                    "uring-register" => libc::SYS_io_uring_register,
                    "setpgid" => libc::SYS_setpgid,
                    "setsid" => libc::SYS_setsid,
                    "unshare" => libc::SYS_unshare,
                    "setns" => libc::SYS_setns,
                    "clone3" => libc::SYS_clone3,
                    _ => panic!("unsupported syscall probe"),
                };
                // SAFETY: zero arguments are invalid for io_uring or request a
                // self group/session change; no foreign pointer is dereferenced.
                let result = unsafe {
                    libc::syscall(
                        syscall, 0_usize, 0_usize, 0_usize, 0_usize, 0_usize, 0_usize,
                    )
                };
                let expected = if name == "clone3" {
                    libc::ENOSYS
                } else {
                    libc::EPERM
                };
                let denied = result == -1
                    && std::io::Error::last_os_error().raw_os_error() == Some(expected);
                println!(
                    "{}",
                    if denied {
                        "SYSCALL_DENIED"
                    } else {
                        "SYSCALL_NOT_DENIED"
                    }
                );
                if !denied {
                    exit_code = 27;
                }
            }
            #[cfg(target_os = "linux")]
            "--probe-namespace-clone" => {
                // SAFETY: the filter must deny namespace clone before argument
                // validation; zero pointers do not name parent memory to write.
                let result = unsafe {
                    libc::syscall(
                        libc::SYS_clone,
                        libc::CLONE_NEWUSER | libc::SIGCHLD,
                        0_usize,
                        0_usize,
                        0_usize,
                        0_usize,
                    )
                };
                if result == 0 {
                    std::process::exit(29);
                }
                let denied = result == -1
                    && std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM);
                if result > 0 {
                    // SAFETY: this is the PID returned by our own unexpected
                    // successful clone; reap it so a failed probe leaks nothing.
                    unsafe {
                        libc::waitpid(result as libc::pid_t, std::ptr::null_mut(), 0);
                    }
                }
                println!(
                    "{}",
                    if denied {
                        "SYSCALL_DENIED"
                    } else {
                        "SYSCALL_NOT_DENIED"
                    }
                );
                if !denied {
                    exit_code = 29;
                }
            }
            "--spawn-thread" => {
                std::thread::spawn(|| println!("THREAD_OK")).join().unwrap();
            }
            #[cfg(target_os = "linux")]
            "--spawn-session-probe" => {
                // A grandchild is not a group leader, so setsid would succeed
                // without the filter; testing it in the leader is insufficient.
                let status = std::process::Command::new(std::env::current_exe().unwrap())
                    .args(["--probe-syscall", "setsid"])
                    .status()
                    .unwrap();
                exit_code = status.code().unwrap_or(28);
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
