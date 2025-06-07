use sanger_rename::SangerFilename;

// Enum to handle stage transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageTransition {
    Stay,
    Next(Stage),
    Previous(Stage),
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    VendorSelection,
    PrimerRename,
    TemplateRename,
    DateSelection,
    ConfirmRename,
}

pub struct SangerFilenames {
    pub filenames: Vec<SangerFilename>,
}
impl SangerFilenames {
    pub fn new() -> Self {
        Self {
            filenames: Vec::new(),
        }
    }

    pub fn add_filename(&mut self, filename: SangerFilename) {
        if !self.filenames.contains(&filename) {
            self.filenames.push(filename);
        }
    }
}

pub struct StrFilenames {
    pub filenames: Vec<String>,
}
