//! iOS: what the command line, the working directory and the data folders are on a desktop.
//!
//! An iOS app starts with no arguments, with `/` as its working directory, and may write only
//! below `$HOME`, the app's container. The desktop game keeps its saves, `options.txt` and caches
//! in the working directory and reads the prepared resources from `.local/horizon-client/26.3`
//! of it, so [`enter_sandbox`] moves the process into the game folder before anything looks at
//! the working directory (`entry::main_with`):
//!
//! ```text
//! Documents/                         shown by the Files app (UIFileSharingEnabled)
//!   Horizon/                         the game folder, the working directory
//!     horizon-client/26.3/           the prepared resources: the player copies them here
//!     saves/  options.txt  ...       as on a desktop
//! Library/Application Support/horizon/   logs and crash reports (`logging::game_home`)
//! ```
//!
//! The module builds on every platform so its rules are tested on a desktop; only iOS calls
//! [`enter_sandbox`].
use std::path::{Path, PathBuf};

/// The game folder's name below `Documents/`.
pub const GAME_FOLDER: &str = "Horizon";

/// The render distance a first start on a phone uses: the desktop's 12 chunks keep a phone's
/// memory (iOS ends an app that holds more than about 2 to 3 GB) and its GPU busy.
pub const DEFAULT_RENDER_DISTANCE: u32 = 6;

/// The app's container (`NSHomeDirectory()`); iOS sets `HOME` to it.
fn container() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .filter(|h| !h.is_empty())
        .map(PathBuf::from)
}

/// `Documents/` of a container.
pub fn documents_in(container: &Path) -> PathBuf {
    container.join("Documents")
}

/// The game folder of a container.
pub fn game_dir_in(container: &Path) -> PathBuf {
    documents_in(container).join(GAME_FOLDER)
}

/// The game folder of this app.
pub fn game_dir() -> Option<PathBuf> {
    container().map(|c| game_dir_in(&c))
}

/// Where the prepared resources are looked for, relative to the game folder (the working
/// directory once [`enter_sandbox`] ran): visible in the Files app, unlike the desktop's `.local`.
pub fn default_resources() -> PathBuf {
    Path::new("horizon-client").join(horizon_core::VANILLA_VERSION)
}

/// Makes the game folder and moves the process into it. Returns the folder, or what stopped it.
pub fn enter_sandbox() -> Result<PathBuf, String> {
    let dir = game_dir().ok_or_else(|| "HOME is not set".to_owned())?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    std::env::set_current_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_game_folder_is_in_documents_so_the_files_app_shows_it() {
        let dir = game_dir_in(Path::new("/var/mobile/Containers/Data/Application/ABC"));
        assert_eq!(
            dir,
            Path::new("/var/mobile/Containers/Data/Application/ABC/Documents/Horizon")
        );
    }

    #[test]
    fn the_resources_are_not_in_a_hidden_folder() {
        let resources = default_resources();
        assert!(
            resources
                .components()
                .all(|c| !c.as_os_str().to_string_lossy().starts_with('.')),
            "{resources:?}: the Files app does not list dot folders"
        );
        assert!(resources.ends_with(horizon_core::VANILLA_VERSION));
    }
}
