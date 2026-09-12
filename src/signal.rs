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

/// Register the listener before enabling signal delivery. Some Windows launchers
/// create children with Ctrl+C ignored; adding a handler does not clear that
/// separate, inherited attribute.
pub async fn ctrl_c() -> std::io::Result<()> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;

        let mut listener = tokio::signal::windows::ctrl_c()?;
        // SAFETY: a null handler changes only this process's ignore attribute.
        // Tokio owns the registered callback before delivery is re-enabled.
        if unsafe { SetConsoleCtrlHandler(None, 0) } == 0 {
            return Err(std::io::Error::last_os_error());
        }
        listener.recv().await;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        tokio::signal::ctrl_c().await
    }
}
