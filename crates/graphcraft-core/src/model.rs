use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Semantic kind of a graph node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    #[default]
    Concept,
    Project,
    Person,
    Hardware,
    Document,
    Cluster,
    Drawer,
}

impl NodeKind {
    pub fn all() -> &'static [Self] {
        &[
            Self::Concept,
            Self::Project,
            Self::Person,
            Self::Hardware,
            Self::Document,
            Self::Cluster,
            Self::Drawer,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Concept => "Concept",
            Self::Project => "Project",
            Self::Person => "Person",
            Self::Hardware => "Hardware",
            Self::Document => "Document",
            Self::Cluster => "Cluster",
            Self::Drawer => "Drawer",
        }
    }
}

/// 2D world coordinate for canvas rendering and layout.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Vec2Pos {
    pub x: f32,
    pub y: f32,
}

impl Vec2Pos {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// A node in the GraphCraft knowledge graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub kind: NodeKind,
    pub notes: String,
    pub tags: Vec<String>,
    pub pos: Vec2Pos,
    #[serde(default = "default_weight")]
    pub weight: f32,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

fn default_weight() -> f32 {
    1.0
}

/// A directed, typed edge connecting two nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub relation: String,
    #[serde(default = "default_weight")]
    pub weight: f32,
}

/// Complete serializable snapshot of a GraphCraft workspace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphDocument {
    pub title: String,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

impl Default for GraphDocument {
    fn default() -> Self {
        Self {
            title: "Untitled Graph".to_string(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

/// Generates a rich initial demonstration graph (Vienna Open Robotics & Craft Ecosystem)
/// so first launch immediately displays an interactive, structured network.
pub fn demo_seed_document() -> GraphDocument {
    let nodes = vec![
        GraphNode {
            id: "saschirob".into(),
            label: "Saschirob Humanoid".into(),
            kind: NodeKind::Hardware,
            notes: "Open-source 1.85m bimanual humanoid on 680x540mm base (EUR 1,400 BOM).".into(),
            tags: vec!["robotics".into(), "rust".into(), "vienna".into()],
            pos: Vec2Pos::new(0.0, 0.0),
            weight: 2.0,
            metadata: BTreeMap::new(),
        },
        GraphNode {
            id: "boomy".into(),
            label: "Boomy Robocar".into(),
            kind: NodeKind::Hardware,
            notes: "Yahboom Raspbot v2 tabletop presenter with micro-projector & drawbridge box."
                .into(),
            tags: vec!["demo".into(), "physical".into()],
            pos: Vec2Pos::new(-220.0, -140.0),
            weight: 1.5,
            metadata: BTreeMap::new(),
        },
        GraphNode {
            id: "tu_wien".into(),
            label: "TU Wien Robotics Club".into(),
            kind: NodeKind::Cluster,
            notes: "Karlsplatz student robotics hub hosting the open humanoid live demo.".into(),
            tags: vec!["vienna".into(), "club".into()],
            pos: Vec2Pos::new(-260.0, 80.0),
            weight: 1.6,
            metadata: BTreeMap::new(),
        },
        GraphNode {
            id: "happylab".into(),
            label: "Happylab 1020".into(),
            kind: NodeKind::Cluster,
            notes: "Laser cutting (Trotec Speedy 400) & CNC aluminum chassis fabrication.".into(),
            tags: vec!["makerspace".into(), "fabrication".into()],
            pos: Vec2Pos::new(-180.0, 220.0),
            weight: 1.2,
            metadata: BTreeMap::new(),
        },
        GraphNode {
            id: "sandra".into(),
            label: "Sandra Schipal".into(),
            kind: NodeKind::Person,
            notes: "Principal systems architect, Craft fleet & Vienna Open Humanoid lead.".into(),
            tags: vec!["founder".into(), "rust".into()],
            pos: Vec2Pos::new(40.0, -210.0),
            weight: 1.8,
            metadata: BTreeMap::new(),
        },
        GraphNode {
            id: "steve".into(),
            label: "Stephan Schipal".into(),
            kind: NodeKind::Person,
            notes: "Collaborator, Hermes/OpenBrain architect, 2027 startup strategist.".into(),
            tags: vec!["partner".into(), "ai".into()],
            pos: Vec2Pos::new(260.0, -160.0),
            weight: 1.8,
            metadata: BTreeMap::new(),
        },
        GraphNode {
            id: "storytold".into(),
            label: "Storytold ArtCraft Beta".into(),
            kind: NodeKind::Concept,
            notes: "Pure-Rust desktop app ecosystem & craft-launcher distribution channel.".into(),
            tags: vec!["rust".into(), "ecosystem".into()],
            pos: Vec2Pos::new(240.0, 40.0),
            weight: 1.7,
            metadata: BTreeMap::new(),
        },
        GraphNode {
            id: "graphcraft".into(),
            label: "GraphCraft Studio".into(),
            kind: NodeKind::Project,
            notes: "Pure-Rust interactive knowledge graph & network canvas with MCP command bus."
                .into(),
            tags: vec!["craft".into(), "egui".into(), "mcp".into()],
            pos: Vec2Pos::new(140.0, 190.0),
            weight: 1.9,
            metadata: BTreeMap::new(),
        },
        GraphNode {
            id: "djcraft".into(),
            label: "DJCraft".into(),
            kind: NodeKind::Project,
            notes: "4-deck pure-Rust audio/video DJ performance studio.".into(),
            tags: vec!["craft".into(), "audio".into()],
            pos: Vec2Pos::new(360.0, 170.0),
            weight: 1.4,
            metadata: BTreeMap::new(),
        },
        GraphNode {
            id: "jeeves".into(),
            label: "Jeeves Homebot 2027".into(),
            kind: NodeKind::Concept,
            notes: "Domestic assistant hardware moat & 2027 startup hardware trajectory.".into(),
            tags: vec!["startup".into(), "moat".into()],
            pos: Vec2Pos::new(20.0, 260.0),
            weight: 1.6,
            metadata: BTreeMap::new(),
        },
        GraphNode {
            id: "backoffice_apps".into(),
            label: "Backoffice apps".into(),
            kind: NodeKind::Drawer,
            notes: "buchcraft\nteamcraft\ncontractcraft\ndeskcraft\nassetcraft\nbillcraft\nspendcraft\nhirecraft\nformcraft\nwikicraft\nhelmcraft\ngaincraft\ntaskcraft\nmoneycraft\nshopcraft\nstockcraft".into(),
            tags: vec!["backoffice".into(), "craft-biz".into(), "drawer".into()],
            pos: Vec2Pos::new(-280.0, -20.0),
            weight: 1.4,
            metadata: BTreeMap::new(),
        },
        GraphNode {
            id: "smbcraft".into(),
            label: "SMBCraft Umbrella".into(),
            kind: NodeKind::Project,
            notes: "Proposed Storytold Beta umbrella bundling the backoffice craft apps (Finance: billcraft/spendcraft/buchcraft; Ops: contractcraft/taskcraft/teamcraft; CRM: gaincraft) on craft-biz + SQLite.".into(),
            tags: vec!["smbcraft".into(), "umbrella".into(), "storytold".into()],
            pos: Vec2Pos::new(-110.0, 140.0),
            weight: 1.8,
            metadata: BTreeMap::new(),
        },
    ];

    let edges = vec![
        GraphEdge {
            id: "e1".into(),
            source: "sandra".into(),
            target: "saschirob".into(),
            relation: "architects".into(),
            weight: 1.5,
        },
        GraphEdge {
            id: "e2".into(),
            source: "sandra".into(),
            target: "graphcraft".into(),
            relation: "builds".into(),
            weight: 1.5,
        },
        GraphEdge {
            id: "e3".into(),
            source: "sandra".into(),
            target: "steve".into(),
            relation: "collaborates_with".into(),
            weight: 2.0,
        },
        GraphEdge {
            id: "e4".into(),
            source: "boomy".into(),
            target: "tu_wien".into(),
            relation: "presents_at".into(),
            weight: 1.2,
        },
        GraphEdge {
            id: "e5".into(),
            source: "boomy".into(),
            target: "saschirob".into(),
            relation: "demonstrates".into(),
            weight: 1.4,
        },
        GraphEdge {
            id: "e6".into(),
            source: "happylab".into(),
            target: "boomy".into(),
            relation: "fabricates_box".into(),
            weight: 1.0,
        },
        GraphEdge {
            id: "e7".into(),
            source: "happylab".into(),
            target: "saschirob".into(),
            relation: "cuts_chassis".into(),
            weight: 1.2,
        },
        GraphEdge {
            id: "e8".into(),
            source: "graphcraft".into(),
            target: "storytold".into(),
            relation: "targets_beta".into(),
            weight: 1.8,
        },
        GraphEdge {
            id: "e9".into(),
            source: "djcraft".into(),
            target: "storytold".into(),
            relation: "targets_beta".into(),
            weight: 1.6,
        },
        GraphEdge {
            id: "e10".into(),
            source: "steve".into(),
            target: "storytold".into(),
            relation: "evaluates".into(),
            weight: 1.4,
        },
        GraphEdge {
            id: "e11".into(),
            source: "saschirob".into(),
            target: "jeeves".into(),
            relation: "evolves_into".into(),
            weight: 1.7,
        },
        GraphEdge {
            id: "e12".into(),
            source: "steve".into(),
            target: "jeeves".into(),
            relation: "strategizes_moat".into(),
            weight: 1.5,
        },
        GraphEdge {
            id: "e13".into(),
            source: "sandra".into(),
            target: "backoffice_apps".into(),
            relation: "maintains".into(),
            weight: 1.6,
        },
        GraphEdge {
            id: "e14".into(),
            source: "backoffice_apps".into(),
            target: "smbcraft".into(),
            relation: "bundles_into".into(),
            weight: 1.9,
        },
        GraphEdge {
            id: "e15".into(),
            source: "smbcraft".into(),
            target: "storytold".into(),
            relation: "targets_beta".into(),
            weight: 1.8,
        },
    ];

    GraphDocument {
        title: "Vienna Open Robotics & Craft Ecosystem".into(),
        nodes,
        edges,
    }
}
