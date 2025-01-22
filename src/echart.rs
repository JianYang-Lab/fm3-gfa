use crate::gml::GMLGraph;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::Path;

#[derive(Serialize)]
pub struct EchartGraph {
    nodes: Vec<Node>,
    links: Vec<Link>,
}

#[derive(Serialize)]
struct Node {
    id: i64,
    x: f64,
    y: f64,
    name: String,
    // value: MyAttr,
}

#[derive(Serialize, Deserialize)]
struct Link {
    source: i64,
    target: i64,
}

impl EchartGraph {
    pub fn from_gml(layout_graph: GMLGraph) -> Result<Self> {
        let mut nodes = Vec::new();
        let mut links = Vec::new();

        for node in layout_graph.nodes.iter() {
            let id = node.id;
            let name = id.to_string();
            let (x, y) = node.get_graphic_pos();

            nodes.push(Node { id, x, y, name });
        }
        for edge in layout_graph.edges.iter() {
            links.push(Link {
                source: edge.source,
                target: edge.target,
            });
        }
        Ok(EchartGraph { nodes, links })
    }

    pub fn oneline_stdout(&self) -> Result<String> {
        let json = serde_json::to_string(self)?;
        Ok(json)
    }
    pub fn to_json<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }
}
