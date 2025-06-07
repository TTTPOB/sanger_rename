use std::str::FromStr;
use strum::EnumIter;

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumIter)]
pub enum Vendor {
    Sangon,
    Ruibio,
    Genewiz,
}

impl FromStr for Vendor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sangon" => Ok(Vendor::Sangon),
            "ruibio" => Ok(Vendor::Ruibio),
            "genewiz" => Ok(Vendor::Genewiz),
            _ => Err(format!("Unknown vendor: {}", s)),
        }
    }
}

impl std::fmt::Display for Vendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Vendor::Sangon => write!(f, "Sangon"),
            Vendor::Ruibio => write!(f, "Ruibio"),
            Vendor::Genewiz => write!(f, "Genewiz"),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct SangerFilename {
    filename: String,
    primer_name: String,
    template_name: String,
    date: Option<time::Date>,
    vendor: Vendor,
}

impl SangerFilename {
    /// Create a new SangerFilename with the specified vendor
    pub fn new(filename: String, vendor: Vendor) -> Self {
        let mut sanger_filename = SangerFilename {
            filename,
            primer_name: String::new(),
            template_name: String::new(),
            date: None,
            vendor,
        };

        // Extract primer and template names based on vendor
        let primer_name = sanger_filename.get_primer_name();
        let template_name = sanger_filename.get_template_name();

        sanger_filename
            .set_primer_name(&primer_name)
            .expect("Failed to set primer name");
        sanger_filename
            .set_template_name(&template_name)
            .expect("Failed to set template name");

        sanger_filename
    }

    /// Create from filename string with vendor detection
    pub fn from_filename_with_vendor(filename: String, vendor: Vendor) -> Self {
        Self::new(filename, vendor)
    }

    pub fn get_full_path(&self) -> String {
        self.filename.clone()
    }

    pub fn get_file_stem(&self) -> String {
        std::path::Path::new(&self.get_full_path())
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    /// Get the filename with extension but without the full path
    pub fn show_file_name(&self) -> String {
        std::path::Path::new(&self.get_full_path())
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    pub fn get_extension_name(&self) -> String {
        std::path::Path::new(&self.get_full_path())
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    pub fn get_template_name(&self) -> String {
        if !self.template_name.is_empty() {
            return self.template_name.clone();
        }

        match self.vendor {
            Vendor::Sangon => self.extract_sangon_template_name(),
            Vendor::Ruibio => self.extract_ruibio_template_name(),
            Vendor::Genewiz => self.extract_genewiz_template_name(),
        }
    }

    pub fn get_primer_name(&self) -> String {
        if !self.primer_name.is_empty() {
            return self.primer_name.clone();
        }

        match self.vendor {
            Vendor::Sangon => self.extract_sangon_primer_name(),
            Vendor::Ruibio => self.extract_ruibio_primer_name(),
            Vendor::Genewiz => self.extract_genewiz_primer_name(),
        }
    }

    pub fn set_primer_name(&mut self, primer_name: &str) -> anyhow::Result<()> {
        self.primer_name = primer_name.to_string();
        Ok(())
    }

    pub fn set_template_name(&mut self, template_name: &str) -> anyhow::Result<()> {
        self.template_name = template_name.to_string();
        Ok(())
    }

    pub fn set_date(&mut self, date: time::Date) -> anyhow::Result<()> {
        self.date = Some(date);
        Ok(())
    }

    pub fn get_vendor_id(&self) -> String {
        match self.vendor {
            Vendor::Sangon => self.extract_sangon_vendor_id(),
            Vendor::Ruibio => self.extract_ruibio_vendor_id(),
            Vendor::Genewiz => self.extract_genewiz_vendor_id(),
        }
    }

    pub fn get_vendor_name(&self) -> String {
        self.vendor.to_string()
    }

    pub fn get_vendor(&self) -> &Vendor {
        &self.vendor
    }
    pub fn set_vendor(&mut self, vendor: Vendor) {
        self.vendor = vendor;
        // Clear cached values and re-extract based on new vendor
        self.primer_name = String::new();
        self.template_name = String::new();
        let primer_name = self.get_primer_name();
        let template_name = self.get_template_name();
        self.primer_name = primer_name;
        self.template_name = template_name;
    }

    pub fn move_to_standardized_name(&self) -> anyhow::Result<()> {
        let standardized_name = self.get_standardized_name();
        let new_path =
            std::path::Path::new(&self.get_full_path()).with_file_name(standardized_name);

        let new_path = format!(
            "{}.{}",
            new_path.to_string_lossy(),
            self.get_extension_name()
        );

        // Rename the file on disk
        std::fs::rename(&self.get_full_path(), new_path)?;
        Ok(())
    }

    pub fn get_standardized_name(&self) -> String {
        // if date is None, use today
        let current_time = time::OffsetDateTime::now_local().unwrap();
        let date = self.date.unwrap_or_else(|| {
            time::Date::from_calendar_date(
                current_time.year(),
                current_time.month(),
                current_time.day(),
            )
            .expect("Failed to get current date")
        });
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

    // Sangon-specific extraction methods
    fn extract_sangon_template_name(&self) -> String {
        // Extract template name from pattern like "0001_31225060307072_(TXPCR)_[SP1]"
        if let Some(start) = self.filename.find('(') {
            if let Some(end) = self.filename.find(')') {
                if end > start {
                    return self.filename[start + 1..end].to_string();
                }
            }
        }
        String::new()
    }

    fn extract_sangon_primer_name(&self) -> String {
        let filestem = self.get_file_stem();
        // Extract primer name from pattern like "0001_31225060307072_(TXPCR)_[SP1]"
        if let Some(start) = filestem.find('[') {
            if let Some(end) = filestem.find(']') {
                if end > start {
                    return filestem[start + 1..end].to_string();
                }
            }
        }
        String::new()
    }

    fn extract_sangon_vendor_id(&self) -> String {
        // Extract vendor ID from pattern like "0001_31225060307072_(TXPCR)_[SP1]"
        let filestem = self.get_file_stem();
        let parts: Vec<&str> = filestem.split('_').collect();
        if parts.len() >= 2 {
            return parts[1].to_string();
        }
        String::new()
    } // Ruibio-specific extraction methods
    fn extract_ruibio_template_name(&self) -> String {
        // Extract template name from pattern like "K528-1.C1.34781340.B08"
        // Template is everything before the first dot
        let filestem = self.get_file_stem();
        if let Some(first_dot) = filestem.find('.') {
            return filestem[..first_dot].to_string();
        }
        String::new()
    }

    fn extract_ruibio_primer_name(&self) -> String {
        let filestem = self.get_file_stem();
        // Extract primer name from pattern like "K528-1.C1.34781340.B08"
        // Primer is between first and second dot
        let parts: Vec<&str> = filestem.split('.').collect();
        if parts.len() >= 2 {
            return parts[1].to_string();
        }
        String::new()
    }

    fn extract_ruibio_vendor_id(&self) -> String {
        let filestem = self.get_file_stem();
        // Extract vendor ID from pattern like "K528-1.C1.34781340.B08"
        // Vendor ID is the last two parts joined by dot
        let parts: Vec<&str> = filestem.split('.').collect();
        if parts.len() >= 3 {
            return format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1]);
        }
        String::new()
    }

    // Genewiz-specific extraction methods
    fn extract_genewiz_template_name(&self) -> String {
        let filestem = self.get_file_stem();
        // Extract template name from pattern like "TL1-T25_A01" or "k1-2-C1_R_G04"
        // Find the last underscore to locate the vendor ID
        if let Some(underscore_pos) = filestem.rfind('_') {
            // Find the last dash before the underscore to separate template from primer
            let before_underscore = &filestem[..underscore_pos];
            if let Some(dash_pos) = before_underscore.rfind('-') {
                return filestem[..dash_pos].to_string();
            }
        }
        String::new()
    }

    fn extract_genewiz_primer_name(&self) -> String {
        let filestem = self.get_file_stem();
        // Extract primer name from pattern like "TL1-T25_A01" or "k1-2-C1_R_G04"
        // Find the last underscore to locate the vendor ID
        if let Some(underscore_pos) = filestem.rfind('_') {
            // Find the last dash before the underscore to separate template from primer
            let before_underscore = &filestem[..underscore_pos];
            if let Some(dash_pos) = before_underscore.rfind('-') {
                return filestem[dash_pos + 1..underscore_pos].to_string();
            }
        }
        String::new()
    }

    fn extract_genewiz_vendor_id(&self) -> String {
        let filestem = self.get_file_stem();
        // Extract vendor ID from pattern like "TL1-T25_A01" or "k1-2-C1_R_G04"
        if let Some(underscore_pos) = filestem.rfind('_') {
            return filestem[underscore_pos + 1..].to_string();
        }
        String::new()
    }
}

// Implement From traits for backward compatibility
impl From<(String, Vendor)> for SangerFilename {
    fn from((filename, vendor): (String, Vendor)) -> Self {
        Self::new(filename, vendor)
    }
}

impl From<(&str, Vendor)> for SangerFilename {
    fn from((filename, vendor): (&str, Vendor)) -> Self {
        Self::new(filename.to_string(), vendor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sangon_extraction() {
        let filename = "0001_31225060307072_(TXPCR)_[SP1].ab1";
        let vendor_id = "31225060307072";
        let template_name = "TXPCR";
        let primer_name = "SP1";
        let sangon_sanger_fn = SangerFilename::new(filename.to_string(), Vendor::Sangon);
        assert_eq!(sangon_sanger_fn.get_vendor_id(), vendor_id);
        assert_eq!(sangon_sanger_fn.get_template_name(), template_name);
        assert_eq!(sangon_sanger_fn.get_primer_name(), primer_name);
    }

    #[test]
    fn test_sangon_standardized_name() {
        let filename = "0001_31225060307072_(TXPCR)_[SP1].ab1";
        let mut sangon_sanger_fn = SangerFilename::new(filename.to_string(), Vendor::Sangon);
        let date = time::Date::from_calendar_date(2025, time::Month::June, 1)
            .expect("Failed to create date");
        sangon_sanger_fn.set_date(date).unwrap();
        let standardized_name = sangon_sanger_fn.get_standardized_name();
        assert_eq!(standardized_name, "250601.TXPCR.SP1");
    }

    #[test]
    fn test_ruibio_extraction() {
        let filename = "K528-1.C1.34781340.B08.ab1";
        let vendor_id = "34781340.B08";
        let template_name = "K528-1";
        let primer_name = "C1";
        let ruibio_sanger_fn = SangerFilename::new(filename.to_string(), Vendor::Ruibio);
        assert_eq!(ruibio_sanger_fn.get_vendor_id(), vendor_id);
        assert_eq!(ruibio_sanger_fn.get_template_name(), template_name);
        assert_eq!(ruibio_sanger_fn.get_primer_name(), primer_name);
    }

    #[test]
    fn test_ruibio_standardized_name() {
        let filename = "K528-1.C1.34781340.B08.ab1";
        let mut ruibio_sanger_fn = SangerFilename::new(filename.to_string(), Vendor::Ruibio);
        let date = time::Date::from_calendar_date(2025, time::Month::December, 6)
            .expect("Failed to create date");
        ruibio_sanger_fn.set_date(date).unwrap();
        let standardized_name = ruibio_sanger_fn.get_standardized_name();
        assert_eq!(standardized_name, "251206.K528-1.C1");
    }

    #[test]
    fn test_genewiz_extraction() {
        let filename = "TL1-T25_A01.ab1";
        let template_name = "TL1";
        let vendor_id = "A01";
        let primer_name = "T25";
        let genewiz_sanger_fn = SangerFilename::new(filename.to_string(), Vendor::Genewiz);
        assert_eq!(genewiz_sanger_fn.get_vendor_id(), vendor_id);
        assert_eq!(genewiz_sanger_fn.get_template_name(), template_name);
        assert_eq!(genewiz_sanger_fn.get_primer_name(), primer_name);
    }

    #[test]
    fn test_genewiz_extraction2() {
        let filename = "k1-2-C1_R_G04.ab1";
        let template_name = "k1-2";
        let vendor_id = "G04";
        let primer_name = "C1_R";
        let genewiz_sanger_fn = SangerFilename::new(filename.to_string(), Vendor::Genewiz);
        assert_eq!(genewiz_sanger_fn.get_vendor_id(), vendor_id);
        assert_eq!(genewiz_sanger_fn.get_template_name(), template_name);
        assert_eq!(genewiz_sanger_fn.get_primer_name(), primer_name);
    }
    #[test]
    fn test_vendor_switching() {
        let filename = "0001_31225060307072_(TXPCR)_[SP1].ab1";
        let mut sanger_fn = SangerFilename::new(filename.to_string(), Vendor::Sangon);

        // Test initial Sangon extraction
        assert_eq!(sanger_fn.get_vendor_name(), "Sangon");
        assert_eq!(sanger_fn.get_template_name(), "TXPCR");
        assert_eq!(sanger_fn.get_primer_name(), "SP1");

        // Switch to Ruibio (should extract differently, but filename doesn't match Ruibio pattern)
        sanger_fn.set_vendor(Vendor::Ruibio);
        assert_eq!(sanger_fn.get_vendor_name(), "Ruibio");
        // The filename doesn't match Ruibio pattern, so extraction will be empty
        assert_eq!(sanger_fn.get_template_name(), "");
        assert_eq!(sanger_fn.get_primer_name(), "");
    }

    #[test]
    fn test_vendor_from_string() {
        assert_eq!(Vendor::from_str("sangon").unwrap(), Vendor::Sangon);
        assert_eq!(Vendor::from_str("RUIBIO").unwrap(), Vendor::Ruibio);
        assert_eq!(Vendor::from_str("GenEwiz").unwrap(), Vendor::Genewiz);
        assert!(Vendor::from_str("unknown").is_err());
    }

    #[test]
    fn test_show_file_name() {
        let filename1 = "0001_31225060307072_(TXPCR)_[SP1].ab1";
        let sanger_fn1 = SangerFilename::new(filename1.to_string(), Vendor::Sangon);
        assert_eq!(
            sanger_fn1.show_file_name(),
            "0001_31225060307072_(TXPCR)_[SP1].ab1"
        );

        let filename2 = "/path/to/file/K528-1.C1.34781340.B08.ab1";
        let sanger_fn2 = SangerFilename::new(filename2.to_string(), Vendor::Ruibio);
        assert_eq!(sanger_fn2.show_file_name(), "K528-1.C1.34781340.B08.ab1");

        let filename3 = "C:\\Users\\test\\TL1-T25_A01.ab1";
        let sanger_fn3 = SangerFilename::new(filename3.to_string(), Vendor::Genewiz);
        assert_eq!(sanger_fn3.show_file_name(), "TL1-T25_A01.ab1");
    }

    #[test]
    fn test_move_to_standardized_name() {
        let filename = "0001_31225060307072_(TXPCR)_[SP1].ab1";
        // create the file at system temp dir
        let temp_dir = std::env::temp_dir();
        let full_path = temp_dir.join(filename);
        std::fs::write(&full_path, b"test content").expect("Failed to create test file");
        let mut sanger_fn =
            SangerFilename::new(full_path.to_string_lossy().to_string(), Vendor::Sangon);
        let date = time::Date::from_calendar_date(2025, time::Month::June, 1)
            .expect("Failed to create date");
        sanger_fn.set_date(date).unwrap();
        // Move to standardized name
        sanger_fn
            .move_to_standardized_name()
            .expect("Failed to move file");
        // Check if the file was renamed correctly
        let standardized_name = sanger_fn.get_standardized_name();
        let new_full_path = temp_dir.join(format!("{}.ab1", standardized_name));
        // print content of the temp dir for debugging
        let dir_content = std::fs::read_dir(temp_dir).unwrap();
        for entry in dir_content {
            let entry = entry.unwrap();
            // if ends with .ab1, print the file name
            if entry.path().extension().map_or(false, |ext| ext == "ab1") {
                println!("File in temp dir: {}", entry.path().display());
            }
        }
        assert!(new_full_path.exists(), "Standardized file does not exist");
    }
}
