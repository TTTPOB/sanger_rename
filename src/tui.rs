use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
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
}

// Stage-specific structs
struct VendorSelectionStage {
    highlighted: usize,
    selected_vendor: Option<Vendor>,
}

struct PrimerRenameStage {
    sanger_fns: Rc<Mutex<SangerFilenames>>,
    rename_map: HashMap<String, Option<String>>,
    highlighted: usize,
    editing: bool,
    current_input: String,
}

struct DateSelectionStage {
    // Add fields specific to date selection stage
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
            highlighted: 0,
            editing: false,
            current_input: String::new(),
        }
    }
    pub fn from_sanger_fns(sanger_fns: Rc<Mutex<SangerFilenames>>) -> Self {
        let mut s = Self::init();
        s.sanger_fns = sanger_fns.clone();
        s.fill_names();
        s
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

        if self.editing {
            match key.code {
                KeyCode::Enter => {
                    // Save the current input as the new name
                    let primer_names: Vec<String> = self.rename_map.keys().cloned().collect();
                    if let Some(primer_name) = primer_names.get(self.highlighted) {
                        let new_name = if self.current_input.is_empty() {
                            None
                        } else {
                            Some(self.current_input.clone())
                        };
                        self.set_rename(primer_name.clone(), new_name);
                    }
                    self.editing = false;
                    self.current_input.clear();
                    StageTransition::Stay
                }
                KeyCode::Esc => {
                    self.editing = false;
                    self.current_input.clear();
                    StageTransition::Stay
                }
                KeyCode::Backspace => {
                    self.current_input.pop();
                    StageTransition::Stay
                }
                KeyCode::Char(c) => {
                    self.current_input.push(c);
                    StageTransition::Stay
                }
                _ => StageTransition::Stay,
            }
        } else {
            match key.code {
                KeyCode::Up => {
                    if self.highlighted > 0 {
                        self.highlighted -= 1;
                    }
                    StageTransition::Stay
                }
                KeyCode::Down => {
                    if self.highlighted < self.rename_map.len().saturating_sub(1) {
                        self.highlighted += 1;
                    }
                    StageTransition::Stay
                }
                KeyCode::Enter => {
                    self.editing = true;
                    // Pre-fill with existing name if any
                    let primer_names: Vec<String> = self.rename_map.keys().cloned().collect();
                    if let Some(primer_name) = primer_names.get(self.highlighted) {
                        if let Some(existing_name) = &self.rename_map[primer_name] {
                            self.current_input = existing_name.clone();
                        }
                    }
                    for sanger_fn in self.sanger_fns.lock().unwrap().filenames.iter() {
                        if let Some(primer_name) = self.rename_map.get(&sanger_fn.get_primer_name())
                        {
                            if primer_name.is_some() {
                                self.current_input = primer_name.clone().unwrap_or_default();
                            }
                        }
                    }
                    StageTransition::Stay
                }
                KeyCode::Esc | KeyCode::Char('q') => StageTransition::Quit,
                KeyCode::Tab => StageTransition::Next(Stage::DateSelection),
                _ => StageTransition::Stay,
            }
        }
    }
    pub fn render(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
        let primer_names: Vec<String> = self.rename_map.keys().cloned().collect();

        terminal.draw(|f| {
            let chunks = Layout::horizontal([
                Constraint::Percentage(50), // Left panel: Primer names with rename inputs
                Constraint::Percentage(50), // Right panel: Rename preview table
            ])
            .split(f.area());

            // Left panel: Primer names with rename inputs
            let left_items: Vec<Line> = primer_names
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let is_highlighted = i == self.highlighted;
                    let new_name = self.rename_map.get(name).and_then(|n| n.as_ref());

                    let display_text = if self.editing && is_highlighted {
                        format!("{} -> {}_", name, self.current_input)
                    } else if let Some(new_name) = new_name {
                        format!("{} -> {}", name, new_name)
                    } else {
                        format!("{} -> <not set>", name)
                    };

                    let style = if is_highlighted {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    Line::from(vec![
                        Span::styled(if is_highlighted { ">> " } else { "   " }, style),
                        Span::styled(display_text, style),
                    ])
                })
                .collect();

            let left_block = Block::default()
                .borders(Borders::ALL)
                .title("Primer Names (Enter to edit, Tab to continue)");
            let primer_rename_view = Paragraph::new(left_items)
                .block(left_block)
                .wrap(ratatui::widgets::Wrap { trim: true });

            let header = Row::new(["Original", "Standardized"])
                .style(Style::default().add_modifier(Modifier::BOLD));
            let mut rows = vec![];
            for sf in self.sanger_fns.lock().unwrap().filenames.iter() {
                let original_name = sf.show_file_name();
                let standardized_name = format!("{}.ab1", sf.get_standardized_name());
                rows.push(Row::new([original_name, standardized_name]));
            }

            let right_block = Block::default()
                .borders(Borders::ALL)
                .title("Rename Preview");
            let table_width = [Constraint::Percentage(50), Constraint::Percentage(50)];
            let rename_preview_view = Table::new(rows, table_width)
                .header(header)
                .block(right_block);

            f.render_widget(primer_rename_view, chunks[0]);
            f.render_widget(rename_preview_view, chunks[1]);
        })?;

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
                        self.filenames_string_to_sanger().unwrap();
                        let sanger_fns = Rc::clone(&self.sanger_fns);
                        self.primer_rename = PrimerRenameStage::from_sanger_fns(sanger_fns);
                    }
                    Stage::DateSelection => {
                        self.date_selection = DateSelectionStage::new();
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
        assert_eq!(sanger_fns.len(), 5);
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
        assert_eq!(primer_names.len(), 5);
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
        app.primer_rename.fill_names();
        let primer_names = app
            .primer_rename
            .rename_map
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(primer_names.len(), 5);
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
        assert!(primer_names.contains(&"C1_R".to_string()));
    }
}
