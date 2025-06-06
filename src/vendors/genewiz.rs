use crate::SangerFilename;

#[derive(Clone, PartialEq, Debug)]
pub struct GenewizSangerFilename {
    filename: String,
}

impl From<String> for GenewizSangerFilename {
    fn from(filename: String) -> Self {
        GenewizSangerFilename { filename }
    }
}

impl From<&str> for GenewizSangerFilename {
    fn from(filename: &str) -> Self {
        GenewizSangerFilename {
            filename: filename.to_string(),
        }
    }
}

impl SangerFilename for GenewizSangerFilename {
    fn get_full_path(&self) -> String {
        self.filename.clone()
    }
    fn get_template_name(&self) -> String {
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
    fn get_primer_name(&self) -> String {
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
    fn get_vendor_id(&self) -> String {
        let filestem = self.get_file_stem();
        // Extract vendor ID from pattern like "TL1-T25_A01" or "k1-2-C1_R_G04"
        if let Some(underscore_pos) = filestem.rfind('_') {
            return filestem[underscore_pos + 1..].to_string();
        }
        String::new()
    }

    fn rename(&self, _new_name: &str) -> Result<(), String> {
        // This would typically rename the actual file
        // For now, just return Ok as a placeholder
        Ok(())
    }
    fn get_vendor_name(&self) -> String {
        "Genewiz".to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_genewiz_extraction() {
        let filename = "TL1-T25_A01.ab1";
        let template_name = "TL1";
        let vendor_id = "A01";
        let primer_name = "T25";
        let genewiz_sanger_fn: GenewizSangerFilename = filename.into();
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
        let genewiz_sanger_fn: GenewizSangerFilename = filename.into();
        assert_eq!(genewiz_sanger_fn.get_vendor_id(), vendor_id);
        assert_eq!(genewiz_sanger_fn.get_template_name(), template_name);
        assert_eq!(genewiz_sanger_fn.get_primer_name(), primer_name);
    }
}
