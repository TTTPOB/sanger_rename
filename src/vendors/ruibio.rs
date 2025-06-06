use crate::SangerFilename;

#[derive(Clone, PartialEq, Debug)]
pub struct RuibioSangerFilename {
    filename: String,
}

impl From<String> for RuibioSangerFilename {
    fn from(filename: String) -> Self {
        RuibioSangerFilename { filename }
    }
}

impl From<&str> for RuibioSangerFilename {
    fn from(filename: &str) -> Self {
        RuibioSangerFilename {
            filename: filename.to_string(),
        }
    }
}

impl SangerFilename for RuibioSangerFilename {
    fn get_template_name(&self) -> String {
        // Extract template name from pattern like "K528-1.C1.34781340.B08"
        // Template is everything before the first dot
        if let Some(first_dot) = self.filename.find('.') {
            return self.filename[..first_dot].to_string();
        }
        String::new()
    }

    fn get_primer_name(&self) -> String {
        // Extract primer name from pattern like "K528-1.C1.34781340.B08"
        // Primer is between first and second dot
        let parts: Vec<&str> = self.filename.split('.').collect();
        if parts.len() >= 2 {
            return parts[1].to_string();
        }
        String::new()
    }

    fn get_vendor_id(&self) -> String {
        // Extract vendor ID from pattern like "K528-1.C1.34781340.B08"
        // Vendor ID is the last two parts joined by dot
        let parts: Vec<&str> = self.filename.split('.').collect();
        if parts.len() >= 3 {
            return format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1]);
        }
        String::new()
    }

    fn rename(&self, _new_name: &str) -> Result<(), String> {
        // This would typically rename the actual file
        // For now, just return Ok as a placeholder
        Ok(())
    }
    fn get_vendor_name(&self) -> String {
        "Ruibio".to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use time::macros::datetime;
    #[test]
    fn test_ruibio_extraction() {
        let filename = "K528-1.C1.34781340.B08";
        let vendor_id = "34781340.B08";
        let template_name = "K528-1";
        let primer_name = "C1";
        let ruibio_sanger_fn: RuibioSangerFilename = filename.into();
        assert_eq!(ruibio_sanger_fn.get_vendor_id(), vendor_id);
        assert_eq!(ruibio_sanger_fn.get_template_name(), template_name);
        assert_eq!(ruibio_sanger_fn.get_primer_name(), primer_name);
    }
    #[test]
    fn test_ruibio_standardized_name() {
        let filename = "K528-1.C1.34781340.B08";
        let ruibio_sanger_fn: RuibioSangerFilename = filename.into();
        let date = datetime!(2025-12-06 00:00:00 +8);
        let standardized_name = ruibio_sanger_fn.get_standardized_name(Some(date.date()));
        assert_eq!(standardized_name, "251206.K528-1.C1");
    }
}
