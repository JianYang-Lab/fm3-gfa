use anyhow::Result;
use clap::{command, Parser};

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
    /// Input GFA file
    #[arg(short, long, required = true, help_heading = Some("I/O"))]
    pub gfa: String,
    /// Input VCF file, gzipped is supported
    #[arg(short, long, required = true, help_heading = Some("I/O"))]
    pub vcf: String,
    /// Output json file, None for stdout
    #[arg(short, long, required = false, help_heading = Some("I/O"))]
    pub output: Option<String>,

    /// Threads
    #[arg(default_value = "1", short = '@', long, help_heading = Some("Options"))]
    pub threads: usize,
    /// Reference Name, if `W` is available in GFA[WIP]
    #[arg(short, long, help_heading = Some("Options"))]
    pub ref_name: Option<String>,
}

pub fn parse_cli() -> Result<Cli> {
    let cli = Cli::parse();
    Ok(cli)
}
