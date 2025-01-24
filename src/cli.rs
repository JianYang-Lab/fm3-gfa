use anyhow::Result;
use clap::{command, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "fm3-gfa")]
#[command(about = "Layout graph by FM3 algorithm")]
#[command(long_about = "long_about todo!!!")]
#[command(author, version)]
#[command(
    help_template = "{name} -- {about}\n\nVersion: {version}\n\nAuthors: {author}\
    \n\n{usage-heading} {usage}\n\n{all-args}"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate all JSONs into a tsv file with Variant ID, stdout
    Generate {
        /// Input GFA file
        #[arg(short, long, required = true, help_heading = Some("I/O"))]
        gfa: String,
        /// Input VCF file
        #[arg(short, long, required = true, help_heading = Some("I/O"))]
        vcf: String,
        /// Threads
        #[arg(default_value = "1", short = '@', long)]
        threads: usize,
    },
    /// Start a simple web server for querying and visualization
    Serve {
        /// Input GFA file
        #[arg(short, long, required = true)]
        gfa: String,
        /// Input VCF file
        #[arg(short, long, required = true)]
        vcf: String,
        /// Port number
        #[arg(short, long, default_value = "8888")]
        port: u16,
    },
}

pub fn parse_cli() -> Result<Cli> {
    let cli = Cli::parse();
    Ok(cli)
}
