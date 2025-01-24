use anyhow::Result;
use fm3_gfa::cli::{parse_cli, Commands};
use fm3_gfa::generate::generate;
use fm3_gfa::serve::serve;

fn main() -> Result<()> {
    // parse cli
    let cli = parse_cli()?;

    // match sub-command
    match cli.command {
        Commands::Generate { gfa, vcf, threads } => generate(&gfa, &vcf, threads)?,
        Commands::Serve { gfa, vcf, port } => {
            tokio::runtime::Runtime::new()?.block_on(serve(&gfa, &vcf, port))?
        }
    }

    Ok(())
}
