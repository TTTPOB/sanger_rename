use clap::Parser;

mod tui;
use sanger_rename::tui::App;

#[derive(Parser)]
#[command(name = "sanger-rename")]
#[command(about = "A tool for renaming files")]
struct Args {
    /// List of filenames to process
    #[arg(value_name = "FILE")]
    filenames: Vec<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Interactive mode - show TUI for vendor selection
    #[arg(short, long)]
    interactive: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut app = App::new();
    app.run()?;
    println!("Selected vendor: {:?}", app.selected_vendor);
    Ok(())
}
