//! OS integration primitives owned by the host layer.
//!
//! These replace the Tauri plugins (`opener`, `notification`). Kept here (not in
//! the SDK) because they are platform-specific shell concerns. The Qt bridge
//! calls these; the SDK never does.

use std::process::Command;

/// Open a URL/path with the platform default handler.
///
/// Callers MUST pass a vetted `http(s)` URL — see `HostLinks::open` which
/// allowlists schemes before delegating here.
pub fn open_external(target: &str) {
    let _ = launch(target);
}

#[cfg(target_os = "macos")]
fn launch(target: &str) -> std::io::Result<()> {
    Command::new("open").arg(target).spawn().map(|_| ())
}

#[cfg(target_os = "windows")]
fn launch(target: &str) -> std::io::Result<()> {
    // `cmd /c start "" <url>` handles URLs without spawning a console window.
    Command::new("cmd")
        .args(["/C", "start", "", target])
        .spawn()
        .map(|_| ())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn launch(target: &str) -> std::io::Result<()> {
    Command::new("xdg-open").arg(target).spawn().map(|_| ())
}

/// True for schemes we are willing to hand to the OS opener.
pub fn is_safe_external(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}
