use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use sanger_rename::Vendor;
use std::io::Stdout;

use super::VendorExt;
use super::common::{Stage, StageTransition};

pub struct VendorSelectionStage {
    pub highlighted: usize,
    pub selected_vendor: Option<Vendor>,
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
            KeyCode::Left | KeyCode::Char('h') => {
                if self.get_highlighted() == 0 {
                    self.set_highlighted(Vendor::all().len() - 1); // Wrap around to the last vendor
                } else {
                    self.set_highlighted(self.get_highlighted() - 1);
                }
                StageTransition::Stay
            }
            KeyCode::Right | KeyCode::Char('l') => {
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
