use crate::gml::Graph;
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
    // // 添加从布局图转换的新方法
    // pub fn from_layout_graph(graph: &PetGraph<NodeAttributes, EdgeAttributes, Undirected>) -> Self {
    //     let mut nodes = Vec::new();
    //     let mut links = Vec::new();

    //     // 转换节点
    //     for node_idx in graph.node_indices() {
    //         let node_attrs = &graph[node_idx];
    //         nodes.push(Node {
    //             id: node_idx.index() as i64, // 使用节点索引作为ID
    //             x: node_attrs.position.x,
    //             y: node_attrs.position.y,
    //             name: node_idx.index().to_string(),
    //         });
    //     }

    //     // 转换边
    //     for edge_ref in graph.edge_references() {
    //         links.push(Link {
    //             source: edge_ref.source().index() as i64,
    //             target: edge_ref.target().index() as i64,
    //         });
    //     }

    //     EchartGraph { nodes, links }
    // }
    pub fn from_gml(layout_graph: Graph) -> Result<Self> {
        // let layout_gml = GMLObject::from_str(layout_result)?;
        // let layout_graph = Graph::from_gml(layout_gml)?;

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
