//! What the launcher leaves behind when it panics. On Windows it has no console, so a crash at
//! start used to look like "it doesn't open": now the panic and its backtrace go to
//! `launcher-crash.txt` in the launcher's data folder, and when the main thread panics Windows
//! shows where that file is.
use horizon_launcher::backend;
use std::path::PathBuf;

pub const FILE: &str = "launcher-crash.txt";

pub fn path() -> PathBuf {
    backend::data_dir().join(FILE)
}

/// Writes every panic to [`path`] (the newest one replaces the last), then lets the previous hook
/// print it as before.
pub fn install() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let panic = info.to_string();
        let report = report(&panic);
        let file = path();
        let written = file
            .parent()
            .map(std::fs::create_dir_all)
            .transpose()
            .and_then(|_| std::fs::write(&file, &report))
            .is_ok();
        previous(info);
        // A worker thread's panic may be survived; the main thread's ends the launcher.
        if written && std::thread::current().name() == Some("main") {
            tell(&file, graphics_hint(&panic));
        }
    }));
}

/// What the player can do about a window that can't draw (#5, #8): GPUI draws with Direct3D 11
/// through DXGI 1.6, which Windows 10 has from version 1803 on, and old drivers refuse parts of it.
pub fn graphics_hint(panic: &str) -> Option<&'static str> {
    if panic.contains("Creating DXGI factory") {
        return Some(
            "This version of Windows is too old for the launcher's window: it needs Windows 10 \
             version 1803 or later (DXGI 1.6). Update Windows in Settings > Update & Security, \
             then open the launcher again.",
        );
    }
    (panic.contains("DirectX") || panic.contains("Direct3D")).then_some(
        "The graphics driver couldn't start Direct3D 11 for the launcher's window. Update the \
         graphics driver (from Intel, NVIDIA or AMD's site, not only Windows Update), then open \
         the launcher again.",
    )
}

fn report(panic: &str) -> String {
    let hint = graphics_hint(panic)
        .map(|h| format!("What to do: {h}\n\n"))
        .unwrap_or_default();
    format!(
        "Horizon Offline Launcher {} crashed\n\
         Time: {}\n\
         System: {} {}\n\n\
         {panic}\n\n\
         {hint}\
         Backtrace:\n{}\n\n\
         Please attach this file to an issue: \
         https://github.com/gerdaroot/Horizon-Offline-Launcher/issues/new?template=crash.yml\n",
        backend::VERSION,
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::backtrace::Backtrace::force_capture(),
    )
}

/// Windows: a message box naming the report, since nothing else shows that anything happened.
#[cfg(target_os = "windows")]
fn tell(file: &std::path::Path, hint: Option<&str>) {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBoxW(hwnd: isize, text: *const u16, caption: *const u16, kind: u32) -> i32;
    }
    let wide = |s: &str| s.encode_utf16().chain([0]).collect::<Vec<u16>>();
    let hint = hint.map(|h| format!("{h}\n\n")).unwrap_or_default();
    let text = wide(&format!(
        "Horizon Offline Launcher crashed.\n\n{hint}The report is in:\n{}\n\nPlease attach it to an \
         issue on github.com/gerdaroot/Horizon-Offline-Launcher.",
        file.display()
    ));
    let caption = wide("Horizon Offline Launcher");
    // MB_OK | MB_ICONERROR
    unsafe { MessageBoxW(0, text.as_ptr(), caption.as_ptr(), 0x10) };
}

#[cfg(not(target_os = "windows"))]
fn tell(_file: &std::path::Path, _hint: Option<&str>) {}

#[cfg(test)]
mod tests {
    #[test]
    fn the_report_names_the_panic_and_where_to_send_it() {
        let report = super::report("panicked at src/main.rs:1:1: no GPU");
        assert!(report.contains("no GPU"));
        assert!(report.contains("Backtrace:"));
        assert!(report.contains("issues/new?template=crash.yml"));
        assert!(!report.contains("What to do"));
    }

    /// #5 (Windows 10 1703) and #8 (an old Intel driver) said nothing a player could act on.
    #[test]
    fn directx_failures_say_what_to_update() {
        let old_windows = "failed to initialize DirectX: Creating DirectX devices: Creating DXGI \
                           factory: No such interface supported (0x80004002)";
        assert!(super::graphics_hint(old_windows).unwrap().contains("1803"));
        assert!(super::report(old_windows).contains("What to do: This version of Windows"));
        let old_driver =
            "window: Creating DirectX renderer\n\nCaused by:\n    0: Creating DirectX \
                          resources\n    1: El parámetro no es correcto. (0x80070057)";
        assert!(super::graphics_hint(old_driver)
            .unwrap()
            .contains("graphics driver"));
        assert!(super::graphics_hint("index out of bounds").is_none());
    }
}
