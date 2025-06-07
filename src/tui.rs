use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    },
    execute,
    style::Stylize,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{self, Block, Borders, List, Paragraph},
};
use sanger_rename::SangerFilenameVaraint;
use std::io::Stdout;
use std::time::Duration;
use std::{fmt::Display, io};
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
    sangler_fns: SangerFilenames,
    sanger_fns_str: SangerFilenamesStr,
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
    filenames: Vec<SangerFilenameVaraint>,
}
struct SangerFilenamesStr {
    filenames: Vec<String>,
}

impl Default for App {
    fn default() -> App {
        App {
            should_quit: false,
            vendor_selection_state: VendorSelectionState::new(),
            quit_without_selection: false,
            stage: Stage::VendorSelection,
            sangler_fns: SangerFilenames {
                filenames: Vec::new(),
            },
            sanger_fns_str: SangerFilenamesStr {
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
        self.sanger_fns_str.filenames.push(filename);
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
