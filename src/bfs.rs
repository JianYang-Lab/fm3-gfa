use crate::gfa::GFAGraph;
use crate::vcf::BubbleVariant;
use anyhow::Result;
use std::cmp::max;
use std::collections::{HashSet, VecDeque};

// query length of a list of nodes
fn query_at_len(at: Vec<Vec<u8>>, g: &GFAGraph) -> Result<usize> {
    let mut at_len = 0;
    for node in at {
        // let node_name = &node.to_string();
        let node_len = g.get_seq_len_by_id(&node);
        match node_len {
            Some(len) => at_len += len,
            None => return Err(anyhow::anyhow!("Node not found in graph")),
        }
    }
    Ok(at_len)
}

// return max length of all paths
fn query_distance(bubble: &BubbleVariant, g: &GFAGraph) -> Result<usize> {
    let ref_nodes = bubble.get_ref_nodes();
    let alt_nodes = bubble.get_alt_nodes();
    let ref_len = query_at_len(ref_nodes, g)?;
    let alt_len = query_at_len(alt_nodes, g)?;
    Ok(max(ref_len, alt_len))
}

fn query_step(bubble: BubbleVariant) -> usize {
    let ref_nodes = bubble.get_ref_nodes();
    let alt_nodes = bubble.get_alt_nodes();
    let ref_len = ref_nodes.len();
    let alt_len = alt_nodes.len();
    max(ref_len, alt_len)
}

pub fn extract_subgraph_by_bfs(bubble: &BubbleVariant, g: &GFAGraph) -> Result<GFAGraph> {
    // get all start nodes, max distance and max step
    let start_nodes = bubble.get_all_nodes();
    let max_distance = query_distance(bubble, g)?;
    let max_step = query_step(bubble.clone());

    // init a subgraph
    let mut subgraph = GFAGraph::new();

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    // init a queue with all start nodes
    for node_id in &start_nodes {
        if let Some(node_idx) = g.get_node_idx(node_id) {
            if !visited.contains(&node_idx) {
                queue.push_back((node_idx, 0, 0));
                visited.insert(node_idx);

                // add node to subgraph
                if let Some(node_data) = g.get_node_data(node_idx) {
                    subgraph.add_node(node_data.id.clone(), node_data.sequence.clone())?;
                }
            }
        }
    }

    // BFS
    while let Some((current_idx, dist_so_far, step_so_far)) = queue.pop_front() {
        let current_cost = g
            .get_node_data(current_idx)
            .map(|data| data.sequence.len())
            .unwrap_or(0);

        let current_id = g
            .get_node_data(current_idx)
            .map(|data| data.id.clone())
            .ok_or_else(|| anyhow::anyhow!("Node data not found"))?;

        // check neighbors to adding nodes and edges
        for neighbor_idx in g.neighbors(current_idx) {
            if let Some(neighbor_data) = g.get_node_data(neighbor_idx) {
                let new_distance = dist_so_far + current_cost;
                let new_step = step_so_far + 1;

                if new_distance <= max_distance || new_step <= max_step {
                    // if neighbor not visited, add it to queue
                    if !visited.contains(&neighbor_idx) {
                        visited.insert(neighbor_idx);
                        queue.push_back((neighbor_idx, new_distance, new_step));

                        // add node to subgraph
                        subgraph
                            .add_node(neighbor_data.id.clone(), neighbor_data.sequence.clone())?;
                    }

                    // add edge to subgraph
                    subgraph.add_edge(&current_id, &neighbor_data.id)?;
                }
            }
        }
    }

    Ok(subgraph)
}
