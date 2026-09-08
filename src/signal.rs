//! Console cancellation without inheriting a launcher's Windows Ctrl+C ignore flag.

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
