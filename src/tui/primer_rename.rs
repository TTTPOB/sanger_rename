use crate::tui::App;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table},
};
use std::io::Stdout;
use std::sync::Mutex;
use std::{collections::HashMap, rc::Rc};

use super::common::{SangerFilenames, Stage, StageTransition};

pub struct PrimerRenameStage {
    pub sanger_fns: Rc<Mutex<SangerFilenames>>,
    pub rename_map: HashMap<String, Option<String>>,
    pub highlighted: usize,
    pub editing: bool,
    pub current_input: String,
}

impl PrimerRenameStage {
    pub fn init() -> Self {
        Self {
            rename_map: HashMap::new(),
            sanger_fns: Rc::new(Mutex::new(SangerFilenames::new())),
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
                    for sanger_fn in self.sanger_fns.lock().unwrap().filenames.iter_mut() {
                        let old_primer_name = sanger_fn.get_primer_name();
                        if let Some(new_name) = self.rename_map.get(&old_primer_name) {
                            if let Some(new_name_str) = new_name {
                                sanger_fn.set_primer_name(new_name_str).unwrap();
                            }
                        }
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
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.highlighted > 0 {
                        self.highlighted -= 1;
                    }
                    StageTransition::Stay
                }
                KeyCode::Down | KeyCode::Char('j') => {
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
                    StageTransition::Stay
                }
                KeyCode::Esc | KeyCode::Char('q') => StageTransition::Quit,
                KeyCode::Tab | KeyCode::Char('n') => StageTransition::Next(Stage::TemplateRename),
                KeyCode::BackTab | KeyCode::Char('p') => {
                    StageTransition::Previous(Stage::VendorSelection)
                }
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
            let left_rows = primer_names
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let is_highlighted = i == self.highlighted;
                    let new_name = self.rename_map.get(name).and_then(|n| n.as_ref());
                    let current_input_display = if self.editing && is_highlighted {
                        format!("{}_", self.current_input)
                    } else {
                        new_name.map_or("<not set>".to_string(), |n| n.clone())
                    };
                    let row_content = [name.clone(), "-->".to_string(), current_input_display];
                    
                    Row::new(row_content).style(if is_highlighted {
                        Style::default()
                            .bg(Color::DarkGray)
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    })
                })
                .collect::<Vec<_>>();

            let left_table_width = [
                Constraint::Percentage(45),
                Constraint::Percentage(10),
                Constraint::Percentage(45),
            ];

            let left_block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Cyan))
                .title("Primer Names (Enter to edit, Tab to continue)")
                .title_alignment(Alignment::Center);
            let left_header = Row::new(["Primer Name", "-->", "New Name"])
                .style(Style::default().add_modifier(Modifier::BOLD));
            let primer_rename_view = Table::new(left_rows, left_table_width)
                .header(left_header)
                .block(left_block);
            f.render_widget(primer_rename_view, chunks[0]);
            App::render_rename_preview_table(f, chunks[1], &self.sanger_fns);
        })?;

        Ok(())
    }
}
