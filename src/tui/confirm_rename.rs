use crate::tui::{App, SangerFilenames};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Padding, Paragraph},
};
use std::{io::Stdout, rc::Rc, sync::Mutex};

use super::common::StageTransition;

pub struct ConfirmRenameStage {
    pub renamed: bool,
    pub sanger_fns: Rc<Mutex<SangerFilenames>>,
}

impl ConfirmRenameStage {
    pub fn init() -> Self {
        Self {
            renamed: false,
            sanger_fns: Rc::new(Mutex::new(SangerFilenames::new())),
        }
    }
    pub fn from_sanger_fns(sanger_fns: Rc<Mutex<SangerFilenames>>) -> Self {
        let mut stage = Self::init();
        stage.sanger_fns = sanger_fns.clone();
        stage
    }
    pub fn handle_key(&mut self, key: KeyEvent) -> StageTransition {
        if key.kind != KeyEventKind::Press {
            return StageTransition::Stay;
        }
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => StageTransition::Quit,
            // shift + enter to confirm renaming
            KeyCode::Enter => {
                for sanger_fn in self.sanger_fns.lock().unwrap().filenames.iter() {
                    sanger_fn.move_to_standardized_name().unwrap();
                }
                self.renamed = true;
                StageTransition::Stay
            }
            KeyCode::Char('p') | KeyCode::BackTab => {
                StageTransition::Previous(super::Stage::TemplateRename)
            }
            _ => StageTransition::Stay,
        }
    }
    pub fn render(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
        terminal.draw(|frame| {
            let chunks =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(frame.area());

            // Render the three-month calendar on the left
            self.render_notice(frame, chunks[0]);

            App::render_rename_preview_table(frame, chunks[1], &self.sanger_fns);
        })?;

        Ok(())
    }

    fn render_notice(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Confirm Rename")
            .title_alignment(ratatui::layout::Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .padding(Padding::new(0, 0, area.height / 3, 0));
        let content = if self.renamed {
            "Renaming completed successfully! Press 'q' to exit."
        } else {
            "Press 'Enter' to confirm renaming"
        };
        let p = Paragraph::new(Text::from(Line::from(content)))
            .block(block)
            .alignment(Alignment::Center);

        frame.render_widget(p, area);
    }
}
