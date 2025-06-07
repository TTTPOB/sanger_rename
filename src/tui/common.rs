use sanger_rename::SangerFilename;
use sanger_rename::Vendor;

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
        if !self
            .filenames
            .iter()
            .any(|f| f.get_full_path() == filename.get_full_path())
        {
            self.filenames.push(filename);
        }
    }

    pub fn from_str_filenames(str_filenames: Vec<String>, vendor: Vendor) -> Self {
        let converted = str_filenames
            .iter()
            .map(|fn_str| SangerFilename::new(fn_str, vendor))
            .collect();
        Self {
            filenames: converted,
        }
    }
}

pub struct StrFilenames {
    pub filenames: Vec<String>,
}
