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
    pub selected_vendor: Option<VendorSelection>,
    pub highlighted: usize,
    pub quit_without_selection: bool,
    pub stage: Stage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    VendorSelection,
    PrimerRename,
    DateSelection,
    RenamePreview,
}

impl Default for App {
    fn default() -> App {
        App {
            should_quit: false,
            selected_vendor: None,
            highlighted: 0,
            quit_without_selection: false,
            stage: Stage::VendorSelection,
        }
    }
}

impl App {
    pub fn new() -> App {
        App::default()
    }
    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Left => {
                if self.highlighted == 0 {
                    self.highlighted = 2; // Wrap around to the last vendor
                } else {
                    self.highlighted -= 1;
                }
            }
            KeyCode::Right => {
                if self.highlighted == 2 {
                    self.highlighted = 0; // Wrap around to the first vendor
                } else {
                    self.highlighted += 1;
                }
            }
            KeyCode::Enter => {
                self.selected_vendor = VendorSelection::from_index(self.highlighted);
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.should_quit = true;
            }
            _ => {}
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
        loop {
            let [header_area, main_area, footer_area] = vertical.areas(terminal.get_frame().area());
            let header_text = format!(
                "Selected: {}",
                VendorSelection::from_index(self.highlighted)
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
                    let is_highlighted = i == self.highlighted;
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
                    let vertical_layout = Layout::vertical([
                        Constraint::Min(0),
                        Constraint::Length(5),
                        Constraint::Min(0),
                    ]);
                    let block_content = Paragraph::new(title.clone())
                        .style(style)
                        .alignment(Alignment::Center)
                        .block(block);
                    f.render_widget(block_content, *area);
                }
                f.render_widget(header_widget, header_area);
            })?;
            if let Some(ev) = event::read()?.as_key_press_event() {
                self.handle_key(ev);
            };
            if self.should_quit {
                break;
            }
        }
        ratatui::restore();
        Ok(())
    }
    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut term = ratatui::init();
        self.vendor_selection_page(&mut term)?;
        Ok(())
    }
}
