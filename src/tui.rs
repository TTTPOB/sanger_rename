use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use sanger_rename::{SangerFilename, Vendor};
use std::io::Stdout;
use std::sync::Mutex;
use std::{collections::HashMap, rc::Rc};
use strum::IntoEnumIterator;

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
    date_selection: DateSelectionStage,
    rename_preview: RenamePreviewStage,
}

// Stage-specific structs
struct VendorSelectionStage {
    highlighted: usize,
    selected_vendor: Option<Vendor>,
}

struct PrimerRenameStage {
    sanger_fns: Rc<Mutex<SangerFilenames>>,
    rename_map: HashMap<String, Option<String>>,
}

struct DateSelectionStage {
    // Add fields specific to date selection stage
    // For now, keeping it empty as it's not implemented yet
}

struct RenamePreviewStage {
    // Add fields specific to rename preview stage
    // For now, keeping it empty as it's not implemented yet
}

impl VendorSelectionStage {
    pub fn new() -> Self {
        Self {
            highlighted: 0,
            selected_vendor: None,
        }
    }

    pub fn set_highlighted(&mut self, index: usize) {
        if index < Vendor::all().len() {
            self.highlighted = index;
        }
    }

    pub fn get_highlighted(&self) -> usize {
        self.highlighted
    }

    pub fn set_selected_vendor(&mut self, vendor: Option<Vendor>) {
        self.selected_vendor = vendor;
    }

    pub fn get_selected_vendor(&self) -> Option<Vendor> {
        self.selected_vendor
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> StageTransition {
        if key.kind != KeyEventKind::Press {
            return StageTransition::Stay;
        }
        match key.code {
            KeyCode::Left => {
                if self.get_highlighted() == 0 {
                    self.set_highlighted(Vendor::all().len() - 1); // Wrap around to the last vendor
                } else {
                    self.set_highlighted(self.get_highlighted() - 1);
                }
                StageTransition::Stay
            }
            KeyCode::Right => {
                if self.get_highlighted() == Vendor::all().len() - 1 {
                    self.set_highlighted(0); // Wrap around to the first vendor
                } else {
                    self.set_highlighted(self.get_highlighted() + 1);
                }
                StageTransition::Stay
            }
            KeyCode::Enter => {
                self.set_selected_vendor(Vendor::from_index(self.get_highlighted()));
                StageTransition::Next(Stage::PrimerRename)
            }
            KeyCode::Esc | KeyCode::Char('q') => StageTransition::Quit,
            _ => StageTransition::Stay,
        }
    }

    pub fn render(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
        let vds = Vendor::all()
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>();
        let vertical = Layout::vertical([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ]);
        let horizontal = Layout::horizontal([Constraint::Percentage(33); 3]).spacing(1);
        let [header_area, main_area, _footer_area] = vertical.areas(terminal.get_frame().area());
        let header_text = format!(
            "Selected: {}",
            Vendor::from_index(self.get_highlighted())
                .map_or("None".to_string(), |v| v.to_string())
        );
        let header_widget = Paragraph::new(Line::from(vec![Span::styled(
            header_text,
            Style::default().fg(Color::Cyan),
        )]));
        let [left, middle, right] = horizontal.areas(main_area);
        terminal.draw(|f| {
            let areas = [left, middle, right];
            for (i, (title, area)) in vds.iter().zip(areas.iter()).enumerate() {
                let is_highlighted = i == self.get_highlighted();
                let style = if is_highlighted {
                    Style::default()
                        .fg(Color::Yellow)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(title.clone(), style))
                    .border_style(style);
                let block_content = Paragraph::new(title.clone())
                    .style(style)
                    .alignment(Alignment::Center)
                    .block(block);
                f.render_widget(block_content, *area);
            }
            f.render_widget(header_widget, header_area);
        })?;
        Ok(())
    }
}

impl PrimerRenameStage {
    pub fn init() -> Self {
        Self {
            rename_map: HashMap::new(),
            sanger_fns: Rc::new(Mutex::new(SangerFilenames {
                filenames: Vec::new(),
            })),
        }
    }
    pub fn new(sanger_fns: Rc<Mutex<SangerFilenames>>) -> Self {
        Self {
            rename_map: HashMap::new(),
            sanger_fns: sanger_fns.clone(),
        }
    }
    pub fn fill_names(&mut self) {
        let sanger_fns = self.sanger_fns.lock().unwrap();
        for sanger_fn in sanger_fns.filenames.iter() {
            let primer_name = sanger_fn.get_primer_name();
            self.rename_map.insert(primer_name.clone(), None);
        }
    }
    pub fn set_rename(&mut self, primer_name: String, new_name: Option<String>) {
        self.rename_map.insert(primer_name, new_name);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> StageTransition {
        if key.kind != KeyEventKind::Press {
            return StageTransition::Stay;
        }
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => StageTransition::Quit,
            // Add more key handling as needed
            _ => StageTransition::Stay,
        }
    }
    pub fn render(&self, _terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
        // Placeholder for primer rename page logic
        Ok(())
    }
}

impl DateSelectionStage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> StageTransition {
        if key.kind != KeyEventKind::Press {
            return StageTransition::Stay;
        }
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => StageTransition::Quit,
            // Add more key handling as needed
            _ => StageTransition::Stay,
        }
    }
    pub fn render(&self, _terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
        // Placeholder for date selection page logic
        Ok(())
    }
}

impl RenamePreviewStage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> StageTransition {
        if key.kind != KeyEventKind::Press {
            return StageTransition::Stay;
        }
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => StageTransition::Quit,
            // Add more key handling as needed
            _ => StageTransition::Stay,
        }
    }
    pub fn render(&self, _terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
        // Placeholder for rename preview page logic
        Ok(())
    }
}

// Enum to handle stage transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StageTransition {
    Stay,
    Next(Stage),
    Previous(Stage),
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    VendorSelection,
    PrimerRename,
    DateSelection,
    RenamePreview,
}

struct SangerFilenames {
    filenames: Vec<SangerFilename>,
}
struct StrFilenames {
    filenames: Vec<String>,
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
            date_selection: DateSelectionStage::new(),
            rename_preview: RenamePreviewStage::new(),
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
            match self.vendor_selection.selected_vendor {
                Some(Vendor::Sangon) => {
                    let fns = SangerFilename::new(filename.clone(), Vendor::Sangon);
                    self.sanger_fns.lock().unwrap().filenames.push(fns);
                }
                Some(Vendor::Ruibio) => {
                    let fns = SangerFilename::new(filename.clone(), Vendor::Ruibio);
                    self.sanger_fns.lock().unwrap().filenames.push(fns);
                }
                Some(Vendor::Genewiz) => {
                    let fns = SangerFilename::new(filename.clone(), Vendor::Genewiz);
                    self.sanger_fns.lock().unwrap().filenames.push(fns);
                }
                None => {
                    return Err(anyhow::anyhow!("No vendor selected"));
                }
            }
        }
        Ok(())
    }
    pub fn get_selected_vendor(&self) -> Option<Vendor> {
        self.vendor_selection.selected_vendor
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
                        let sanger_fns = Rc::clone(&self.sanger_fns);
                        self.primer_rename = PrimerRenameStage::new(sanger_fns);
                    }
                    Stage::DateSelection => {
                        self.date_selection = DateSelectionStage::new();
                    }
                    Stage::RenamePreview => {
                        self.rename_preview = RenamePreviewStage::new();
                    }
                    _ => {}
                }
            }
            StageTransition::Previous(stage) => self.stage = stage,
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
            Stage::RenamePreview => self.rename_preview.handle_key(key),
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
    pub fn date_selection_page(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> anyhow::Result<()> {
        self.date_selection.render(terminal)
    }
    pub fn rename_preview_page(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> anyhow::Result<()> {
        self.rename_preview.render(terminal)
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
                Stage::DateSelection => {
                    self.date_selection_page(&mut term)?;
                }
                Stage::RenamePreview => {
                    self.rename_preview_page(&mut term)?;
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

    #[test]
    fn test_add_filenames() {
        let mut app = App::new();
        let fns = vec![
            "C:\\Users\\username\\Downloads\\20250604150114670_RR7114\\报告成功\\K528-3.250604-mbp-s3.34810430.D07.seq",
            "C:\\Users\\username\\Downloads\\20250604150114670_RR7114\\报告成功\\K528-3.250604-mbp-s3.34810430.D07.seq",
        ].iter().map(|s| s.to_string()).collect::<Vec<String>>();
        app.add_filenames(fns);
        assert_eq!(app.get_filenames().len(), 2);
    }
    #[test]
    fn test_convert_filename() {
        let mut app = App::new();
        app.set_selected_vendor(Some(Vendor::Ruibio));
        let filename = "C:\\Users\\username\\Downloads\\20250604150114670_RR7114\\报告成功\\K528-3.250604-mbp-s3.34810430.D07.seq".to_string();
        app.add_filenames(vec![filename.clone()]);
        let converted = app.filenames_string_to_sanger();
        assert!(converted.is_ok());
        let sanger_fns = app.get_sanger_filenames();
        let converted_filename = sanger_fns
            .get(0)
            .expect("Expected at least one converted filename");
        assert_eq!(converted_filename.get_full_path(), filename);
        assert_eq!(
            converted_filename.get_file_stem(),
            "K528-3.250604-mbp-s3.34810430.D07"
        );
        assert_eq!(converted_filename.get_vendor_name(), "Ruibio");
    }

    #[test]
    fn test_get_primer_names() {
        let mut app = App::new();
        app.set_selected_vendor(Some(Vendor::Ruibio));
        let filename = "C:\\Users\\username\\Downloads\\20250604150114670_RR7114\\报告成功\\K528-3.250604-mbp-s3.34810430.D07.seq".to_string();
        app.add_filenames(vec![filename.clone()]);
        app.filenames_string_to_sanger().unwrap();
        app.stage = Stage::PrimerRename;
        let primer_names = app.get_all_primer_names().unwrap();
        assert_eq!(primer_names.len(), 1);
        assert_eq!(primer_names[0], "250604-mbp-s3");
    }

    #[test]
    fn test_primer_rename_fill() {
        let mut app = App::new();
        app.set_selected_vendor(Some(Vendor::Ruibio));
        let filename = "C:\\Users\\username\\Downloads\\20250604150114670_RR7114\\报告成功\\K528-3.250604-mbp-s3.34810430.D07.seq".to_string();
        app.add_filenames(vec![filename.clone()]);
        app.filenames_string_to_sanger().unwrap();
        app.handle_stage_transition(StageTransition::Next(Stage::PrimerRename));
        assert_eq!(app.stage, Stage::PrimerRename);
        app.primer_rename.fill_names();
        let primer_names = app
            .primer_rename
            .rename_map
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(primer_names.len(), 1);
        assert_eq!(primer_names[0], "250604-mbp-s3");
        // also assert the val
        assert_eq!(app.primer_rename.rename_map[&primer_names[0]], None);
    }
}
