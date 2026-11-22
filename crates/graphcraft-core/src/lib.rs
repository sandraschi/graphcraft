#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]

pub mod algorithms;
pub mod error;
pub mod io;
pub mod layout;
pub mod model;

use crate::algorithms::{compute_pagerank, compute_stats, find_shortest_path};
use crate::error::GraphError;
use crate::io::{export_graph, ExportFormat};
use crate::layout::{apply_circular_layout, apply_grid_layout, step_force_directed};
use crate::model::{demo_seed_document, GraphDocument, GraphEdge, GraphNode, NodeKind, Vec2Pos};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;
use uuid::Uuid;

/// Universal Command Bus shared across L1 core, L2 CLI/MCP, and L3 egui desktop app.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum Command {
    Status,
    GetDocument,
    ListNodes {
        query: Option<String>,
        kind: Option<NodeKind>,
    },
    AddNode {
        id: Option<String>,
        label: String,
        kind: Option<NodeKind>,
        notes: Option<String>,
        tags: Option<Vec<String>>,
        x: Option<f32>,
        y: Option<f32>,
    },
    UpdateNode {
        id: String,
        label: Option<String>,
        kind: Option<NodeKind>,
        notes: Option<String>,
        tags: Option<Vec<String>>,
        x: Option<f32>,
        y: Option<f32>,
    },
    DeleteNode {
        id: String,
    },
    AddEdge {
        id: Option<String>,
        source: String,
        target: String,
        relation: String,
        weight: Option<f32>,
    },
    DeleteEdge {
        id: String,
    },
    ComputeStats,
    ComputePagerank {
        damping: Option<f64>,
        iterations: Option<usize>,
    },
    FindShortestPath {
        source: String,
        target: String,
        directed: Option<bool>,
    },
    RunLayout {
        mode: String,
        steps: Option<usize>,
    },
    Export {
        format: String,
    },
    LoadDemoSeed,
    ClearGraph,
    Undo,
    Redo,
}

/// Stateful in-memory graph engine with undo/redo snapshot history.
#[derive(Debug, Clone)]
pub struct GraphEngine {
    pub doc: GraphDocument,
    undo_stack: Vec<GraphDocument>,
    redo_stack: Vec<GraphDocument>,
}

impl Default for GraphEngine {
    fn default() -> Self {
        Self::new_with_demo()
    }
}

impl GraphEngine {
    /// Creates an engine pre-loaded with the Vienna Open Robotics & Craft demo graph.
    pub fn new_with_demo() -> Self {
        Self {
            doc: demo_seed_document(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Creates an empty workspace.
    pub fn new_empty(title: impl Into<String>) -> Self {
        Self {
            doc: GraphDocument {
                title: title.into(),
                nodes: Vec::new(),
                edges: Vec::new(),
            },
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    fn snapshot_for_undo(&mut self) {
        self.undo_stack.push(self.doc.clone());
        if self.undo_stack.len() > 64 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.doc.clone());
            self.doc = prev;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.doc.clone());
            self.doc = next;
            true
        } else {
            false
        }
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), GraphError> {
        let content = serde_json::to_string_pretty(&self.doc)
            .map_err(|e| GraphError::Serialization(e.to_string()))?;
        std::fs::write(path, content).map_err(|e| GraphError::Io(e.to_string()))
    }

    pub fn load_from_file(&mut self, path: &Path) -> Result<(), GraphError> {
        let content = std::fs::read_to_string(path).map_err(|e| GraphError::Io(e.to_string()))?;
        let parsed: GraphDocument =
            serde_json::from_str(&content).map_err(|e| GraphError::Serialization(e.to_string()))?;
        self.snapshot_for_undo();
        self.doc = parsed;
        Ok(())
    }

    pub fn execute(&mut self, cmd: Command) -> Result<Value, GraphError> {
        match cmd {
            Command::Status => Ok(json!({
                "app": "graphcraft",
                "version": env!("CARGO_PKG_VERSION"),
                "title": self.doc.title,
                "nodes": self.doc.nodes.len(),
                "edges": self.doc.edges.len(),
                "can_undo": self.can_undo(),
                "can_redo": self.can_redo(),
            })),
            Command::GetDocument => serde_json::to_value(&self.doc)
                .map_err(|e| GraphError::Serialization(e.to_string())),
            Command::ListNodes { query, kind } => {
                let q = query.unwrap_or_default().trim().to_ascii_lowercase();
                let filtered: Vec<&GraphNode> = self
                    .doc
                    .nodes
                    .iter()
                    .filter(|n| {
                        let kind_ok = kind.is_none_or(|k| n.kind == k);
                        let query_ok = q.is_empty()
                            || n.id.to_ascii_lowercase().contains(&q)
                            || n.label.to_ascii_lowercase().contains(&q)
                            || n.notes.to_ascii_lowercase().contains(&q)
                            || n.tags.iter().any(|t| t.to_ascii_lowercase().contains(&q));
                        kind_ok && query_ok
                    })
                    .collect();
                Ok(json!({
                    "count": filtered.len(),
                    "nodes": filtered,
                }))
            }
            Command::AddNode {
                id,
                label,
                kind,
                notes,
                tags,
                x,
                y,
            } => {
                let clean_label = label.trim().to_string();
                if clean_label.is_empty() {
                    return Err(GraphError::Validation("Node label cannot be empty".into()));
                }
                let node_id = match id {
                    Some(raw) if !raw.trim().is_empty() => raw.trim().to_string(),
                    _ => format!(
                        "n_{}",
                        Uuid::new_v4()
                            .to_string()
                            .split('-')
                            .next()
                            .unwrap_or("0000")
                    ),
                };
                if self.doc.nodes.iter().any(|n| n.id == node_id) {
                    return Err(GraphError::DuplicateNode(node_id));
                }
                self.snapshot_for_undo();
                let node = GraphNode {
                    id: node_id.clone(),
                    label: clean_label,
                    kind: kind.unwrap_or_default(),
                    notes: notes.unwrap_or_default(),
                    tags: tags.unwrap_or_default(),
                    pos: Vec2Pos::new(x.unwrap_or(0.0), y.unwrap_or(0.0)),
                    weight: 1.0,
                    metadata: BTreeMap::new(),
                };
                self.doc.nodes.push(node.clone());
                Ok(json!({ "added": node }))
            }
            Command::UpdateNode {
                id,
                label,
                kind,
                notes,
                tags,
                x,
                y,
            } => {
                if !self.doc.nodes.iter().any(|n| n.id == id) {
                    return Err(GraphError::NodeNotFound(id));
                }
                self.snapshot_for_undo();
                let node = self
                    .doc
                    .nodes
                    .iter_mut()
                    .find(|n| n.id == id)
                    .ok_or_else(|| GraphError::NodeNotFound(id.clone()))?;
                if let Some(lbl) = label {
                    let trimmed = lbl.trim();
                    if trimmed.is_empty() {
                        return Err(GraphError::Validation("Node label cannot be empty".into()));
                    }
                    node.label = trimmed.to_string();
                }
                if let Some(k) = kind {
                    node.kind = k;
                }
                if let Some(nt) = notes {
                    node.notes = nt;
                }
                if let Some(tg) = tags {
                    node.tags = tg;
                }
                if let Some(nx) = x {
                    node.pos.x = nx;
                }
                if let Some(ny) = y {
                    node.pos.y = ny;
                }
                Ok(json!({ "updated": node }))
            }
            Command::DeleteNode { id } => {
                let idx = self
                    .doc
                    .nodes
                    .iter()
                    .position(|n| n.id == id)
                    .ok_or_else(|| GraphError::NodeNotFound(id.clone()))?;
                self.snapshot_for_undo();
                let removed = self.doc.nodes.remove(idx);
                let before_edges = self.doc.edges.len();
                self.doc.edges.retain(|e| e.source != id && e.target != id);
                let removed_edges = before_edges - self.doc.edges.len();
                Ok(json!({
                    "deleted_node": removed.id,
                    "removed_incident_edges": removed_edges,
                }))
            }
            Command::AddEdge {
                id,
                source,
                target,
                relation,
                weight,
            } => {
                if !self.doc.nodes.iter().any(|n| n.id == source) {
                    return Err(GraphError::NodeNotFound(source));
                }
                if !self.doc.nodes.iter().any(|n| n.id == target) {
                    return Err(GraphError::NodeNotFound(target));
                }
                let clean_rel = relation.trim().to_string();
                if clean_rel.is_empty() {
                    return Err(GraphError::Validation(
                        "Edge relation cannot be empty".into(),
                    ));
                }
                let edge_id = match id {
                    Some(raw) if !raw.trim().is_empty() => raw.trim().to_string(),
                    _ => format!(
                        "e_{}",
                        Uuid::new_v4()
                            .to_string()
                            .split('-')
                            .next()
                            .unwrap_or("0000")
                    ),
                };
                self.snapshot_for_undo();
                let edge = GraphEdge {
                    id: edge_id,
                    source,
                    target,
                    relation: clean_rel,
                    weight: weight.unwrap_or(1.0).max(0.1),
                };
                self.doc.edges.push(edge.clone());
                Ok(json!({ "added": edge }))
            }
            Command::DeleteEdge { id } => {
                let idx = self
                    .doc
                    .edges
                    .iter()
                    .position(|e| e.id == id)
                    .ok_or_else(|| GraphError::EdgeNotFound(id.clone()))?;
                self.snapshot_for_undo();
                let removed = self.doc.edges.remove(idx);
                Ok(json!({ "deleted_edge": removed.id }))
            }
            Command::ComputeStats => {
                let stats = compute_stats(&self.doc);
                serde_json::to_value(stats).map_err(|e| GraphError::Serialization(e.to_string()))
            }
            Command::ComputePagerank {
                damping,
                iterations,
            } => {
                let d = damping.unwrap_or(0.85).clamp(0.1, 0.99);
                let iters = iterations.unwrap_or(25).clamp(1, 200);
                let scores = compute_pagerank(&self.doc, d, iters);
                Ok(json!({ "scores": scores }))
            }
            Command::FindShortestPath {
                source,
                target,
                directed,
            } => {
                let res =
                    find_shortest_path(&self.doc, &source, &target, directed.unwrap_or(false))?;
                serde_json::to_value(res).map_err(|e| GraphError::Serialization(e.to_string()))
            }
            Command::RunLayout { mode, steps } => {
                self.snapshot_for_undo();
                match mode.trim().to_ascii_lowercase().as_str() {
                    "force" | "force_directed" => {
                        let count = steps.unwrap_or(30).clamp(1, 500);
                        for _ in 0..count {
                            step_force_directed(&mut self.doc, 0.8);
                        }
                        Ok(json!({ "layout": "force_directed", "steps": count }))
                    }
                    "circle" | "circular" => {
                        apply_circular_layout(&mut self.doc);
                        Ok(json!({ "layout": "circular" }))
                    }
                    "grid" => {
                        apply_grid_layout(&mut self.doc);
                        Ok(json!({ "layout": "grid" }))
                    }
                    other => Err(GraphError::Validation(format!(
                        "Unknown layout mode '{other}'. Expected force, circular, or grid."
                    ))),
                }
            }
            Command::Export { format } => {
                let fmt = ExportFormat::parse(&format)?;
                let content = export_graph(&self.doc, fmt)?;
                Ok(json!({
                    "format": format,
                    "content": content,
                }))
            }
            Command::LoadDemoSeed => {
                self.snapshot_for_undo();
                self.doc = demo_seed_document();
                Ok(json!({
                    "loaded": self.doc.title,
                    "nodes": self.doc.nodes.len(),
                    "edges": self.doc.edges.len(),
                }))
            }
            Command::ClearGraph => {
                self.snapshot_for_undo();
                self.doc = GraphDocument::default();
                Ok(json!({ "cleared": true }))
            }
            Command::Undo => Ok(json!({ "undone": self.undo() })),
            Command::Redo => Ok(json!({ "redone": self.redo() })),
        }
    }
}

/// Never-panicking universal command dispatch boundary for L2 and L3.
pub fn dispatch(engine: &mut GraphEngine, cmd: Command) -> Value {
    match engine.execute(cmd) {
        Ok(val) => json!({ "ok": true, "data": val }),
        Err(err) => json!({ "ok": false, "error": err.to_string() }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_seed_loads_valid_graph() {
        let mut engine = GraphEngine::new_with_demo();
        let res = dispatch(&mut engine, Command::Status);
        assert_eq!(res["ok"], true);
        assert_eq!(res["data"]["nodes"], 10);
        assert_eq!(res["data"]["edges"], 12);
    }

    #[test]
    fn add_update_delete_node_roundtrip() {
        let mut engine = GraphEngine::new_empty("Test");
        let add = dispatch(
            &mut engine,
            Command::AddNode {
                id: Some("n1".into()),
                label: "Alpha".into(),
                kind: Some(NodeKind::Concept),
                notes: Some("First concept".into()),
                tags: Some(vec!["tag1".into()]),
                x: Some(10.0),
                y: Some(20.0),
            },
        );
        assert_eq!(add["ok"], true);
        assert_eq!(engine.doc.nodes.len(), 1);

        let upd = dispatch(
            &mut engine,
            Command::UpdateNode {
                id: "n1".into(),
                label: Some("Alpha Prime".into()),
                kind: None,
                notes: None,
                tags: None,
                x: None,
                y: None,
            },
        );
        assert_eq!(upd["ok"], true);
        assert_eq!(engine.doc.nodes[0].label, "Alpha Prime");

        let del = dispatch(&mut engine, Command::DeleteNode { id: "n1".into() });
        assert_eq!(del["ok"], true);
        assert!(engine.doc.nodes.is_empty());
    }

    #[test]
    fn hostile_empty_label_rejected() {
        let mut engine = GraphEngine::new_empty("Test");
        let res = dispatch(
            &mut engine,
            Command::AddNode {
                id: None,
                label: "   ".into(),
                kind: None,
                notes: None,
                tags: None,
                x: None,
                y: None,
            },
        );
        assert_eq!(res["ok"], false);
        assert!(res["error"].as_str().unwrap_or("").contains("empty"));
    }

    #[test]
    fn duplicate_node_id_rejected() {
        let mut engine = GraphEngine::new_empty("Test");
        let _ = dispatch(
            &mut engine,
            Command::AddNode {
                id: Some("dup".into()),
                label: "One".into(),
                kind: None,
                notes: None,
                tags: None,
                x: None,
                y: None,
            },
        );
        let second = dispatch(
            &mut engine,
            Command::AddNode {
                id: Some("dup".into()),
                label: "Two".into(),
                kind: None,
                notes: None,
                tags: None,
                x: None,
                y: None,
            },
        );
        assert_eq!(second["ok"], false);
    }

    #[test]
    fn deleting_node_cascades_incident_edges() {
        let mut engine = GraphEngine::new_with_demo();
        let res = dispatch(
            &mut engine,
            Command::DeleteNode {
                id: "sandra".into(),
            },
        );
        assert_eq!(res["ok"], true);
        assert_eq!(res["data"]["removed_incident_edges"], 3);
        assert!(!engine
            .doc
            .edges
            .iter()
            .any(|e| e.source == "sandra" || e.target == "sandra"));
    }

    #[test]
    fn undo_redo_restores_exact_state() {
        let mut engine = GraphEngine::new_with_demo();
        let initial_nodes = engine.doc.nodes.len();
        let _ = dispatch(&mut engine, Command::DeleteNode { id: "boomy".into() });
        assert_eq!(engine.doc.nodes.len(), initial_nodes - 1);

        let undo = dispatch(&mut engine, Command::Undo);
        assert_eq!(undo["data"]["undone"], true);
        assert_eq!(engine.doc.nodes.len(), initial_nodes);

        let redo = dispatch(&mut engine, Command::Redo);
        assert_eq!(redo["data"]["redone"], true);
        assert_eq!(engine.doc.nodes.len(), initial_nodes - 1);
    }

    #[test]
    fn pagerank_scores_sum_near_one() {
        let engine = GraphEngine::new_with_demo();
        let scores = compute_pagerank(&engine.doc, 0.85, 30);
        assert_eq!(scores.len(), 10);
        let total: f64 = scores.iter().map(|s| s.score).sum();
        assert!((total - 1.0).abs() < 0.05);
    }

    #[test]
    fn shortest_path_finds_route_between_boomy_and_storytold() {
        let engine = GraphEngine::new_with_demo();
        let res = find_shortest_path(&engine.doc, "boomy", "storytold", false);
        assert!(res.is_ok());
        if let Ok(path_res) = res {
            assert!(path_res.found);
            assert!(path_res.path.len() >= 2);
            assert_eq!(path_res.path.first().map(String::as_str), Some("boomy"));
            assert_eq!(path_res.path.last().map(String::as_str), Some("storytold"));
        }
    }

    #[test]
    fn shortest_path_missing_node_errors() {
        let engine = GraphEngine::new_with_demo();
        let res = find_shortest_path(&engine.doc, "boomy", "nonexistent", false);
        assert!(res.is_err());
    }

    #[test]
    fn force_directed_and_circular_layouts_produce_finite_coords() {
        let mut engine = GraphEngine::new_with_demo();
        let f_res = dispatch(
            &mut engine,
            Command::RunLayout {
                mode: "force".into(),
                steps: Some(25),
            },
        );
        assert_eq!(f_res["ok"], true);
        for n in &engine.doc.nodes {
            assert!(n.pos.x.is_finite() && n.pos.y.is_finite());
        }

        let c_res = dispatch(
            &mut engine,
            Command::RunLayout {
                mode: "circular".into(),
                steps: None,
            },
        );
        assert_eq!(c_res["ok"], true);
        for n in &engine.doc.nodes {
            assert!(n.pos.x.is_finite() && n.pos.y.is_finite());
        }
    }

    #[test]
    fn export_all_four_formats_succeeds() {
        let mut engine = GraphEngine::new_with_demo();
        for fmt in ["json", "dot", "mermaid", "gexf"] {
            let res = dispatch(&mut engine, Command::Export { format: fmt.into() });
            assert_eq!(res["ok"], true);
            let content = res["data"]["content"].as_str().unwrap_or("");
            assert!(!content.is_empty());
        }
    }

    #[test]
    fn list_nodes_filters_by_query_and_kind() {
        let mut engine = GraphEngine::new_with_demo();
        let res = dispatch(
            &mut engine,
            Command::ListNodes {
                query: Some("boomy".into()),
                kind: Some(NodeKind::Hardware),
            },
        );
        assert_eq!(res["ok"], true);
        assert_eq!(res["data"]["count"], 1);
    }

    #[test]
    fn file_save_and_load_roundtrip() {
        let engine = GraphEngine::new_with_demo();
        let dir = std::env::temp_dir();
        let path = dir.join(format!("graphcraft_test_{}.json", Uuid::new_v4()));
        assert!(engine.save_to_file(&path).is_ok());

        let mut loaded = GraphEngine::new_empty("Empty");
        assert!(loaded.load_from_file(&path).is_ok());
        assert_eq!(loaded.doc, engine.doc);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn edge_missing_endpoint_rejected() {
        let mut engine = GraphEngine::new_with_demo();
        let res = dispatch(
            &mut engine,
            Command::AddEdge {
                id: None,
                source: "boomy".into(),
                target: "ghost_node".into(),
                relation: "links".into(),
                weight: Some(1.0),
            },
        );
        assert_eq!(res["ok"], false);
    }

    #[test]
    fn clear_graph_and_reload_demo_seed() {
        let mut engine = GraphEngine::new_with_demo();
        let clr = dispatch(&mut engine, Command::ClearGraph);
        assert_eq!(clr["ok"], true);
        assert!(engine.doc.nodes.is_empty());

        let reload = dispatch(&mut engine, Command::LoadDemoSeed);
        assert_eq!(reload["ok"], true);
        assert_eq!(engine.doc.nodes.len(), 10);
    }

    #[test]
    fn grid_layout_places_distinct_coordinates() {
        let mut engine = GraphEngine::new_with_demo();
        let res = dispatch(
            &mut engine,
            Command::RunLayout {
                mode: "grid".into(),
                steps: None,
            },
        );
        assert_eq!(res["ok"], true);
        let first = engine.doc.nodes[0].pos;
        let second = engine.doc.nodes[1].pos;
        assert!((first.x - second.x).abs() > 1.0 || (first.y - second.y).abs() > 1.0);
    }
}
