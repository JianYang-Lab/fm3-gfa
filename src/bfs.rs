use crate::gfa::GFAGraph;
use crate::vcf::BubbleVariant;
use anyhow::Result;
use std::cmp::max;
use std::collections::{HashSet, VecDeque};

// query length of a list of nodes
fn query_at_len(at: &Vec<Vec<u8>>, g: &GFAGraph) -> Result<usize> {
    let mut at_len = 0;
    for node in at {
        // let node_name = &node.to_string();
        let node_len = g.get_seq_len_by_id(node);
        match node_len {
            Some(len) => at_len += len,
            None => return Err(anyhow::anyhow!("Node not found in graph")),
        }
    }
    Ok(at_len)
}

// return max length of all paths and max step of all paths

fn query_dis_step(bubble: &BubbleVariant, g: &GFAGraph) -> Result<(usize, usize)> {
    let ref_nodes = bubble.get_ref_nodes(false);
    let alt_nodes = bubble.get_alt_nodes(false);
    let ref_len = query_at_len(&ref_nodes, g)?;
    let alt_len = query_at_len(&alt_nodes, g)?;
    Ok((max(ref_len, alt_len), max(ref_nodes.len(), alt_nodes.len())))
}

pub fn extract_subgraph_by_bfs(bubble: &BubbleVariant, g: &GFAGraph) -> Result<GFAGraph> {
    // get all start nodes, max distance and max step

    let ref_nodes = bubble.get_ref_nodes(true);
    let alt_nodes = bubble.get_alt_nodes(true);

    let (max_distance, max_step) = query_dis_step(bubble, g)?;

    // init a subgraph
    let mut subgraph = GFAGraph::new();

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    // add start nodes to queue
    for node in ref_nodes {
        if let Some(node_idx) = g.get_node_idx(&node) {
            if !visited.contains(&node_idx) {
                queue.push_back((node_idx, 0, 0));
                visited.insert(node_idx);

                // add node to subgraph
                if let Some(node_data) = g.get_node_data(node_idx) {
                    subgraph.add_node(
                        node_data.id.clone(),
                        node_data.sequence.clone(),
                        "REF".to_string(),
                    )?;
                }
            }
        }
    }

    // add start nodes to queue
    for node in alt_nodes {
        if let Some(node_idx) = g.get_node_idx(&node) {
            if !visited.contains(&node_idx) {
                queue.push_back((node_idx, 0, 0));
                visited.insert(node_idx);

                // add node to subgraph
                if let Some(node_data) = g.get_node_data(node_idx) {
                    subgraph.add_node(
                        node_data.id.clone(),
                        node_data.sequence.clone(),
                        "ALT".to_string(),
                    )?;
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
                        subgraph.add_node(
                            neighbor_data.id.clone(),
                            neighbor_data.sequence.clone(),
                            "REF".to_string(),
                        )?;
                    }

                    // add edge to subgraph
                    subgraph.add_edge(&current_id, &neighbor_data.id)?;
                }
            }
        }
    }

    Ok(subgraph)
}
