use clap::Parser;

mod tui;
use tui::run_tui;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let selected_vendor = if args.interactive || args.filenames.is_empty() {
        // Show TUI if interactive mode is enabled or no files are specified
        match run_tui()? {
            Some(vendor) => {
                println!("Selected vendor: {}", vendor.as_str());
                vendor
            }
            None => {
                println!("No vendor selected. Exiting.");
                return Ok(());
            }
        }
    } else {
        // Default behavior or could prompt for vendor selection
        println!("Use --interactive or -i flag to select vendor interactively.");
        return Ok(());
    };

    if args.verbose {
        println!(
            "Processing {} files with {} vendor",
            args.filenames.len(),
            selected_vendor.as_str()
        );
    }

    for filename in &args.filenames {
        println!(
            "Processing file: {} with vendor: {}",
            filename,
            selected_vendor.as_str()
        );
        // TODO: Implement actual file processing logic based on selected vendor
    }

    if args.filenames.is_empty() && !args.interactive {
        println!("No files specified. Use --help for usage information.");
        println!("Use --interactive or -i flag to select vendor interactively.");
    }

    Ok(())
}
