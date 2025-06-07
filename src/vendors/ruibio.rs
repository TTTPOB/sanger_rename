use crate::SangerFilename;

#[derive(Clone, PartialEq, Debug)]
pub struct RuibioSangerFilename {
    filename: String,
    primer_name: String,
    template_name: String,
    date: Option<time::Date>,
}

impl From<String> for RuibioSangerFilename {
    fn from(filename: String) -> Self {
        let mut fname = RuibioSangerFilename {
            filename,
            primer_name: String::new(),
            template_name: String::new(),
            date: None,
        };
        fname
            .set_primer_name(&fname.get_primer_name())
            .expect("Failed to set primer name");
        fname
            .set_template_name(&fname.get_template_name())
            .expect("Failed to set template name");
        fname
    }
}

impl From<&str> for RuibioSangerFilename {
    fn from(filename: &str) -> Self {
        RuibioSangerFilename::from(filename.to_string())
    }
}

impl SangerFilename for RuibioSangerFilename {
    fn get_full_path(&self) -> String {
        self.filename.clone()
    }
    fn get_template_name(&self) -> String {
        // Extract template name from pattern like "K528-1.C1.34781340.B08"
        // Template is everything before the first dot
        if let Some(first_dot) = self.filename.find('.') {
            return self.filename[..first_dot].to_string();
        }
        String::new()
    }
    fn set_primer_name(&mut self, primer_name: &str) -> anyhow::Result<()> {
        self.primer_name = primer_name.to_string();
        Ok(())
    }
    fn set_template_name(&mut self, template_name: &str) -> anyhow::Result<()> {
        self.template_name = template_name.to_string();
        Ok(())
    }
    fn set_date(&mut self, date: time::Date) -> anyhow::Result<()> {
        self.date = Some(date);
        Ok(())
    }

    fn get_primer_name(&self) -> String {
        let filestem = self.get_file_stem();
        // Extract primer name from pattern like "K528-1.C1.34781340.B08"
        // Primer is between first and second dot
        let parts: Vec<&str> = filestem.split('.').collect();
        if parts.len() >= 2 {
            return parts[1].to_string();
        }
        String::new()
    }

    fn get_vendor_id(&self) -> String {
        let filestem = self.get_file_stem();
        // Extract vendor ID from pattern like "K528-1.C1.34781340.B08"
        // Vendor ID is the last two parts joined by dot
        let parts: Vec<&str> = filestem.split('.').collect();
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
        let filename = "K528-1.C1.34781340.B08.ab1";
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
        let filename = "K528-1.C1.34781340.B08.ab1";
        let ruibio_sanger_fn: RuibioSangerFilename = filename.into();
        let date = datetime!(2025-12-06 00:00:00 +8);
        let standardized_name = ruibio_sanger_fn.get_standardized_name(Some(date.date()));
        assert_eq!(standardized_name, "251206.K528-1.C1");
    }
}
