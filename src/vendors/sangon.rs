use crate::SangerFilename;

#[derive(Clone, PartialEq, Debug)]
pub struct SangonSangerFilename {
    filename: String,
}

impl From<String> for SangonSangerFilename {
    fn from(filename: String) -> Self {
        SangonSangerFilename { filename }
    }
}

impl From<&str> for SangonSangerFilename {
    fn from(filename: &str) -> Self {
        SangonSangerFilename {
            filename: filename.to_string(),
        }
    }
}

impl SangerFilename for SangonSangerFilename {
    fn get_full_path(&self) -> String {
        self.filename.clone()
    }
    fn get_template_name(&self) -> String {
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

    fn get_primer_name(&self) -> String {
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

    fn get_vendor_id(&self) -> String {
        // Extract vendor ID from pattern like "0001_31225060307072_(TXPCR)_[SP1]"
        let filestem = self.get_file_stem();
        let parts: Vec<&str> = filestem.split('_').collect();
        if parts.len() >= 2 {
            return parts[1].to_string();
        }
        String::new()
    }

    fn rename(&self, _new_name: &str) -> Result<(), String> {
        // This would typically rename the actual file
        // For now, just return Ok as a placeholder
        Ok(())
    }
    fn get_vendor_name(&self) -> String {
        "Sangon".to_string()
    }
}

#[cfg(test)]
mod test {
    use time::macros::datetime;

    use super::*;

    #[test]
    fn test_sangon_extraction() {
        let filename = "0001_31225060307072_(TXPCR)_[SP1].ab1";
        let vendor_id = "31225060307072";
        let template_name = "TXPCR";
        let primer_name = "SP1";
        let sangon_sanger_fn: SangonSangerFilename = filename.into();
        assert_eq!(sangon_sanger_fn.get_vendor_id(), vendor_id);
        assert_eq!(sangon_sanger_fn.get_template_name(), template_name);
        assert_eq!(sangon_sanger_fn.get_primer_name(), primer_name);
    }

    #[test]
    fn test_sangon_standardized_name() {
        let filename = "0001_31225060307072_(TXPCR)_[SP1].ab1";
        let sangon_sanger_fn: SangonSangerFilename = filename.into();
        let date = datetime!(2025-06-01 00:00:00 +8);
        let standardized_name = sangon_sanger_fn.get_standardized_name(Some(date.date()));
        assert_eq!(standardized_name, "250601.TXPCR.SP1");
    }
}
