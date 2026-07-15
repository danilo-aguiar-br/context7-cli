// SPDX-License-Identifier: MIT OR Apache-2.0
//! Platform-specific initialisation (Windows UTF-8 + ANSI).
//! Platform-specific initialization for console encoding and ANSI support.
//!
//! On Windows, configures UTF-8 code page (65001) and enables
//! virtual terminal processing for ANSI escape sequences.
//! On other platforms, this is a no-op.

/// Initialises platform-specific console settings.
///
/// Must be called as the FIRST action in `main()`, before any I/O.
pub fn init_platform() {
    #[cfg(windows)]
    configure_console_utf8();

    #[cfg(windows)]
    enable_ansi_windows();
}

#[cfg(windows)]
fn configure_console_utf8() {
    use windows_sys::Win32::System::Console::{SetConsoleCP, SetConsoleOutputCP};
    // SAFETY: Windows API functions are idempotent and only set the console codepage.
    // No concurrency at this point — single calls at the start of main() before any thread.
    unsafe {
        SetConsoleOutputCP(65001); // CP_UTF8
        SetConsoleCP(65001);
    }
}

#[cfg(windows)]
fn enable_ansi_windows() {
    use colored::control;
    // colored v2 on Windows Terminal / PowerShell 7+ auto-detects ANSI.
    // For legacy cmd.exe without VirtualTerminalProcessing: force via set_virtual_terminal.
    // If it fails (very old cmd.exe), disable colours to avoid raw escape sequences.
    if control::set_virtual_terminal(true).is_err() {
        control::set_override(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that `init_platform` does not panic on any platform.
    ///
    /// On Linux and macOS the function is no-op; on Windows it configures UTF-8 and ANSI.
    /// The test ensures the call is safe regardless of the operating system.
    #[test]
    fn test_init_platform_does_not_panic() {
        // Must complete without panic on Linux, macOS, and Windows.
        init_platform();
    }

    /// Verifies that `init_platform` is a safe no-op on non-Windows platforms.
    ///
    /// On Linux and macOS the function does not execute any instructions — it just returns.
    /// The test confirms that multiple consecutive calls are safe (idempotent).
    #[cfg(not(windows))]
    #[test]
    fn test_init_platform_is_noop_on_non_windows() {
        // Multiple calls must be idempotent and not cause side effects.
        init_platform();
        init_platform();
        init_platform();
    }
}
