use crate::vendors::{
    genewiz::GenewizSangerFilename, ruibio::RuibioSangerFilename, sangon::SangonSangerFilename,
};

pub trait SangerFilename {
    fn get_template_name(&self) -> String;
    fn get_primer_name(&self) -> String;
    fn get_vendor_id(&self) -> String;
    fn rename(&self, new_name: &str) -> Result<(), String>;
    fn get_vendor_name(&self) -> String;
}

#[derive(Clone, PartialEq, Debug)]
pub enum Vendor {
    Sangon(SangonSangerFilename),
    Ruibio(RuibioSangerFilename),
    Genewiz(GenewizSangerFilename),
}

pub mod vendors {
    pub mod genewiz;
    pub mod ruibio;
    pub mod sangon;
}

pub mod tui;
