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
    value: MyAttr,
}

#[derive(Serialize, Deserialize, Clone)]
struct MyAttr {
    #[serde(rename = "Sequence")]
    sequence: String,
    #[serde(rename = "Status")]
    status: String,
}

impl Default for MyAttr {
    fn default() -> Self {
        MyAttr {
            sequence: "".to_string(),
            status: "".to_string(),
        }
    }
}

impl MyAttr {
    pub fn new(sequence: String, status: String) -> Self {
        MyAttr { sequence, status }
    }
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

            nodes.push(Node {
                id,
                x,
                y,
                name,
                value: MyAttr::default(),
            });
        }
        for edge in layout_graph.edges.iter() {
            links.push(Link {
                source: edge.source,
                target: edge.target,
            });
        }
        Ok(EchartGraph { nodes, links })
    }

    pub fn from_gml_anno(layout_g: GMLGraph, origin_g: GMLGraph) -> Result<Self> {
        // the order of nodes in layout_g and origin_g is the same

        let mut nodes = Vec::new();
        let mut links = Vec::new();

        for i in 0..layout_g.nodes.len() {
            let layout_node = &layout_g.nodes[i];
            let origin_node = &origin_g.nodes[i];
            let id = layout_node.id;
            let name = origin_node.label.clone().unwrap();
            let (x, y) = layout_node.get_graphic_pos();

            let sequence = origin_node.get_sequence();
            let status = origin_node.get_status();
            nodes.push(Node {
                id,
                x,
                y,
                name,
                value: MyAttr::new(sequence, status),
            });
        }
        for edge in layout_g.edges.iter() {
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
