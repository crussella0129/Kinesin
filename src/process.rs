//! Owned child processes. Taking pipes does not transfer lifecycle ownership.
//! Normal teardown terminates the process group/job and reaps the direct child;
//! dropping the owner synchronously signals termination as a fallback.

use std::io;
use std::process::ExitStatus;

use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};

/// Remove inherited credentials while retaining the platform startup variables.
pub fn scrub_environment(command: &mut Command) {
    command.env_clear();
    if let Some(path) = std::env::var_os("PATH") {
        command.env("PATH", path);
    }
    #[cfg(windows)]
    for key in ["SystemRoot", "SystemDrive", "PATHEXT", "TEMP", "TMP"] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
}

pub struct OwnedProcess {
    child: Child,
    #[cfg(unix)]
    group: libc::pid_t,
    #[cfg(windows)]
    job: windows::Job,
    settled: bool,
}

impl OwnedProcess {
    /// The caller owns executable authorization, stdio, and any sandbox policy.
    /// Unix group creation precedes every pre_exec sandbox callback.
    pub fn spawn(command: &mut Command) -> io::Result<Self> {
        command.kill_on_drop(true);
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.as_std_mut().process_group(0);
            let child = command.spawn()?;
            let group = child
                .id()
                .and_then(|id| libc::pid_t::try_from(id).ok())
                .ok_or_else(|| io::Error::other("child process identifier unavailable"))?;
            Ok(Self {
                child,
                group,
                settled: false,
            })
        }
        #[cfg(windows)]
        {
            use windows_sys::Win32::System::Threading::CREATE_SUSPENDED;
            let job = windows::Job::new()?;
            command.creation_flags(CREATE_SUSPENDED);
            let child = command.spawn()?;
            // Failure drops both owners: the suspended direct child and any
            // successfully assigned job members are terminated before resuming.
            job.assign_and_resume(&child)?;
            Ok(Self {
                child,
                job,
                settled: false,
            })
        }
    }

    pub fn take_stdin(&mut self) -> Option<ChildStdin> {
        self.child.stdin.take()
    }

    pub fn take_stdout(&mut self) -> Option<ChildStdout> {
        self.child.stdout.take()
    }

    pub fn take_stderr(&mut self) -> Option<ChildStderr> {
        self.child.stderr.take()
    }

    /// Observe leader exit without releasing Unix process-group identity. The
    /// unreaped leader reserves its PID until `terminate_and_wait` kills the
    /// group; reaping here could let cleanup signal an unrelated reused PGID.
    /// This wait is cancellation-safe and creates no detached blocking waiter.
    pub async fn wait_leader(&mut self) -> io::Result<ExitStatus> {
        #[cfg(unix)]
        {
            if self.settled {
                return self.child.wait().await;
            }
            // Match Child::wait: close any stdin still owned by the child so an
            // otherwise-finished process cannot wait forever for our EOF.
            self.child.stdin.take();
            // Register before observing the child, so an exit between the
            // observation and recv remains buffered in the signal stream.
            let mut changed =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::child())?;
            loop {
                if let Some(status) = self.peek_leader_status()? {
                    return Ok(status);
                }
                changed
                    .recv()
                    .await
                    .ok_or_else(|| io::Error::other("child signal stream closed"))?;
            }
        }
        #[cfg(windows)]
        self.child.wait().await
    }

    #[cfg(unix)]
    fn peek_leader_status(&self) -> io::Result<Option<ExitStatus>> {
        use std::os::unix::process::ExitStatusExt;

        // SAFETY: siginfo_t is a C output structure whose all-zero representation
        // is valid; clearing it also makes a no-change WNOHANG result portable.
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        // SAFETY: group is our direct child's still-owned positive PID. WNOWAIT
        // observes exit without consuming the status owned by Tokio's Child.
        if unsafe {
            libc::waitid(
                libc::P_PID,
                self.group as libc::id_t,
                &mut info,
                libc::WEXITED | libc::WNOWAIT | libc::WNOHANG,
            )
        } == -1
        {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: waitid initialized the SIGCHLD fields, or the structure is
        // still zeroed because no exit was available with WNOHANG.
        if unsafe { info.si_pid() } == 0 {
            return Ok(None);
        }
        // SAFETY: a nonzero si_pid from WEXITED identifies child exit status.
        let status = unsafe { info.si_status() };
        let raw = match info.si_code {
            libc::CLD_EXITED => status << 8,
            libc::CLD_KILLED => status,
            libc::CLD_DUMPED => status | 0x80,
            _ => return Err(io::Error::other("unexpected child exit notification")),
        };
        Ok(Some(ExitStatus::from_raw(raw)))
    }

    fn terminate(&mut self) -> io::Result<()> {
        #[cfg(unix)]
        {
            // SAFETY: group is the positive PID of our child, established as a
            // fresh process group at spawn; it is retained only through cleanup.
            if unsafe { libc::killpg(self.group, libc::SIGKILL) } == -1 {
                let error = io::Error::last_os_error();
                if error.raw_os_error() != Some(libc::ESRCH) {
                    return Err(error);
                }
            }
            Ok(())
        }
        #[cfg(windows)]
        self.job.terminate()
    }

    /// Stop group/job members and reap the direct child before normal release.
    /// Unix orphan zombies belong to the OS reaper; trusted servers must not
    /// deliberately escape their process group. Sandboxed commands deny escape.
    pub async fn terminate_and_wait(&mut self) -> io::Result<()> {
        if self.settled {
            return Ok(());
        }
        self.terminate()?;
        self.child.wait().await?;
        #[cfg(windows)]
        self.job.wait_empty().await?;
        self.settled = true;
        Ok(())
    }
}

impl Drop for OwnedProcess {
    fn drop(&mut self) {
        if !self.settled {
            let _ = self.terminate();
            // Tokio retains the direct child's reaping responsibility after
            // drop. On Windows Job's close also enforces kill-on-close.
            let _ = self.child.start_kill();
        }
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;
    use std::time::Duration;

    #[tokio::test]
    async fn leader_observation_retains_pid_until_group_cleanup() {
        let mut command = Command::new("/bin/false");
        let mut process = OwnedProcess::spawn(&mut command).unwrap();
        let pid = process.group;
        let status = tokio::time::timeout(Duration::from_secs(5), process.wait_leader())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(status.code(), Some(1));

        // Independently inspect the kernel's child record at the public API
        // boundary: a retained zombie reserves the PID even when no descendant
        // survives. The former Child::wait implementation returned ECHILD here.
        // SAFETY: zero is a valid siginfo_t representation; waitid fills it.
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        // SAFETY: pid belongs to this test's child; WNOWAIT does not reap it.
        assert_eq!(
            unsafe {
                libc::waitid(
                    libc::P_PID,
                    pid as libc::id_t,
                    &mut info,
                    libc::WEXITED | libc::WNOWAIT | libc::WNOHANG,
                )
            },
            0,
            "leader was reaped before its process group was terminated"
        );
        // SAFETY: successful WEXITED waitid initialized child status fields.
        assert_eq!(unsafe { info.si_pid() }, pid);
        assert_eq!(process.wait_leader().await.unwrap().code(), Some(1));

        process.terminate_and_wait().await.unwrap();
        assert!(process.settled);
        // The Tokio child caches the real status after final reaping.
        assert_eq!(process.wait_leader().await.unwrap().code(), Some(1));
        // SAFETY: this nonblocking query only observes our former child PID.
        assert_eq!(
            unsafe {
                libc::waitid(
                    libc::P_PID,
                    pid as libc::id_t,
                    &mut info,
                    libc::WEXITED | libc::WNOWAIT | libc::WNOHANG,
                )
            },
            -1
        );
        assert_eq!(
            io::Error::last_os_error().raw_os_error(),
            Some(libc::ECHILD)
        );
    }

    #[tokio::test]
    async fn cancelled_observation_preserves_cleanup_and_signal_status() {
        let mut command = Command::new("/bin/sleep");
        command.arg("60");
        let mut process = OwnedProcess::spawn(&mut command).unwrap();
        assert!(
            tokio::time::timeout(Duration::from_millis(20), process.wait_leader())
                .await
                .is_err()
        );
        // SAFETY: the still-owned, unreaped child reserves this group's PID.
        assert_eq!(unsafe { libc::killpg(process.group, libc::SIGTERM) }, 0);
        let status = tokio::time::timeout(Duration::from_secs(5), process.wait_leader())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(status.signal(), Some(libc::SIGTERM));
        assert_eq!(status.code(), None);
        process.terminate_and_wait().await.unwrap();
        assert_eq!(process.wait_leader().await.unwrap(), status);
    }
}

#[cfg(windows)]
mod windows {
    use super::*;
    use std::mem::size_of;
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    use std::ptr::{null, null_mut};
    use std::time::Duration;
    use windows_sys::Win32::Foundation::{ERROR_NO_MORE_FILES, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
    };
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOBOBJECT_BASIC_ACCOUNTING_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JobObjectBasicAccountingInformation, JobObjectExtendedLimitInformation,
        QueryInformationJobObject, SetInformationJobObject, TerminateJobObject,
    };
    use windows_sys::Win32::System::Threading::{OpenThread, ResumeThread, THREAD_SUSPEND_RESUME};

    pub(super) struct Job(OwnedHandle);

    impl Job {
        pub(super) fn new() -> io::Result<Self> {
            // SAFETY: null security/name arguments request an unnamed,
            // non-inheritable job; success returns a uniquely owned handle.
            let handle = unsafe { CreateJobObjectW(null(), null()) };
            if handle.is_null() {
                return Err(io::Error::last_os_error());
            }
            // SAFETY: CreateJobObjectW returned a valid new owned handle.
            let job = Self(unsafe { OwnedHandle::from_raw_handle(handle) });
            let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            // SAFETY: live job, matching aligned initialized structure and size.
            if unsafe {
                SetInformationJobObject(
                    job.0.as_raw_handle(),
                    JobObjectExtendedLimitInformation,
                    (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                    size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(job)
        }

        pub(super) fn assign_and_resume(&self, child: &Child) -> io::Result<()> {
            let process = child
                .raw_handle()
                .ok_or_else(|| io::Error::other("child handle unavailable"))?;
            let id = child
                .id()
                .ok_or_else(|| io::Error::other("child identifier unavailable"))?;
            // SAFETY: both handles are live; child was created suspended and
            // cannot spawn descendants before assignment succeeds.
            if unsafe { AssignProcessToJobObject(self.0.as_raw_handle(), process) } == 0 {
                return Err(io::Error::last_os_error());
            }
            // SAFETY: snapshot flags request thread metadata only.
            let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
            if snapshot == INVALID_HANDLE_VALUE {
                return Err(io::Error::last_os_error());
            }
            // SAFETY: successful snapshot is a newly owned handle.
            let snapshot = unsafe { OwnedHandle::from_raw_handle(snapshot) };
            let mut entry = THREADENTRY32 {
                dwSize: size_of::<THREADENTRY32>() as u32,
                ..Default::default()
            };
            // SAFETY: valid snapshot and initialized size/output structure.
            let mut found = unsafe { Thread32First(snapshot.as_raw_handle(), &mut entry) };
            let mut resumed = false;
            while found != 0 {
                if entry.th32OwnerProcessID == id {
                    // SAFETY: this thread belongs to the live suspended child;
                    // request only the right required to resume it.
                    let thread =
                        unsafe { OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID) };
                    if thread.is_null() {
                        return Err(io::Error::last_os_error());
                    }
                    // SAFETY: OpenThread returned a new owned handle.
                    let thread = unsafe { OwnedHandle::from_raw_handle(thread) };
                    // SAFETY: live thread handle with resume access.
                    if unsafe { ResumeThread(thread.as_raw_handle()) } == u32::MAX {
                        return Err(io::Error::last_os_error());
                    }
                    resumed = true;
                }
                entry.dwSize = size_of::<THREADENTRY32>() as u32;
                // SAFETY: same live snapshot and initialized output structure.
                found = unsafe { Thread32Next(snapshot.as_raw_handle(), &mut entry) };
            }
            let error = io::Error::last_os_error();
            if error.raw_os_error() != Some(ERROR_NO_MORE_FILES as i32) {
                return Err(error);
            }
            if !resumed {
                return Err(io::Error::other("suspended child thread unavailable"));
            }
            Ok(())
        }

        pub(super) fn terminate(&self) -> io::Result<()> {
            // SAFETY: live exclusively owned job handle; this affects only its
            // assigned process tree, and repeated termination is permitted.
            if unsafe { TerminateJobObject(self.0.as_raw_handle(), 1) } == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        }

        pub(super) async fn wait_empty(&self) -> io::Result<()> {
            loop {
                let mut info = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
                // SAFETY: matching initialized output structure and length,
                // valid job handle; optional returned-length pointer is null.
                if unsafe {
                    QueryInformationJobObject(
                        self.0.as_raw_handle(),
                        JobObjectBasicAccountingInformation,
                        (&mut info as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                        size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                        null_mut(),
                    )
                } == 0
                {
                    return Err(io::Error::last_os_error());
                }
                if info.ActiveProcesses == 0 {
                    return Ok(());
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }
}
