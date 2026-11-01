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
        let report = report(&info.to_string());
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
            tell(&file);
        }
    }));
}

fn report(panic: &str) -> String {
    format!(
        "Horizon Offline Launcher {} crashed\n\
         Time: {}\n\
         System: {} {}\n\n\
         {panic}\n\n\
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
fn tell(file: &std::path::Path) {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBoxW(hwnd: isize, text: *const u16, caption: *const u16, kind: u32) -> i32;
    }
    let wide = |s: &str| s.encode_utf16().chain([0]).collect::<Vec<u16>>();
    let text = wide(&format!(
        "Horizon Offline Launcher crashed.\n\nThe report is in:\n{}\n\nPlease attach it to an issue on \
         github.com/gerdaroot/Horizon-Offline-Launcher.",
        file.display()
    ));
    let caption = wide("Horizon Offline Launcher");
    // MB_OK | MB_ICONERROR
    unsafe { MessageBoxW(0, text.as_ptr(), caption.as_ptr(), 0x10) };
}

#[cfg(not(target_os = "windows"))]
fn tell(_file: &std::path::Path) {}

#[cfg(test)]
mod tests {
    #[test]
    fn the_report_names_the_panic_and_where_to_send_it() {
        let report = super::report("panicked at src/main.rs:1:1: no GPU");
        assert!(report.contains("no GPU"));
        assert!(report.contains("Backtrace:"));
        assert!(report.contains("issues/new?template=crash.yml"));
    }
}
