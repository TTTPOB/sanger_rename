use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::io;
use std::time::Duration;

pub enum VendorSelection {
    Sangon,
    Ruibio,
    Genewiz,
}

impl VendorSelection {
    pub fn as_str(&self) -> &str {
        match self {
            VendorSelection::Sangon => "Sangon",
            VendorSelection::Ruibio => "Ruibio",
            VendorSelection::Genewiz => "Genewiz",
        }
    }
}

pub struct App {
    pub should_quit: bool,
    pub selected_vendor: Option<VendorSelection>,
    pub highlighted: usize,
    pub quit_without_selection: bool,
}

impl Default for App {
    fn default() -> App {
        App {
            should_quit: false,
            selected_vendor: None,
            highlighted: 0,
            quit_without_selection: false,
        }
    }
}

impl App {
    pub fn new() -> App {
        App::default()
    }
    pub fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.quit_without_selection = true;
                self.should_quit = true;
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.selected_vendor = Some(VendorSelection::Sangon);
                self.should_quit = true;
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.selected_vendor = Some(VendorSelection::Ruibio);
                self.should_quit = true;
            }
            KeyCode::Char('g') | KeyCode::Char('G') => {
                self.selected_vendor = Some(VendorSelection::Genewiz);
                self.should_quit = true;
            }
            KeyCode::Up => {
                if self.highlighted > 0 {
                    self.highlighted -= 1;
                }
            }
            KeyCode::Down => {
                if self.highlighted < 2 {
                    self.highlighted += 1;
                }
            }
            KeyCode::Enter => {
                match self.highlighted {
                    0 => self.selected_vendor = Some(VendorSelection::Sangon),
                    1 => self.selected_vendor = Some(VendorSelection::Ruibio),
                    2 => self.selected_vendor = Some(VendorSelection::Genewiz),
                    _ => {}
                }
                self.should_quit = true;
            }
            _ => {}
        }
    }
}

pub fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Sanger Rename Tool")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Vendor selection area
    let vendor_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
        ])
        .split(chunks[1]);

    // Subtitle
    let subtitle = Paragraph::new("Choose the vendor:")
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(subtitle, vendor_chunks[0]);

    let vendors = [
        VendorSelection::Sangon,
        VendorSelection::Ruibio,
        VendorSelection::Genewiz,
    ];
    let keys = ['S', 'R', 'G'];

    for (i, (vendor, key)) in vendors.iter().zip(keys.iter()).enumerate() {
        let is_highlighted = i == app.highlighted;
        let style = if is_highlighted {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let border_style = if is_highlighted {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };

        let content = vec![
            Line::from(vec![Span::styled(
                format!("{}", vendor.as_str()),
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Press '"),
                Span::styled(
                    format!("{}", key),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("' to select"),
            ]),
            Line::from(""),
            Line::from(vec![Span::raw("↑/↓ to navigate, Enter to select")]),
        ];

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(format!(" {} ", vendor.as_str()));

        let paragraph = Paragraph::new(content)
            .style(style)
            .alignment(Alignment::Center)
            .block(block);

        f.render_widget(paragraph, vendor_chunks[i + 1]);
    }

    // Instructions
    let instructions = Paragraph::new("Press 'Q' or 'Esc' to quit")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, chunks[2]);
}

pub fn run_tui() -> Result<Option<VendorSelection>, Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    match res {
        Ok(_) => {
            if app.quit_without_selection {
                Ok(None)
            } else {
                Ok(app.selected_vendor)
            }
        }
        Err(err) => Err(Box::new(err)),
    }
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        // Poll for events with a timeout to prevent rapid redraws
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key.code);
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
