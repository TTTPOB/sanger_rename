use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{
        Block, Borders, Padding, Paragraph,
        calendar::{CalendarEventStore, Monthly},
    },
};
use std::{io::Stdout, rc::Rc, sync::Mutex};
use time::ext::NumericalDuration;
use time::{Date, Month, OffsetDateTime};

use crate::tui::{App, SangerFilenames};

use super::common::StageTransition;

pub struct ConfirmRenameStage {
    pub selected_date: Date,
    pub sanger_fns: Rc<Mutex<SangerFilenames>>,
}

impl ConfirmRenameStage {
    pub fn init() -> Self {
        Self {
            selected_date: OffsetDateTime::now_local().unwrap().date(),
            sanger_fns: Rc::new(Mutex::new(SangerFilenames {
                filenames: Vec::new(),
            })),
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
            KeyCode::Enter => {
                //for all fn set the date
                for sanger_fn in self.sanger_fns.lock().unwrap().filenames.iter_mut() {
                    sanger_fn.set_date(self.selected_date);
                }
                StageTransition::Stay // You can change this to move to next stage if needed
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.selected_date -= 1.days();
                StageTransition::Stay
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.selected_date += 1.weeks();
                StageTransition::Stay
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_date -= 1.weeks();
                StageTransition::Stay
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.selected_date += 1.days();
                StageTransition::Stay
            }
            KeyCode::Char('n') | KeyCode::Tab => {
                self.selected_date = self.next_month(self.selected_date);
                StageTransition::Stay
            }
            KeyCode::Char('p') | KeyCode::BackTab => {
                self.selected_date = self.prev_month(self.selected_date);
                StageTransition::Previous(super::Stage::TemplateRename)
            }
            _ => StageTransition::Stay,
        }
    }

    fn next_month(&self, date: Date) -> Date {
        if date.month() == Month::December {
            date.replace_month(Month::January)
                .unwrap()
                .replace_year(date.year() + 1)
                .unwrap()
        } else {
            date.replace_month(date.month().next()).unwrap()
        }
    }

    fn prev_month(&self, date: Date) -> Date {
        if date.month() == Month::January {
            date.replace_month(Month::December)
                .unwrap()
                .replace_year(date.year() - 1)
                .unwrap()
        } else {
            date.replace_month(date.month().previous()).unwrap()
        }
    }

    fn create_events(&self) -> anyhow::Result<CalendarEventStore> {
        const SELECTED: Style = Style::new()
            .fg(Color::White)
            .bg(Color::Red)
            .add_modifier(Modifier::BOLD);

        let mut list = CalendarEventStore::today(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Blue),
        );

        // Mark the selected date
        list.add(self.selected_date, SELECTED);

        Ok(list)
    }

    pub fn render(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
        let events = self.create_events()?;

        terminal.draw(|frame| {
            let chunks =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(frame.area());

            // Render the three-month calendar on the left
            self.render_notice(frame, chunks[0], &events);

            App::render_rename_preview_table(frame, chunks[1], &self.sanger_fns);
        })?;

        Ok(())
    }

    fn render_notice(&self, frame: &mut Frame, area: Rect, events: &CalendarEventStore) {
        let block = Block::default()
            .title("Confirm Rename")
            .title_alignment(ratatui::layout::Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .padding(Padding::new(0, 0, area.height / 3, 0));
        let p = Paragraph::new(Text::from(Line::from(
            "Press 'Shift + Enter' to confirm renaming",
        )))
        .block(block)
        .alignment(Alignment::Center);

        frame.render_widget(p, area);
    }
}
