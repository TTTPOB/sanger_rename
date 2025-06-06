pub trait SangerFilename {
    fn get_template_name(&self) -> String;
    fn get_primer_name(&self) -> String;
    fn get_vendor_id(&self) -> String;
    fn rename(&self, new_name: &str) -> Result<(), String>;
}

pub mod vendors {
    pub mod genewiz;
    pub mod ruibio;
    pub mod sangon;
}
