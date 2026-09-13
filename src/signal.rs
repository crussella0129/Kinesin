//! Console cancellation without inheriting a launcher's Windows Ctrl+C ignore flag.

/// First-use prompts run before the controller and asynchronous signal listener.
/// Restore the operating system's default cancellation behavior during that
/// stage, when there is no live run or journal requiring settlement.
pub fn enable_setup_ctrl_c() -> std::io::Result<()> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
        // SAFETY: this changes only the current process's inherited ignore flag.
        if unsafe { SetConsoleCtrlHandler(None, 0) } == 0 {
            return Err(std::io::Error::last_os_error());
        }
    }
    Ok(())
}

/// A listener registered before any owned process or durable work is started.
/// Creating an asynchronous task alone does not guarantee its first poll occurs
/// before the process spawn, so registration is deliberately synchronous.
pub struct CtrlC {
    #[cfg(windows)]
    listener: tokio::signal::windows::CtrlC,
    #[cfg(unix)]
    listener: tokio::signal::unix::Signal,
}

/// Call within a Tokio runtime before starting owned work. Some Windows
/// launchers create children with Ctrl+C ignored; adding a handler does not
/// clear that separate inherited attribute, so restore delivery after ownership
/// of the listener is established.
pub fn ctrl_c_listener() -> std::io::Result<CtrlC> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;

        let listener = tokio::signal::windows::ctrl_c()?;
        // SAFETY: a null handler changes only this process's ignore attribute.
        // Tokio owns the registered callback before delivery is re-enabled.
        if unsafe { SetConsoleCtrlHandler(None, 0) } == 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(CtrlC { listener })
    }
    #[cfg(unix)]
    {
        let listener = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
        Ok(CtrlC { listener })
    }
}

impl CtrlC {
    pub async fn recv(&mut self) -> std::io::Result<()> {
        self.listener
            .recv()
            .await
            .ok_or_else(|| std::io::Error::other("Ctrl+C signal stream closed"))
    }
}

/// Compatibility wrapper for callers whose effects start after this is polled.
pub async fn ctrl_c() -> std::io::Result<()> {
    ctrl_c_listener()?.recv().await
}
