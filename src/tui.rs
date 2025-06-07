use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use sanger_rename::{
    SangerFilenameVariant, vendors::genewiz::GenewizSangerFilename,
    vendors::ruibio::RuibioSangerFilename, vendors::sangon::SangonSangerFilename,
};
use std::fmt::Display;
use std::io::Stdout;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum VendorSelection {
    Sangon,
    Ruibio,
    Genewiz,
}

impl VendorSelection {
    pub fn all() -> Vec<VendorSelection> {
        VendorSelection::iter().collect()
    }
    pub fn from_index(index: usize) -> Option<VendorSelection> {
        Self::all().get(index).copied()
    }
}

impl Display for VendorSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VendorSelection::Sangon => write!(f, "Sangon"),
            VendorSelection::Ruibio => write!(f, "Ruibio"),
            VendorSelection::Genewiz => write!(f, "Genewiz"),
        }
    }
}

pub struct App {
    pub should_quit: bool,
    vendor_selection_state: VendorSelectionState,
    pub quit_without_selection: bool,
    pub stage: Stage,
    sanger_fns: SangerFilenames,
    str_fns: StrFilenames,
}
struct VendorSelectionState {
    highlighted: usize,
    selected_vendor: Option<VendorSelection>,
}
impl VendorSelectionState {
    pub fn new() -> Self {
        Self {
            highlighted: 0,
            selected_vendor: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    VendorSelection,
    PrimerRename,
    DateSelection,
    RenamePreview,
}

struct SangerFilenames {
    filenames: Vec<SangerFilenameVariant>,
}
struct StrFilenames {
    filenames: Vec<String>,
}

impl Default for App {
    fn default() -> App {
        App {
            should_quit: false,
            vendor_selection_state: VendorSelectionState::new(),
            quit_without_selection: false,
            stage: Stage::VendorSelection,
            sanger_fns: SangerFilenames {
                filenames: Vec::new(),
            },
            str_fns: StrFilenames {
                filenames: Vec::new(),
            },
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
    pub fn get_sanger_filenames(&self) -> &Vec<SangerFilenameVariant> {
        &self.sanger_fns.filenames
    }
    pub fn filenames_string_to_sanger(&mut self) -> anyhow::Result<()> {
        for filename in &self.str_fns.filenames {
            match self.vendor_selection_state.selected_vendor {
                Some(VendorSelection::Sangon) => {
                    let fns = SangonSangerFilename::from(filename.clone());
                    self.sanger_fns
                        .filenames
                        .push(SangerFilenameVariant::Sangon(fns));
                }
                Some(VendorSelection::Ruibio) => {
                    let fns = RuibioSangerFilename::from(filename.clone());
                    self.sanger_fns
                        .filenames
                        .push(SangerFilenameVariant::Ruibio(fns));
                }
                Some(VendorSelection::Genewiz) => {
                    let fns = GenewizSangerFilename::from(filename.clone());
                    self.sanger_fns
                        .filenames
                        .push(SangerFilenameVariant::Genewiz(fns));
                }
                None => {
                    return Err(anyhow::anyhow!("No vendor selected"));
                }
            }
        }
        Ok(())
    }
    pub fn get_selected_vendor(&self) -> Option<VendorSelection> {
        self.vendor_selection_state.selected_vendor
    }
    pub fn set_vendor_highlighted(&mut self, index: usize) {
        if index < VendorSelection::all().len() {
            self.vendor_selection_state.highlighted = index;
        }
    }
    pub fn get_vendor_highlighted(&self) -> usize {
        self.vendor_selection_state.highlighted
    }
    pub fn set_selected_vendor(&mut self, vendor: Option<VendorSelection>) {
        self.vendor_selection_state.selected_vendor = vendor;
    }
    pub fn handle_key_vendor_selection(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Left => {
                if self.get_vendor_highlighted() == 0 {
                    self.set_vendor_highlighted(VendorSelection::all().len() - 1); // Wrap around to the last vendor
                } else {
                    self.set_vendor_highlighted(self.get_vendor_highlighted() - 1);
                }
            }
            KeyCode::Right => {
                if self.get_vendor_highlighted() == VendorSelection::all().len() - 1 {
                    self.set_vendor_highlighted(0); // Wrap around to the first vendor
                } else {
                    self.set_vendor_highlighted(self.get_vendor_highlighted() + 1);
                }
            }
            KeyCode::Enter => {
                self.set_selected_vendor(VendorSelection::from_index(
                    self.get_vendor_highlighted(),
                ));
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.should_quit = true;
            }
            _ => {}
        }
    }
    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match self.stage {
            Stage::VendorSelection => self.handle_key_vendor_selection(key),
            Stage::PrimerRename => {
                // Handle keys for primer rename stage
            }
            Stage::DateSelection => {
                // Handle keys for date selection stage
            }
            Stage::RenamePreview => {
                // Handle keys for rename preview stage
            }
        }
    }
    pub fn vendor_selection_page(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> anyhow::Result<()> {
        let vds = VendorSelection::all()
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>();
        let vertical = Layout::vertical([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ]);
        let horizontal = Layout::horizontal([Constraint::Percentage(33); 3]).spacing(1);
        let [header_area, main_area, footer_area] = vertical.areas(terminal.get_frame().area());
        let header_text = format!(
            "Selected: {}",
            VendorSelection::from_index(self.get_vendor_highlighted())
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
                let is_highlighted = i == self.get_vendor_highlighted();
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
    pub fn primer_rename_page(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> anyhow::Result<()> {
        // Placeholder for primer rename page logic
        Ok(())
    }
    pub fn date_selection_page(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> anyhow::Result<()> {
        // Placeholder for date selection page logic
        Ok(())
    }
    pub fn rename_preview_page(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> anyhow::Result<()> {
        // Placeholder for rename preview page logic
        Ok(())
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
    use sanger_rename::SangerFilename;

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
        app.set_selected_vendor(Some(VendorSelection::Ruibio));
        let filename = "C:\\Users\\username\\Downloads\\20250604150114670_RR7114\\报告成功\\K528-3.250604-mbp-s3.34810430.D07.seq".to_string();
        app.add_filenames(vec![filename.clone()]);
        let converted = app.filenames_string_to_sanger();
        assert!(converted.is_ok());
        let converted_filename = app
            .get_sanger_filenames()
            .get(0)
            .expect("Expected at least one converted filename");
        match converted_filename {
            SangerFilenameVariant::Ruibio(r) => {
                assert_eq!(r.get_full_path(), filename);
                assert_eq!(r.get_file_stem(), "K528-3.250604-mbp-s3.34810430.D07");
            }
            _ => panic!("Expected RuibioSangerFilename variant"),
        }
    }
}
