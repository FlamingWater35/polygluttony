//! Load an existing `glossary.json` from a work folder (written by the Python
//! tool or, later, by our Glossary step). Missing/invalid file ⇒ `None` —
//! glossaries are optional everywhere.

use std::path::Path;

use super::model::Glossary;

pub fn load_folder_glossary(folder: &Path) -> Option<Glossary> {
    let path = folder.join("glossary.json");
    let text = std::fs::read_to_string(path).ok()?;
    Glossary::from_json(&text)
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
}
