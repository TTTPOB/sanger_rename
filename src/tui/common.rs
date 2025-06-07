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
}

pub struct SangerFilenames {
    pub filenames: Vec<SangerFilename>,
}

pub struct StrFilenames {
    pub filenames: Vec<String>,
}
