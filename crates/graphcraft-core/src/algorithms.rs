use crate::error::GraphError;
use crate::model::GraphDocument;
use petgraph::algo::{connected_components, dijkstra, is_cyclic_directed};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Computed network centrality and structural metrics for the current graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub connected_components: usize,
    pub has_cycles: bool,
    pub density: f64,
    pub top_pagerank: Vec<NodeScore>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeScore {
    pub id: String,
    pub label: String,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShortestPathResult {
    pub found: bool,
    pub path: Vec<String>,
    pub total_cost: f64,
}

fn build_petgraph(doc: &GraphDocument) -> (DiGraph<String, f64>, HashMap<String, NodeIndex>) {
    let mut g = DiGraph::<String, f64>::new();
    let mut index_map = HashMap::with_capacity(doc.nodes.len());

    for n in &doc.nodes {
        let idx = g.add_node(n.id.clone());
        index_map.insert(n.id.clone(), idx);
    }

    for e in &doc.edges {
        if let (Some(&src), Some(&dst)) = (index_map.get(&e.source), index_map.get(&e.target)) {
            let cost = if e.weight > 0.0 {
                1.0 / f64::from(e.weight)
            } else {
                1.0
            };
            g.add_edge(src, dst, cost);
        }
    }

    (g, index_map)
}

/// Computes iterative PageRank centrality scores for all nodes in the graph.
pub fn compute_pagerank(doc: &GraphDocument, damping: f64, iterations: usize) -> Vec<NodeScore> {
    let n = doc.nodes.len();
    if n == 0 {
        return Vec::new();
    }

    let mut id_to_idx = HashMap::with_capacity(n);
    for (i, node) in doc.nodes.iter().enumerate() {
        id_to_idx.insert(node.id.as_str(), i);
    }

    let mut out_degree = vec![0usize; n];
    let mut in_neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];

    for edge in &doc.edges {
        if let (Some(&src), Some(&dst)) = (
            id_to_idx.get(edge.source.as_str()),
            id_to_idx.get(edge.target.as_str()),
        ) {
            out_degree[src] += 1;
            in_neighbors[dst].push(src);
        }
    }

    let mut ranks = vec![1.0 / (n as f64); n];
    let base = (1.0 - damping) / (n as f64);

    for _ in 0..iterations {
        let mut next = vec![base; n];
        let mut dangling_sum = 0.0;
        for i in 0..n {
            if out_degree[i] == 0 {
                dangling_sum += ranks[i];
            }
        }
        let dangling_share = damping * dangling_sum / (n as f64);

        for i in 0..n {
            let mut sum = 0.0;
            for &src in &in_neighbors[i] {
                if out_degree[src] > 0 {
                    sum += ranks[src] / (out_degree[src] as f64);
                }
            }
            next[i] += damping * sum + dangling_share;
        }
        ranks = next;
    }

    let mut scores: Vec<NodeScore> = doc
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| NodeScore {
            id: node.id.clone(),
            label: node.label.clone(),
            score: (ranks[i] * 1000.0).round() / 1000.0,
        })
        .collect();

    scores.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scores
}

/// Computes aggregate topological statistics and top PageRank nodes.
pub fn compute_stats(doc: &GraphDocument) -> GraphStats {
    let (g, _) = build_petgraph(doc);
    let n = doc.nodes.len();
    let m = doc.edges.len();
    let max_edges = if n > 1 { n * (n - 1) } else { 1 };
    let density = if n > 1 {
        ((m as f64) / (max_edges as f64) * 1000.0).round() / 1000.0
    } else {
        0.0
    };

    let pr = compute_pagerank(doc, 0.85, 25);
    let top_pagerank = pr.into_iter().take(5).collect();

    GraphStats {
        node_count: n,
        edge_count: m,
        connected_components: connected_components(&g),
        has_cycles: is_cyclic_directed(&g),
        density,
        top_pagerank,
    }
}

/// Finds the shortest weighted path between `source_id` and `target_id` (treating edges as undirected
/// or directed based on `directed` flag).
pub fn find_shortest_path(
    doc: &GraphDocument,
    source_id: &str,
    target_id: &str,
    directed: bool,
) -> Result<ShortestPathResult, GraphError> {
    let (mut g, index_map) = build_petgraph(doc);

    let &src_idx = index_map
        .get(source_id)
        .ok_or_else(|| GraphError::NodeNotFound(source_id.to_string()))?;
    let &dst_idx = index_map
        .get(target_id)
        .ok_or_else(|| GraphError::NodeNotFound(target_id.to_string()))?;

    if !directed {
        for e in &doc.edges {
            if let (Some(&s), Some(&t)) = (index_map.get(&e.source), index_map.get(&e.target)) {
                let cost = if e.weight > 0.0 {
                    1.0 / f64::from(e.weight)
                } else {
                    1.0
                };
                g.add_edge(t, s, cost);
            }
        }
    }

    if src_idx == dst_idx {
        return Ok(ShortestPathResult {
            found: true,
            path: vec![source_id.to_string()],
            total_cost: 0.0,
        });
    }

    let costs = dijkstra(&g, src_idx, Some(dst_idx), |e| *e.weight());
    let Some(&total_cost) = costs.get(&dst_idx) else {
        return Ok(ShortestPathResult {
            found: false,
            path: Vec::new(),
            total_cost: 0.0,
        });
    };

    // Reconstruct path via BFS/predecessor walk over valid shortest edges
    let mut path_indices = vec![dst_idx];
    let mut curr = dst_idx;
    let mut guard = 0usize;
    while curr != src_idx && guard <= doc.nodes.len() {
        guard += 1;
        let curr_cost = costs.get(&curr).copied().unwrap_or(f64::INFINITY);
        let mut found_prev = None;
        for edge in g.edges_directed(curr, petgraph::Direction::Incoming) {
            let prev = edge.source();
            if let Some(&prev_cost) = costs.get(&prev) {
                if (prev_cost + *edge.weight() - curr_cost).abs() < 1e-6 {
                    found_prev = Some(prev);
                    break;
                }
            }
        }
        match found_prev {
            Some(p) => {
                path_indices.push(p);
                curr = p;
            }
            None => break,
        }
    }

    path_indices.reverse();
    let path: Vec<String> = path_indices
        .into_iter()
        .filter_map(|idx| g.node_weight(idx).cloned())
        .collect();

    Ok(ShortestPathResult {
        found: path.first().map(String::as_str) == Some(source_id),
        path,
        total_cost: (total_cost * 1000.0).round() / 1000.0,
    })
}
