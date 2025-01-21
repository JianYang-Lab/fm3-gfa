use anyhow::Result;
use fm3_gfa::echart::EchartGraph;
use fm3_gfa::gml::{GMLObject, Graph};
use fm3_gfa::Layout;
use std::str::FromStr;

fn main() -> Result<()> {
    // stdin -> GML -> Graph
    let gml_content = std::fs::read_to_string("test.gml").unwrap();
    let layout = Layout::new()?;
    let res = layout.run(&gml_content)?;
    let layout_graph = GMLObject::from_str(&res)?;
    let layout_graph = Graph::from_gml(layout_graph)?;
    // println!("{:?}", layout_graph);
    // println!("{}", res);
    let echart_graph = EchartGraph::from_gml(layout_graph)?;
    eprintln!("echart_graph done");

    // 5. 保存结果
    let oneline = echart_graph.oneline_stdout()?;
    eprintln!("oneline done");

    // 6. stdout
    println!("{}", oneline);

    // let gml = GMLObject::from_str(&gml_content)?;
    // let graph = Graph::from_gml(gml)?;
    // println!("{:?}", graph);
    // // layout(&mut graph)?;
    // let mut g = graph.to_petgraph();
    // // add node attr in g
    // // 2. 配置布局选项
    // let options = LayoutOptions::new()
    //     .with_force_model(ForceModel::New)
    //     .with_edge_length(100.0)
    //     .with_iterations(100);

    // // 3. 创建并运行布局
    // let mut layout = FMMMLayout::new().with_options(options);
    // layout.run(&mut g)?;
    // // println!("{:?}", g);
    // let echart_graph = EchartGraph::from_layout_graph(&g);
    // let oneline = echart_graph.oneline_stdout()?;
    // std::fs::write("vis/test.json", oneline.as_bytes())?;

    // multilevel_layout(&mut graph, &config);

    Ok(())
}
