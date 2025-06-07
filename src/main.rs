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
    app.add_filenames(args.filenames);
    println!("Selected vendor: {:?}", app.get_selected_vendor());
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

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
}
