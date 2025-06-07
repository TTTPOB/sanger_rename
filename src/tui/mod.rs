use crossterm::event::{self, KeyEvent, KeyEventKind};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    prelude::*,
    widgets::{Block, Borders, Row, Table},
};
use sanger_rename::{SangerFilename, Vendor};
use std::io::Stdout;
use std::rc::Rc;
use std::sync::Mutex;
use strum::IntoEnumIterator;

pub mod common;
pub mod date_selection;
pub mod primer_rename;
pub mod template_rename;
pub mod vendor_selection;

pub use common::{SangerFilenames, Stage, StageTransition, StrFilenames};
pub use date_selection::DateSelectionStage;
pub use primer_rename::PrimerRenameStage;
pub use template_rename::TemplateRenameStage;
pub use vendor_selection::VendorSelectionStage;

// Extension trait for additional TUI-specific methods on Vendor
pub trait VendorExt {
    fn all() -> Vec<Vendor>;
    fn from_index(index: usize) -> Option<Vendor>;
}

impl VendorExt for Vendor {
    fn all() -> Vec<Vendor> {
        Vendor::iter().collect()
    }

    fn from_index(index: usize) -> Option<Vendor> {
        Self::all().get(index).copied()
    }
}

pub struct App {
    pub should_quit: bool,
    pub quit_without_selection: bool,
    pub stage: Stage,
    sanger_fns: Rc<Mutex<SangerFilenames>>,
    str_fns: StrFilenames,
    vendor_selection: VendorSelectionStage,
    primer_rename: PrimerRenameStage,
    template_rename: TemplateRenameStage,
    date_selection: DateSelectionStage,
}

impl Default for App {
    fn default() -> App {
        App {
            should_quit: false,
            quit_without_selection: false,
            stage: Stage::VendorSelection,
            sanger_fns: Rc::new(Mutex::new(SangerFilenames {
                filenames: Vec::new(),
            })),
            str_fns: StrFilenames {
                filenames: Vec::new(),
            },
            vendor_selection: VendorSelectionStage::new(),
            primer_rename: PrimerRenameStage::init(),
            template_rename: TemplateRenameStage::init(),
            date_selection: DateSelectionStage::init(),
        }
    }
}

impl App {
    pub fn new() -> App {
        App::default()
    }
    fn add_filename(&mut self, filename: String) {
        self.str_fns.filenames.push(filename);
    }
    pub fn add_filenames(&mut self, filenames: Vec<String>) {
        self.str_fns.filenames.extend(filenames);
    }
    pub fn get_filenames(&self) -> &Vec<String> {
        &self.str_fns.filenames
    }
    pub fn get_all_primer_names(&self) -> anyhow::Result<Vec<String>> {
        if self.stage != Stage::PrimerRename {
            return Err(anyhow::anyhow!("Not in primer rename stage"));
        }
        let mut primer_names = Vec::new();
        for sanger_file in self.sanger_fns.lock().unwrap().filenames.iter() {
            primer_names.push(sanger_file.get_primer_name());
        }
        Ok(primer_names)
    }
    pub fn get_sanger_filenames(&self) -> Vec<SangerFilename> {
        let v = self.sanger_fns.lock().unwrap().filenames.clone();
        v
    }
    pub fn filenames_string_to_sanger(&mut self) -> anyhow::Result<()> {
        for filename in &self.str_fns.filenames {
            match self.vendor_selection.get_selected_vendor() {
                Some(Vendor::Sangon) => {
                    let fns = SangerFilename::new(filename.clone(), Vendor::Sangon);
                    self.sanger_fns.lock().unwrap().add_filename(fns);
                }
                Some(Vendor::Ruibio) => {
                    let fns = SangerFilename::new(filename.clone(), Vendor::Ruibio);
                    self.sanger_fns.lock().unwrap().add_filename(fns);
                }
                Some(Vendor::Genewiz) => {
                    let fns = SangerFilename::new(filename.clone(), Vendor::Genewiz);
                    self.sanger_fns.lock().unwrap().add_filename(fns);
                }
                None => {
                    return Err(anyhow::anyhow!("No vendor selected"));
                }
            }
        }
        Ok(())
    }
    pub fn get_selected_vendor(&self) -> Option<Vendor> {
        self.vendor_selection.get_selected_vendor()
    }
    pub fn set_vendor_highlighted(&mut self, index: usize) {
        self.vendor_selection.set_highlighted(index);
    }
    pub fn get_vendor_highlighted(&self) -> usize {
        self.vendor_selection.get_highlighted()
    }
    pub fn set_selected_vendor(&mut self, vendor: Option<Vendor>) {
        self.vendor_selection.set_selected_vendor(vendor);
    }
    pub fn handle_key_vendor_selection(&mut self, key: KeyEvent) {
        let transition = self.vendor_selection.handle_key(key);
        self.handle_stage_transition(transition);
    }
    fn handle_stage_transition(&mut self, transition: StageTransition) {
        match transition {
            StageTransition::Stay => {}
            StageTransition::Next(stage) => {
                self.stage = stage;
                match self.stage {
                    Stage::PrimerRename => {
                        self.filenames_string_to_sanger().unwrap();
                        let sanger_fns = Rc::clone(&self.sanger_fns);
                        self.primer_rename = PrimerRenameStage::from_sanger_fns(sanger_fns);
                    }
                    Stage::TemplateRename => {
                        let sanger_fns = Rc::clone(&self.sanger_fns);
                        self.template_rename = TemplateRenameStage::from_sanger_fns(sanger_fns);
                    }
                    Stage::DateSelection => {
                        let sanger_fns = Rc::clone(&self.sanger_fns);
                        self.date_selection = DateSelectionStage::from_sanger_fns(sanger_fns);
                    }
                    _ => {}
                }
            }
            StageTransition::Previous(stage) => {
                self.stage = stage;
                match self.stage {
                    Stage::VendorSelection => {
                        self.vendor_selection = VendorSelectionStage::new();
                    }
                    Stage::PrimerRename => {
                        let sanger_fns = Rc::clone(&self.sanger_fns);
                        self.primer_rename = PrimerRenameStage::from_sanger_fns(sanger_fns);
                    }
                    Stage::TemplateRename => {
                        let sanger_fns = Rc::clone(&self.sanger_fns);
                        self.template_rename = TemplateRenameStage::from_sanger_fns(sanger_fns);
                    }
                    Stage::DateSelection => {
                        let sanger_fns = Rc::clone(&self.sanger_fns);
                        self.date_selection = DateSelectionStage::from_sanger_fns(sanger_fns);
                    }
                }
            }
            StageTransition::Quit => self.should_quit = true,
        }
    }
    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        let transition = match self.stage {
            Stage::VendorSelection => self.vendor_selection.handle_key(key),
            Stage::PrimerRename => self.primer_rename.handle_key(key),
            Stage::DateSelection => self.date_selection.handle_key(key),
            Stage::TemplateRename => self.template_rename.handle_key(key),
        };
        self.handle_stage_transition(transition);
    }
    pub fn vendor_selection_page(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> anyhow::Result<()> {
        self.vendor_selection.render(terminal)
    }
    pub fn primer_rename_page(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> anyhow::Result<()> {
        self.primer_rename.render(terminal)
    }
    pub fn template_rename_page(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> anyhow::Result<()> {
        self.template_rename.render(terminal)
    }
    pub fn date_selection_page(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> anyhow::Result<()> {
        self.date_selection.render(terminal)
    }
    fn render_rename_preview_table(
        frame: &mut Frame,
        area: Rect,
        sanger_fns: &Rc<Mutex<SangerFilenames>>,
    ) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Rename Preview")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Cyan));
        let header = Row::new(["Original", "-->", "Standardized"])
            .style(Style::default().add_modifier(Modifier::BOLD));

        let mut rows = vec![];
        for sf in sanger_fns.lock().unwrap().filenames.iter() {
            let original_name = sf.show_file_name();
            let extname = sf.get_extension_name();
            let standardized_name = format!("{}.{}", sf.get_standardized_name(), extname);
            rows.push(Row::new([
                original_name,
                "-->".to_string(),
                standardized_name,
            ]));
        }

        let table_width = [
            Constraint::Percentage(45),
            Constraint::Percentage(10),
            Constraint::Percentage(45),
        ];

        let table = Table::new(rows, table_width).header(header).block(block);

        frame.render_widget(table, area);
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut term = ratatui::init();
        loop {
            match self.stage {
                Stage::VendorSelection => {
                    self.vendor_selection_page(&mut term)?;
                }
                Stage::PrimerRename => {
                    self.primer_rename_page(&mut term)?;
                }
                Stage::TemplateRename => {
                    self.template_rename_page(&mut term)?;
                }
                Stage::DateSelection => {
                    self.date_selection_page(&mut term)?;
                }
            }
            if let Some(ev) = event::read()?.as_key_press_event() {
                self.handle_key(ev);
            }
            if self.should_quit {
                break;
            }
        }
        ratatui::restore();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Setup function to create test filenames for each vendor
    fn setup_test_filenames(vendor: Vendor) -> Vec<String> {
        let fixture_dir = match vendor {
            Vendor::Sangon => "fixtures/sangon",
            Vendor::Ruibio => "fixtures/ruibio",
            Vendor::Genewiz => "fixtures/genewiz",
        };

        // Read all .ab1 files from the fixture directory
        let mut filenames = Vec::new();
        if let Ok(entries) = fs::read_dir(fixture_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("ab1") {
                        if let Some(path_str) = path.to_str() {
                            filenames.push(path_str.to_string());
                        }
                    }
                }
            }
        }
        filenames.sort(); // Ensure consistent ordering
        filenames
    }
    #[test]
    fn test_add_filenames() {
        let mut app = App::new();
        let fns = setup_test_filenames(Vendor::Ruibio);
        app.add_filenames(fns.clone());
        assert_eq!(app.get_filenames().len(), fns.len());
    }
    #[test]
    fn test_name_convert() {
        let mut app = App::new();
        app.set_selected_vendor(Some(Vendor::Ruibio));
        let filenames = setup_test_filenames(Vendor::Ruibio);
        app.add_filenames(filenames);
        let converted = app.filenames_string_to_sanger();
        assert!(converted.is_ok());
        let sanger_fns = app.get_sanger_filenames();
        assert_eq!(sanger_fns.len(), 6);
    }
    #[test]
    fn test_convert_filename() {
        let mut app = App::new();
        app.set_selected_vendor(Some(Vendor::Ruibio));
        let filenames = setup_test_filenames(Vendor::Ruibio);
        app.add_filenames(filenames.clone());
        let converted = app.filenames_string_to_sanger();
        assert!(converted.is_ok());
        let sanger_fns = app.get_sanger_filenames();
        let converted_filename = sanger_fns
            .get(0)
            .expect("Expected at least one converted filename");
        assert!(filenames.contains(&converted_filename.get_full_path()));
        assert_eq!(converted_filename.get_vendor_name(), "Ruibio");
    }
    #[test]
    fn test_get_primer_names() {
        let mut app = App::new();
        app.set_selected_vendor(Some(Vendor::Ruibio));
        let filenames = setup_test_filenames(Vendor::Ruibio);
        app.add_filenames(filenames);
        app.filenames_string_to_sanger().unwrap();
        app.stage = Stage::PrimerRename;
        let primer_names = app.get_all_primer_names().unwrap();
        assert_eq!(primer_names.len(), 6);
        // Test that we can extract primer names from the fixtures
        assert!(primer_names.contains(&"C1".to_string()));
        assert!(primer_names.contains(&"T7".to_string()));
    }
    #[test]
    fn test_primer_rename_fill() {
        let mut app = App::new();
        app.set_selected_vendor(Some(Vendor::Ruibio));
        let filenames = setup_test_filenames(Vendor::Ruibio);
        app.add_filenames(filenames);
        app.filenames_string_to_sanger().unwrap();
        app.handle_stage_transition(StageTransition::Next(Stage::PrimerRename));
        assert_eq!(app.stage, Stage::PrimerRename);
        let primer_names = app
            .primer_rename
            .rename_map
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(primer_names.len(), 5); // 5 unique primer names from 6 files
        // All primer names should initially have None as their renamed value
        for primer_name in &primer_names {
            assert_eq!(app.primer_rename.rename_map[primer_name], None);
        }
    }

    #[test]
    fn test_sangon_filenames() {
        let mut app = App::new();
        app.set_selected_vendor(Some(Vendor::Sangon));
        let filenames = setup_test_filenames(Vendor::Sangon);
        app.add_filenames(filenames);
        app.filenames_string_to_sanger().unwrap();
        let sanger_fns = app.get_sanger_filenames();
        assert_eq!(sanger_fns.len(), 5);

        // Test that we can extract template and primer names from Sangon fixtures
        let template_names: Vec<String> =
            sanger_fns.iter().map(|f| f.get_template_name()).collect();
        assert!(template_names.contains(&"TXPCR".to_string()));
        assert!(template_names.contains(&"GAPDH".to_string()));

        let primer_names: Vec<String> = sanger_fns.iter().map(|f| f.get_primer_name()).collect();
        assert!(primer_names.contains(&"SP1".to_string()));
        assert!(primer_names.contains(&"SP2".to_string()));
    }

    #[test]
    fn test_genewiz_filenames() {
        let mut app = App::new();
        app.set_selected_vendor(Some(Vendor::Genewiz));
        let filenames = setup_test_filenames(Vendor::Genewiz);
        app.add_filenames(filenames);
        app.filenames_string_to_sanger().unwrap();
        let sanger_fns = app.get_sanger_filenames();
        assert_eq!(sanger_fns.len(), 5);

        // Test that we can extract template and primer names from Genewiz fixtures
        let template_names: Vec<String> =
            sanger_fns.iter().map(|f| f.get_template_name()).collect();
        assert!(template_names.contains(&"TL1".to_string()));
        assert!(template_names.contains(&"k1-2".to_string()));

        let primer_names: Vec<String> = sanger_fns.iter().map(|f| f.get_primer_name()).collect();
        assert!(primer_names.contains(&"T25".to_string()));
    }
}
