use clap::Parser;

mod tui;
use tui::App;

#[derive(Parser)]
#[command(name = "sanger-rename")]
#[command(about = "A tool for renaming files")]
struct Args {
    /// List of filenames to process
    #[arg(value_name = "FILE")]
    filenames: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut app = App::new();
    app.add_filenames(args.filenames); // Add filenames BEFORE running TUI
    app.run()?;
    Ok(())
}
