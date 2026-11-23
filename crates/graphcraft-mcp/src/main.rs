#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]

use clap::{Parser, Subcommand};
use graphcraft_core::model::NodeKind;
use graphcraft_core::{dispatch, Command, GraphEngine};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "graphcraft-mcp",
    version,
    about = "GraphCraft CLI & MCP Stdio Server"
)]
struct Cli {
    /// Optional path to a GraphCraft JSON workspace file
    #[arg(long, global = true)]
    file: Option<PathBuf>,

    /// Optional path to a GraphCraft SQLite database file
    #[arg(long, global = true)]
    db: Option<PathBuf>,

    /// Output compact or formatted JSON
    #[arg(long, global = true, default_value_t = false)]
    json: bool,

    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand, Debug)]
enum CliCommand {
    /// Print workspace status
    Status,
    /// List or search nodes
    List {
        #[arg(long)]
        query: Option<String>,
        #[arg(long)]
        kind: Option<String>,
    },
    /// Compute network topology statistics
    Stats,
    /// Compute PageRank centrality scores
    Pagerank {
        #[arg(long, default_value_t = 0.85)]
        damping: f64,
    },
    /// Find shortest path between two nodes
    Path {
        source: String,
        target: String,
        #[arg(long, default_value_t = false)]
        directed: bool,
    },
    /// Export the graph to json, dot, mermaid, or gexf
    Export {
        #[arg(default_value = "mermaid")]
        format: String,
    },
    /// Run pure JSON-RPC MCP 2026-07-28 server over stdio
    Mcp,
}

fn parse_kind(raw: &str) -> Option<NodeKind> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "concept" => Some(NodeKind::Concept),
        "project" => Some(NodeKind::Project),
        "person" => Some(NodeKind::Person),
        "hardware" => Some(NodeKind::Hardware),
        "document" => Some(NodeKind::Document),
        "cluster" => Some(NodeKind::Cluster),
        "drawer" => Some(NodeKind::Drawer),
        _ => None,
    }
}

fn mcp_tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "graph_status",
                "description": "Get GraphCraft workspace status (title, node count, edge count).",
                "inputSchema": { "type": "object", "properties": {} }
            },
            {
                "name": "graph_list_nodes",
                "description": "List or filter nodes in the graph by search query or semantic kind.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" },
                        "kind": { "type": "string", "enum": ["concept", "project", "person", "hardware", "document", "cluster", "drawer"] }
                    }
                }
            },
            {
                "name": "graph_add_node",
                "description": "Add a new node to the knowledge graph.",
                "inputSchema": {
                    "type": "object",
                    "required": ["label"],
                    "properties": {
                        "id": { "type": "string" },
                        "label": { "type": "string" },
                        "kind": { "type": "string" },
                        "notes": { "type": "string" },
                        "tags": { "type": "array", "items": { "type": "string" } },
                        "x": { "type": "number" },
                        "y": { "type": "number" }
                    }
                }
            },
            {
                "name": "graph_add_edge",
                "description": "Connect two nodes with a directed, labeled relation edge.",
                "inputSchema": {
                    "type": "object",
                    "required": ["source", "target", "relation"],
                    "properties": {
                        "source": { "type": "string" },
                        "target": { "type": "string" },
                        "relation": { "type": "string" },
                        "weight": { "type": "number" }
                    }
                }
            },
            {
                "name": "graph_stats",
                "description": "Compute graph statistics, cycles, density, and top PageRank nodes.",
                "inputSchema": { "type": "object", "properties": {} }
            },
            {
                "name": "graph_shortest_path",
                "description": "Find shortest weighted path between two node IDs.",
                "inputSchema": {
                    "type": "object",
                    "required": ["source", "target"],
                    "properties": {
                        "source": { "type": "string" },
                        "target": { "type": "string" },
                        "directed": { "type": "boolean" }
                    }
                }
            },
            {
                "name": "graph_export",
                "description": "Export graph to json, dot (Graphviz), mermaid, or gexf (Gephi).",
                "inputSchema": {
                    "type": "object",
                    "required": ["format"],
                    "properties": {
                        "format": { "type": "string", "enum": ["json", "dot", "mermaid", "gexf"] }
                    }
                }
            }
        ]
    })
}

fn handle_mcp_call(engine: &mut GraphEngine, name: &str, args: &Value) -> Value {
    let cmd = match name {
        "graph_status" => Command::Status,
        "graph_list_nodes" => Command::ListNodes {
            query: args
                .get("query")
                .and_then(Value::as_str)
                .map(str::to_string),
            kind: args
                .get("kind")
                .and_then(Value::as_str)
                .and_then(parse_kind),
        },
        "graph_add_node" => Command::AddNode {
            id: args.get("id").and_then(Value::as_str).map(str::to_string),
            label: args
                .get("label")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            kind: args
                .get("kind")
                .and_then(Value::as_str)
                .and_then(parse_kind),
            notes: args
                .get("notes")
                .and_then(Value::as_str)
                .map(str::to_string),
            tags: args.get("tags").and_then(Value::as_array).map(|arr| {
                arr.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            }),
            x: args.get("x").and_then(Value::as_f64).map(|v| v as f32),
            y: args.get("y").and_then(Value::as_f64).map(|v| v as f32),
        },
        "graph_add_edge" => Command::AddEdge {
            id: None,
            source: args
                .get("source")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            target: args
                .get("target")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            relation: args
                .get("relation")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            weight: args.get("weight").and_then(Value::as_f64).map(|v| v as f32),
        },
        "graph_stats" => Command::ComputeStats,
        "graph_shortest_path" => Command::FindShortestPath {
            source: args
                .get("source")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            target: args
                .get("target")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            directed: args.get("directed").and_then(Value::as_bool),
        },
        "graph_export" => Command::Export {
            format: args
                .get("format")
                .and_then(Value::as_str)
                .unwrap_or("json")
                .to_string(),
        },
        unknown => {
            return json!({
                "isError": true,
                "content": [{ "type": "text", "text": format!("Unknown tool: {unknown}") }]
            });
        }
    };

    let result = dispatch(engine, cmd);
    let is_error = !result["ok"].as_bool().unwrap_or(false);
    let text = serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string());
    json!({
        "isError": is_error,
        "content": [{ "type": "text", "text": text }]
    })
}

pub fn process_jsonrpc_line(engine: &mut GraphEngine, line: &str) -> Option<Value> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let msg: Value = match serde_json::from_str(trimmed) {
        Ok(v) => v,
        Err(err) => {
            return Some(json!({
                "jsonrpc": "2.0",
                "id": Value::Null,
                "error": { "code": -32700, "message": format!("Parse error: {err}") }
            }));
        }
    };

    let id = msg.get("id").cloned();
    let method = msg.get("method").and_then(Value::as_str).unwrap_or("");

    // Notifications (no id) produce no JSON-RPC response
    if method.starts_with("notifications/") || id.is_none() {
        return None;
    }

    let req_id = id.unwrap_or(Value::Null);
    let params = msg.get("params").cloned().unwrap_or_else(|| json!({}));

    let response = match method {
        "initialize" => json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "protocolVersion": "2026-07-28",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "graphcraft-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        }),
        "ping" => json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {}
        }),
        "tools/list" => json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": mcp_tools_list()
        }),
        "tools/call" => {
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let result = handle_mcp_call(engine, name, &args);
            json!({
                "jsonrpc": "2.0",
                "id": req_id,
                "result": result
            })
        }
        other => json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "error": { "code": -32601, "message": format!("Method not found: {other}") }
        }),
    };

    Some(response)
}

fn run_mcp_stdio(engine: &mut GraphEngine, db_path: Option<&std::path::Path>) -> io::Result<()> {
    eprintln!("graphcraft-mcp stdio server listening (protocol 2026-07-28)");
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();

    for line_res in stdin.lock().lines() {
        let line = line_res?;
        if let Some(resp) = process_jsonrpc_line(engine, &line) {
            if let Some(path) = db_path {
                if let Err(err) = engine.save_db(path) {
                    eprintln!("Failed to persist db {}: {err}", path.display());
                }
            }
            let serialized = serde_json::to_string(&resp)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            writeln!(stdout, "{serialized}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    let db_path = cli
        .db
        .clone()
        .unwrap_or_else(graphcraft_core::db::default_db_path);
    let mut engine = match GraphEngine::open_or_seed(&db_path) {
        Ok(eng) => eng,
        Err(err) => {
            eprintln!("Failed to open db {}: {err}", db_path.display());
            GraphEngine::new_with_demo()
        }
    };

    if let Some(path) = cli.file.as_ref() {
        if let Err(err) = engine.load_from_file(path) {
            eprintln!("Failed to load {}: {err}", path.display());
            std::process::exit(1);
        }
    }

    if let CliCommand::Mcp = cli.command {
        let save_target = if cli.file.is_some() {
            None
        } else {
            Some(db_path.as_path())
        };
        if let Err(err) = run_mcp_stdio(&mut engine, save_target) {
            eprintln!("MCP stdio error: {err}");
            std::process::exit(1);
        }
        return;
    }

    let core_cmd = match cli.command {
        CliCommand::Status => Command::Status,
        CliCommand::List { query, kind } => Command::ListNodes {
            query,
            kind: kind.as_deref().and_then(parse_kind),
        },
        CliCommand::Stats => Command::ComputeStats,
        CliCommand::Pagerank { damping } => Command::ComputePagerank {
            damping: Some(damping),
            iterations: Some(25),
        },
        CliCommand::Path {
            source,
            target,
            directed,
        } => Command::FindShortestPath {
            source,
            target,
            directed: Some(directed),
        },
        CliCommand::Export { format } => Command::Export { format },
        CliCommand::Mcp => return,
    };

    let res = dispatch(&mut engine, core_cmd);
    if cli.json {
        if let Ok(text) = serde_json::to_string(&res) {
            println!("{text}");
        }
    } else if let Ok(text) = serde_json::to_string_pretty(&res) {
        println!("{text}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_initialize_and_tools_list_protocol() {
        let mut engine = GraphEngine::new_with_demo();
        let init_req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let init_resp = process_jsonrpc_line(&mut engine, init_req);
        assert!(init_resp.is_some());
        if let Some(v) = init_resp {
            assert_eq!(v["result"]["protocolVersion"], "2026-07-28");
            assert_eq!(v["result"]["serverInfo"]["name"], "graphcraft-mcp");
        }

        let list_req = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
        let list_resp = process_jsonrpc_line(&mut engine, list_req);
        assert!(list_resp.is_some());
        if let Some(v) = list_resp {
            let count = v["result"]["tools"].as_array().map(Vec::len).unwrap_or(0);
            assert_eq!(count, 7);
        }
    }

    #[test]
    fn mcp_tool_call_add_and_path() {
        let mut engine = GraphEngine::new_with_demo();
        let add_req = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"graph_add_node","arguments":{"id":"n_test","label":"Test Node","kind":"concept"}}}"#;
        let add_resp = process_jsonrpc_line(&mut engine, add_req);
        assert!(add_resp.is_some());
        if let Some(v) = add_resp {
            assert_eq!(v["result"]["isError"], false);
        }

        let path_req = r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"graph_shortest_path","arguments":{"source":"boomy","target":"storytold"}}}"#;
        let path_resp = process_jsonrpc_line(&mut engine, path_req);
        assert!(path_resp.is_some());
        if let Some(v) = path_resp {
            assert_eq!(v["result"]["isError"], false);
        }
    }

    #[test]
    fn mcp_notification_produces_no_output() {
        let mut engine = GraphEngine::new_with_demo();
        let notif = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        assert!(process_jsonrpc_line(&mut engine, notif).is_none());
    }
}
