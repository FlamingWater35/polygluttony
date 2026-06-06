//! Load and save `glossary.json` from/to a work folder (written by the Python
//! tool or by our Glossary step). Missing/invalid file ⇒ `None` on load —
//! glossaries are optional everywhere.

use std::path::Path;

use super::model::Glossary;
use crate::error::AppResult;

pub fn load_folder_glossary(folder: &Path) -> Option<Glossary> {
    let path = folder.join("glossary.json");
    let text = std::fs::read_to_string(path).ok()?;
    Glossary::from_json(&text)
}

/// Crash-safe write: temp file in the same dir + rename, pretty JSON (the
/// file is user-editable via "Open in editor"). rename is atomic against
/// process crashes; we deliberately skip fsync — a power-loss-torn glossary
/// is recoverable by rebuilding.
pub fn save_folder_glossary(folder: &Path, glossary: &Glossary) -> AppResult<()> {
    let tmp = folder.join(".glossary.json.tmp");
    std::fs::write(&tmp, glossary.to_json_pretty())?;
    std::fs::rename(&tmp, folder.join("glossary.json"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_when_present_none_when_absent() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load_folder_glossary(dir.path()).is_none());
        std::fs::write(
            dir.path().join("glossary.json"),
            r#"{"world_type":"wuxia","terms":{"characters":{"张三":"Zhang San"}}}"#,
        )
        .unwrap();
        let g = load_folder_glossary(dir.path()).unwrap();
        assert_eq!(g.world_type, "wuxia");
        assert_eq!(g.characters.get("张三").unwrap(), "Zhang San");
    }

    #[test]
    fn save_is_atomic_and_pretty() {
        let dir = tempfile::tempdir().unwrap();
        let mut g = Glossary::new("wuxia");
        g.characters.insert("张三".into(), "Zhang San".into());
        save_folder_glossary(dir.path(), &g).unwrap();
        // No temp file left behind.
        assert!(!dir.path().join(".glossary.json.tmp").exists());
        let text = std::fs::read_to_string(dir.path().join("glossary.json")).unwrap();
        assert!(text.contains("\n  ")); // pretty
        let back = load_folder_glossary(dir.path()).unwrap();
        assert_eq!(back.characters.get("张三").unwrap(), "Zhang San");
    }
}
