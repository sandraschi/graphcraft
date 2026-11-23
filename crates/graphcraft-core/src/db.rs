use crate::error::GraphError;
use crate::model::{GraphDocument, GraphEdge, GraphNode, NodeKind, Vec2Pos};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Returns the default per-user SQLite database path for GraphCraft.
pub fn default_db_path() -> PathBuf {
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(local_app_data)
            .join("graphcraft")
            .join("graphcraft.db");
    }
    if let Some(xdg_data) = std::env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(xdg_data)
            .join("graphcraft")
            .join("graphcraft.db");
    }
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        return PathBuf::from(home)
            .join(".graphcraft")
            .join("graphcraft.db");
    }
    PathBuf::from("graphcraft.db")
}

fn ensure_schema(conn: &Connection) -> Result<(), GraphError> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS workspace_meta (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            title TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS nodes (
            id TEXT PRIMARY KEY,
            label TEXT NOT NULL,
            kind TEXT NOT NULL,
            notes TEXT NOT NULL,
            tags_json TEXT NOT NULL,
            x REAL NOT NULL,
            y REAL NOT NULL,
            weight REAL NOT NULL DEFAULT 1.0,
            metadata_json TEXT NOT NULL DEFAULT '{}'
        );
        CREATE TABLE IF NOT EXISTS edges (
            id TEXT PRIMARY KEY,
            source TEXT NOT NULL,
            target TEXT NOT NULL,
            relation TEXT NOT NULL,
            weight REAL NOT NULL
        );
        ",
    )?;
    Ok(())
}

fn kind_to_str(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Concept => "concept",
        NodeKind::Project => "project",
        NodeKind::Person => "person",
        NodeKind::Hardware => "hardware",
        NodeKind::Document => "document",
        NodeKind::Cluster => "cluster",
        NodeKind::Drawer => "drawer",
    }
}

fn str_to_kind(raw: &str) -> NodeKind {
    match raw.trim().to_ascii_lowercase().as_str() {
        "project" => NodeKind::Project,
        "person" => NodeKind::Person,
        "hardware" => NodeKind::Hardware,
        "document" => NodeKind::Document,
        "cluster" => NodeKind::Cluster,
        "drawer" => NodeKind::Drawer,
        _ => NodeKind::Concept,
    }
}

/// Persists the entire `GraphDocument` atomically into a SQLite database file.
pub fn save_to_db(doc: &GraphDocument, path: &Path) -> Result<(), GraphError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| GraphError::Io(e.to_string()))?;
        }
    }
    let mut conn = Connection::open(path)?;
    ensure_schema(&conn)?;

    let tx = conn.transaction()?;
    tx.execute("DELETE FROM workspace_meta", [])?;
    tx.execute("DELETE FROM nodes", [])?;
    tx.execute("DELETE FROM edges", [])?;

    tx.execute(
        "INSERT INTO workspace_meta (id, title) VALUES (1, ?1)",
        params![doc.title],
    )?;

    {
        let mut stmt = tx.prepare(
            "INSERT INTO nodes (id, label, kind, notes, tags_json, x, y, weight, metadata_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )?;
        for n in &doc.nodes {
            let tags_json = serde_json::to_string(&n.tags)
                .map_err(|e| GraphError::Serialization(e.to_string()))?;
            let metadata_json = serde_json::to_string(&n.metadata)
                .map_err(|e| GraphError::Serialization(e.to_string()))?;
            stmt.execute(params![
                n.id,
                n.label,
                kind_to_str(n.kind),
                n.notes,
                tags_json,
                f64::from(n.pos.x),
                f64::from(n.pos.y),
                f64::from(n.weight),
                metadata_json,
            ])?;
        }
    }

    {
        let mut stmt = tx.prepare(
            "INSERT INTO edges (id, source, target, relation, weight) VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        for e in &doc.edges {
            stmt.execute(params![
                e.id,
                e.source,
                e.target,
                e.relation,
                f64::from(e.weight),
            ])?;
        }
    }

    tx.commit()?;
    Ok(())
}

/// Loads a `GraphDocument` from a SQLite database file, returning `None` if no workspace exists yet.
pub fn load_from_db(path: &Path) -> Result<Option<GraphDocument>, GraphError> {
    if !path.exists() {
        return Ok(None);
    }
    let conn = Connection::open(path)?;
    ensure_schema(&conn)?;

    let title: Option<String> = conn
        .query_row("SELECT title FROM workspace_meta WHERE id = 1", [], |row| {
            row.get(0)
        })
        .optional()?;

    let Some(title) = title else {
        return Ok(None);
    };

    let mut nodes = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, label, kind, notes, tags_json, x, y, weight, metadata_json FROM nodes ORDER BY rowid",
        )?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let label: String = row.get(1)?;
            let kind_str: String = row.get(2)?;
            let notes: String = row.get(3)?;
            let tags_json: String = row.get(4)?;
            let x: f64 = row.get(5)?;
            let y: f64 = row.get(6)?;
            let weight: f64 = row.get(7)?;
            let metadata_json: String = row.get(8)?;
            Ok((
                id,
                label,
                kind_str,
                notes,
                tags_json,
                x,
                y,
                weight,
                metadata_json,
            ))
        })?;

        for row in rows {
            let (id, label, kind_str, notes, tags_json, x, y, weight, metadata_json) = row?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            let metadata: BTreeMap<String, String> =
                serde_json::from_str(&metadata_json).unwrap_or_default();
            nodes.push(GraphNode {
                id,
                label,
                kind: str_to_kind(&kind_str),
                notes,
                tags,
                pos: Vec2Pos::new(x as f32, y as f32),
                weight: weight as f32,
                metadata,
            });
        }
    }

    let mut edges = Vec::new();
    {
        let mut stmt =
            conn.prepare("SELECT id, source, target, relation, weight FROM edges ORDER BY rowid")?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let source: String = row.get(1)?;
            let target: String = row.get(2)?;
            let relation: String = row.get(3)?;
            let weight: f64 = row.get(4)?;
            Ok((id, source, target, relation, weight))
        })?;

        for row in rows {
            let (id, source, target, relation, weight) = row?;
            edges.push(GraphEdge {
                id,
                source,
                target,
                relation,
                weight: weight as f32,
            });
        }
    }

    Ok(Some(GraphDocument {
        title,
        nodes,
        edges,
    }))
}
