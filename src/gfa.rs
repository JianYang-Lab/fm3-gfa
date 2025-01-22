use anyhow::Result;
use gfa::{
    gfa::{SegmentId, GFA},
    parser::*,
};
use petgraph::{
    graph::{Graph, NodeIndex, UnGraph},
    visit::EdgeRef,
};
use std::collections::{HashMap, HashSet};
// use petgraph::visit::NodeIndexable;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeData {
    pub id: Vec<u8>,
    pub sequence: String,
    // REF or ALT
    pub status: String,
}
impl NodeData {
    fn default() -> NodeData {
        NodeData {
            id: Vec::new(),
            sequence: String::new(),
            status: String::new(),
        }
    }
}

#[derive(Debug)]
// GFAGraph stores the node name as String
pub struct GFAGraph {
    // store node id
    pub inner_graph: UnGraph<Vec<u8>, ()>,
    // store node attrs
    node_attrs: Vec<NodeData>,
    // id to index map
    id_to_idx: HashMap<Vec<u8>, NodeIndex>,
}

impl Default for GFAGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl GFAGraph {
    // add node in graph
    pub fn add_node(&mut self, id: Vec<u8>, sequence: String, status: String) -> Result<NodeIndex> {
        let node_idx = self.inner_graph.add_node(id.clone());
        let node_data = NodeData {
            id: id.clone(),
            sequence,
            status,
        };

        while self.node_attrs.len() <= node_idx.index() {
            self.node_attrs.push(NodeData::default());
        }
        self.node_attrs[node_idx.index()] = node_data;
        self.id_to_idx.insert(id, node_idx);

        Ok(node_idx)
    }

    // add edge in graph
    pub fn add_edge(&mut self, from: &[u8], to: &[u8]) -> Result<()> {
        if let (Some(from_idx), Some(to_idx)) = (self.get_node_idx(from), self.get_node_idx(to)) {
            self.inner_graph.add_edge(from_idx, to_idx, ());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Node not found"))
        }
    }
    pub fn new() -> Self {
        Self {
            inner_graph: Graph::new_undirected(),
            node_attrs: Vec::new(),
            id_to_idx: HashMap::new(),
        }
    }

    /// Convert the graph to GML format string
    pub fn to_gml_string(&self) -> String {
        let mut result = String::new();
        result.push_str("graph [\n");

        // Write nodes
        for node_idx in self.node_indices() {
            if let Some(node_data) = self.get_node_data(node_idx) {
                result.push_str("\tnode [\n");
                result.push_str(&format!("\t\tid {}\n", node_idx.index()));
                result.push_str(&format!(
                    "\t\tlabel \"{}\"\n",
                    String::from_utf8_lossy(&node_data.id)
                ));
                result.push_str(&format!("\t\tsequence \"{}\"\n", node_data.sequence));
                result.push_str(&format!("\t\tstatus \"{}\"\n", node_data.status));
                result.push_str("\t]\n");
            }
        }

        // prevent duplicate edges
        let mut processed_edges = HashSet::new();

        // Write edges
        for edge_ref in self.inner_graph.edge_references() {
            let source = edge_ref.source().index();
            let target = edge_ref.target().index();

            // always store the smaller node index first
            let (source, target) = if source < target {
                (source, target)
            } else {
                (target, source)
            };

            // only write edge if it hasn't been written before
            if processed_edges.insert((source, target)) {
                result.push_str("\tedge [\n");
                result.push_str(&format!("\t\tsource {}\n", source));
                result.push_str(&format!("\t\ttarget {}\n", target));
                result.push_str("\t]\n");
            }
        }

        result.push_str("]\n");
        result
    }

    // get node data by node name
    pub fn get_node_data(&self, idx: NodeIndex) -> Option<&NodeData> {
        self.node_attrs.get(idx.index())
    }

    // get node index by node name
    pub fn get_node_idx(&self, node_id: &[u8]) -> Option<NodeIndex> {
        self.id_to_idx.get(node_id).copied()
    }

    // get node data by node id
    pub fn get_node_data_by_id(&self, node_id: &[u8]) -> Option<&NodeData> {
        self.get_node_idx(node_id)
            .and_then(|idx| self.get_node_data(idx))
    }

    pub fn node_count(&self) -> usize {
        self.inner_graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.inner_graph.edge_count()
    }

    // get neighbors of a node idx
    pub fn neighbors(&self, node_idx: NodeIndex) -> Vec<NodeIndex> {
        self.inner_graph.neighbors(node_idx).collect()
    }

    // get neighbors of a node by node id
    pub fn neighbors_by_id(&self, node_id: &[u8]) -> Vec<NodeIndex> {
        self.get_node_idx(node_id)
            .map(|idx| self.neighbors(idx))
            .unwrap_or_default()
    }

    // get sequence of a node
    pub fn get_sequence(&self, node_idx: NodeIndex) -> Option<&[u8]> {
        self.get_node_data(node_idx)
            .map(|data| data.sequence.as_bytes())
    }

    // get seq length of a node
    pub fn get_seq_len(&self, node_idx: NodeIndex) -> Option<usize> {
        self.get_node_data(node_idx).map(|data| data.sequence.len())
    }

    // get seq length of a node by node id
    pub fn get_seq_len_by_id(&self, node_id: &[u8]) -> Option<usize> {
        self.get_node_idx(node_id)
            .and_then(|idx| self.get_seq_len(idx))
    }

    // get node id
    pub fn get_id(&self, node_idx: NodeIndex) -> Option<&[u8]> {
        self.get_node_data(node_idx).map(|data| data.id.as_slice())
    }

    // get all node indices
    pub fn node_indices(&self) -> impl Iterator<Item = NodeIndex> + '_ {
        self.inner_graph.node_indices()
    }

    // using petgraph's connected_components
    pub fn connected_components(&self) -> usize {
        petgraph::algo::connected_components(&self.inner_graph)
    }

    // get shortest path between two nodes by a* algorithm
    pub fn shortest_path(&self, from_id: &[u8], to_id: &[u8]) -> Option<Vec<NodeIndex>> {
        let start = self.get_node_idx(from_id)?;
        let end = self.get_node_idx(to_id)?;

        petgraph::algo::astar(
            &self.inner_graph,
            start,
            |finish| finish == end,
            |_| 1,
            |_| 0,
        )
        .map(|(_, path)| path)
    }
}

pub fn gfa_to_graph(path: &str) -> Result<GFAGraph> {
    // parse using rs-gfa
    let parser = GFAParser::new();
    parser.ignore_line_type(b'H');
    parser.ignore_line_type(b'P');
    parser.ignore_line_type(b'C');
    let gfa: GFA<Vec<u8>, ()> = parser.parse_file(path)?;

    // build new graph
    let mut gfa_graph = GFAGraph::new();

    // add nodes
    for segment in gfa.segments {
        let node_id = segment.name;
        let node_idx = gfa_graph.inner_graph.add_node(node_id.clone());
        gfa_graph.id_to_idx.insert(node_id.clone(), node_idx);

        // ensure node data vector is long enough
        while gfa_graph.node_attrs.len() <= node_idx.index() {
            gfa_graph.node_attrs.push(NodeData::default());
        }

        // update node data
        gfa_graph.node_attrs[node_idx.index()] = NodeData {
            id: node_id,
            sequence: segment.sequence.display(),
            // unnecessary for whole graph
            status: String::new(),
        };
    }

    // add edges
    for link in gfa.links.iter() {
        let from_id = &link.from_segment;
        let to_id = &link.to_segment;

        if let (Some(from_idx), Some(to_idx)) = (
            gfa_graph.get_node_idx(from_id),
            gfa_graph.get_node_idx(to_id),
        ) {
            gfa_graph.inner_graph.add_edge(from_idx, to_idx, ());
        }
    }

    Ok(gfa_graph)
}
