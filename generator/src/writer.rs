use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct GeneratedFiles {
    files: Vec<PathBuf>,
}

impl GeneratedFiles {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn add_file(&mut self, path: PathBuf) {
        self.files.push(path);
    }

    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    pub fn count(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

impl Default for GeneratedFiles {
    fn default() -> Self {
        Self::new()
    }
}