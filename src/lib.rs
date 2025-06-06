use std::fmt::Display;

use crate::vendors::{
    genewiz::GenewizSangerFilename, ruibio::RuibioSangerFilename, sangon::SangonSangerFilename,
};

pub trait SangerFilename {
    fn get_template_name(&self) -> String;
    fn get_primer_name(&self) -> String;
    fn get_vendor_id(&self) -> String;
    fn rename(&self, new_name: &str) -> Result<(), String>;
    fn get_vendor_name(&self) -> String;
    fn get_standardized_name(&self, date: Option<time::Date>) -> String {
        let date = date.unwrap_or_else(|| time::OffsetDateTime::now_utc().date());
        let template_name = self.get_template_name();
        let primer_name = self.get_primer_name();
        // date of 2025 12m 06d to 251206
        let date_str = format!(
            "{:02}{:02}{:02}",
            date.year() % 100, // last two digits of the year
            date.month() as u8,
            date.day()
        );
        format!("{}.{}.{}", date_str, template_name, primer_name)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum SangerFilenameVaraint {
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
