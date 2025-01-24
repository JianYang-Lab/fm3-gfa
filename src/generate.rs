// src/generate.rs
use anyhow::Result;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::{str::FromStr, time::Duration};

use crate::{
    bfs::extract_subgraph_by_bfs,
    echart::EchartGraph,
    gfa::gfa_to_graph,
    gml::{GMLGraph, GMLObject},
    layout::Layout,
    vcf::parse_vcf_file,
};

pub fn generate(gfa_path: &str, vcf_path: &str, threads: usize) -> Result<()> {
    // load gfa file
    let spinner = ProgressBar::new_spinner();
    spinner.set_message("Loading GFA file");
    spinner.enable_steady_tick(Duration::from_millis(100));
    let whole_gfa = gfa_to_graph(gfa_path)?;
    spinner.finish();
    let spinner = ProgressBar::new_spinner();

    // load vcf file
    spinner.set_message("Loading VCF file");
    spinner.enable_steady_tick(Duration::from_millis(100));
    let bubbles = parse_vcf_file(vcf_path)?;
    spinner.finish();

    // set progress bar style
    let style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")?;

    // Build thread pool globally
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()?;

    // parallel process
    bubbles
        .into_par_iter()
        .progress_with_style(style)
        .for_each(|bubble| {
            match (|| -> Result<String> {
                // extract subgraph by bfs
                let sub_graph = extract_subgraph_by_bfs(&bubble, &whole_gfa)?;
                // GFAGraph to GMLGraph
                let gml_content = sub_graph.to_gml_string();
                let origin_g = GMLObject::from_str(&gml_content)?;
                let origin_g = GMLGraph::from_gml(origin_g)?;
                // layout by FM3
                let layout = Layout::new()?;
                let res = layout.run(&gml_content)?;
                let layout_graph = GMLObject::from_str(&res)?;
                let layout_graph = GMLGraph::from_gml(layout_graph)?;
                let echart_graph = EchartGraph::from_gml_anno(layout_graph, origin_g)?;
                echart_graph.oneline_stdout()
            })() {
                Ok(line) => println!("{}\t{}", bubble.id, line),
                Err(e) => eprintln!("Error processing bubble: {}", e),
            }
        });

    Ok(())
}
