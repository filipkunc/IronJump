use ironjump::progress::Progress;
use std::path::PathBuf;

pub struct FileProgressStore {
    path: PathBuf,
}
impl FileProgressStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
    pub fn load(&self) -> Result<Progress, String> {
        match std::fs::read_to_string(&self.path) {
            Ok(text) => Progress::parse(&text),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Progress::default()),
            Err(error) => Err(error.to_string()),
        }
    }
    pub fn save(&self, progress: Progress) -> std::io::Result<()> {
        let temporary = self.path.with_extension("tmp");
        std::fs::write(&temporary, progress.encode())?;
        std::fs::rename(temporary, &self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn progress_roundtrips_and_missing_files_start_fresh() {
        let path =
            std::env::temp_dir().join(format!("ironjump-storage-test-{}.txt", std::process::id()));
        let store = FileProgressStore::new(path.clone());
        assert_eq!(store.load().unwrap(), Progress::default());
        let progress = Progress {
            selected: 12,
            unlocked: 15,
        };
        store.save(progress).unwrap();
        assert_eq!(store.load().unwrap(), progress);
        std::fs::write(&path, "broken").unwrap();
        assert!(store.load().is_err());
        std::fs::remove_file(&path).unwrap();
        assert!(!path.with_extension("tmp").exists());
    }
}
