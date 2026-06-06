//! O15: watch `{folder}/glossary.json` for external edits, emit `FileChanged`.
//! Single watch slot — the Glossary view (un)registers on mount/unmount.

use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter};

use crate::error::{AppError, AppResult};
use crate::events::{GlossaryEvent, GLOSSARY_EVENT};

const DEBOUNCE: Duration = Duration::from_millis(300);

/// Managed Tauri state (std Mutex — accessed from sync commands + the notify
/// callback thread, never across an await).
#[derive(Default)]
pub struct GlossaryWatchState(pub Mutex<Option<WatchHandle>>);

pub struct WatchHandle {
    _watcher: RecommendedWatcher,
}

/// Leading-edge debounce: emit now iff outside the window since the last emit.
fn should_emit(last: &mut Option<Instant>, now: Instant, window: Duration) -> bool {
    match last {
        Some(t) if now.duration_since(*t) < window => false,
        _ => {
            *last = Some(now);
            true
        }
    }
}

/// Returns `true` iff at least one path in the notify event is `glossary.json`.
///
/// Atomic-save renames produce events whose paths may include the temporary
/// `.glossary.json.tmp` name; this filter passes only the final canonical name.
fn is_glossary_event(paths: &[std::path::PathBuf]) -> bool {
    paths
        .iter()
        .any(|p| p.file_name().is_some_and(|n| n == "glossary.json"))
}

/// Replaces any existing watch (single slot).
pub fn watch(app: AppHandle, state: &GlossaryWatchState, folder: &Path) -> AppResult<()> {
    let last = Mutex::new(None::<Instant>);
    let mut watcher = notify::recommended_watcher(
        move |res: Result<notify::Event, notify::Error>| {
            let Ok(event) = res else { return };
            // Folder-level watch: only glossary.json matters.
            if !is_glossary_event(&event.paths) {
                return;
            }
            if should_emit(&mut last.lock().unwrap(), Instant::now(), DEBOUNCE) {
                let _ = app.emit(GLOSSARY_EVENT, &GlossaryEvent::FileChanged);
            }
        },
    )
    .map_err(|e| AppError::Other(e.to_string()))?;
    watcher
        .watch(folder, RecursiveMode::NonRecursive)
        .map_err(|e| AppError::Other(e.to_string()))?;
    *state.0.lock().unwrap() = Some(WatchHandle { _watcher: watcher });
    Ok(())
}

pub fn unwatch(state: &GlossaryWatchState) {
    *state.0.lock().unwrap() = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{Duration, Instant};

    #[test]
    fn debounce_suppresses_within_window() {
        let t0 = Instant::now();
        let mut last: Option<Instant> = None;
        assert!(should_emit(&mut last, t0, Duration::from_millis(300)));
        assert!(!should_emit(&mut last, t0 + Duration::from_millis(100), Duration::from_millis(300)));
        assert!(should_emit(&mut last, t0 + Duration::from_millis(400), Duration::from_millis(300)));
    }

    #[test]
    fn is_glossary_event_matches_canonical_name() {
        // Exact glossary.json path → true.
        let paths = vec![PathBuf::from("/some/folder/glossary.json")];
        assert!(is_glossary_event(&paths));

        // Temporary file only (atomic-save stage) → false.
        let paths = vec![PathBuf::from("/some/folder/.glossary.json.tmp")];
        assert!(!is_glossary_event(&paths));

        // Unrelated file → false.
        let paths = vec![PathBuf::from("/some/folder/episode01.ass")];
        assert!(!is_glossary_event(&paths));
    }
}
