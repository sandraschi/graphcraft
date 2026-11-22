use crate::error::GraphError;
use crate::model::GraphDocument;

/// Supported interchange formats for GraphCraft.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Dot,
    Mermaid,
    Gexf,
}

impl ExportFormat {
    pub fn parse(raw: &str) -> Result<Self, GraphError> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "dot" | "graphviz" => Ok(Self::Dot),
            "mermaid" | "mmd" => Ok(Self::Mermaid),
            "gexf" | "gephi" => Ok(Self::Gexf),
            other => Err(GraphError::Validation(format!(
                "Unsupported export format '{other}'. Expected json, dot, mermaid, or gexf."
            ))),
        }
    }
}

fn sanitize_ident(raw: &str) -> String {
    let clean: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if clean.is_empty() {
        "node".to_string()
    } else {
        clean
    }
}

fn escape_xml(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_quoted(raw: &str) -> String {
    raw.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Exports a `GraphDocument` to JSON, Graphviz DOT, Mermaid flowchart, or Gephi GEXF.
pub fn export_graph(doc: &GraphDocument, format: ExportFormat) -> Result<String, GraphError> {
    match format {
        ExportFormat::Json => {
            serde_json::to_string_pretty(doc).map_err(|e| GraphError::Serialization(e.to_string()))
        }
        ExportFormat::Dot => {
            let mut out = String::from("digraph GraphCraft {\n  rankdir=LR;\n");
            for n in &doc.nodes {
                out.push_str(&format!(
                    "  \"{}\" [label=\"{}\"];\n",
                    escape_quoted(&n.id),
                    escape_quoted(&n.label)
                ));
            }
            for e in &doc.edges {
                out.push_str(&format!(
                    "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
                    escape_quoted(&e.source),
                    escape_quoted(&e.target),
                    escape_quoted(&e.relation)
                ));
            }
            out.push_str("}\n");
            Ok(out)
        }
        ExportFormat::Mermaid => {
            let mut out = String::from("flowchart LR\n");
            for n in &doc.nodes {
                out.push_str(&format!(
                    "  {}[\"{}\"]\n",
                    sanitize_ident(&n.id),
                    escape_quoted(&n.label)
                ));
            }
            for e in &doc.edges {
                out.push_str(&format!(
                    "  {} -->|\"{}\"| {}\n",
                    sanitize_ident(&e.source),
                    escape_quoted(&e.relation),
                    sanitize_ident(&e.target)
                ));
            }
            Ok(out)
        }
        ExportFormat::Gexf => {
            let mut out = String::from(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
                 <gexf xmlns=\"http://gexf.net/1.3\" version=\"1.3\">\n\
                 \x20 <graph mode=\"static\" defaultedgetype=\"directed\">\n\
                 \x20   <nodes>\n",
            );
            for n in &doc.nodes {
                out.push_str(&format!(
                    "      <node id=\"{}\" label=\"{}\" />\n",
                    escape_xml(&n.id),
                    escape_xml(&n.label)
                ));
            }
            out.push_str("    </nodes>\n    <edges>\n");
            for e in &doc.edges {
                out.push_str(&format!(
                    "      <edge id=\"{}\" source=\"{}\" target=\"{}\" label=\"{}\" weight=\"{}\" />\n",
                    escape_xml(&e.id),
                    escape_xml(&e.source),
                    escape_xml(&e.target),
                    escape_xml(&e.relation),
                    e.weight
                ));
            }
            out.push_str("    </edges>\n  </graph>\n</gexf>\n");
            Ok(out)
        }
    }
}
