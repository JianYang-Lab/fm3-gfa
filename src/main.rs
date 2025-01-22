use std::str::FromStr;

use anyhow::Result;
// use fm3_gfa::echart::EchartGraph;
// use fm3_gfa::gml::{GMLObject, Graph};
use fm3_gfa::layout::Layout;
use fm3_gfa::{
    bfs::extract_subgraph_by_bfs,
    echart::EchartGraph,
    gfa::gfa_to_graph,
    gml::{GMLGraph, GMLObject},
    vcf::parse_vcf_file,
};
use indicatif::ProgressBar;

// use fm3_gfa::Layout;
// use std::str::FromStr;

fn main() -> Result<()> {
    // GFA/VCF => Sub-GFA => GML => Layout(GML) => Echart

    let g = gfa_to_graph("tests/test.gfa")?;

    let bubbles = parse_vcf_file("tests/tmp.vcf")?;
    let bar = ProgressBar::new(bubbles.len() as u64);
    for bubble in bubbles {
        let sub_graph = extract_subgraph_by_bfs(&bubble, &g)?;
        let gml_content = sub_graph.to_gml_string();
        let origin_g = GMLObject::from_str(&gml_content)?;
        let origin_g = GMLGraph::from_gml(origin_g)?;
        let layout = Layout::new()?;
        let res = layout.run(&gml_content)?;
        let layout_graph = GMLObject::from_str(&res)?;
        let layout_graph = GMLGraph::from_gml(layout_graph)?;
        let echart_graph = EchartGraph::from_gml_anno(layout_graph, origin_g)?;
        let oneline = echart_graph.oneline_stdout()?;
        println!("{}", oneline);
        bar.inc(1);
    }

    Ok(())
}
