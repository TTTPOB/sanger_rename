use clap::Parser;

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
}

fn main() {
    let args = Args::parse();

    if args.verbose {
        println!("Processing {} files", args.filenames.len());
    }

    for filename in &args.filenames {
        println!("Processing file: {}", filename);
    }

    if args.filenames.is_empty() {
        println!("No files specified. Use --help for usage information.");
    }
}
