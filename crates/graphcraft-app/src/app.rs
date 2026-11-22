use eframe::egui::{self, Align2, Color32, CornerRadius, FontId, Pos2, Sense, Stroke, Vec2};
use graphcraft_core::algorithms::{
    compute_pagerank, compute_stats, find_shortest_path, GraphStats,
};
use graphcraft_core::layout::step_force_directed;
use graphcraft_core::model::NodeKind;
use graphcraft_core::{dispatch, Command, GraphEngine};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveTab {
    Canvas,
    Table,
    Export,
    Help,
}

pub struct GraphCraftApp {
    pub engine: GraphEngine,
    pub light_mode: bool,
    active_tab: ActiveTab,
    camera_offset: Vec2,
    zoom: f32,
    selected_node_id: Option<String>,
    dragging_node_id: Option<String>,
    physics_running: bool,
    search_query: String,

    // Creation forms
    new_node_label: String,
    new_node_kind: NodeKind,
    new_edge_source: String,
    new_edge_target: String,
    new_edge_relation: String,

    // Pathfinding
    path_source: String,
    path_target: String,
    highlighted_path: Vec<String>,
    highlighted_edges: HashSet<(String, String)>,

    // Cached analytics
    stats: GraphStats,
    pagerank_map: HashMap<String, f64>,

    // Export preview
    export_format: String,
    export_preview: String,

    // Status / modals
    status_message: String,
    show_about: bool,
    show_shortcuts: bool,
    show_command_palette: bool,
    palette_filter: String,
}

impl GraphCraftApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let app = Self::new_headless();
        apply_theme(&cc.egui_ctx, app.light_mode);
        app
    }

    pub fn new_headless() -> Self {
        let engine = GraphEngine::new_with_demo();
        let stats = compute_stats(&engine.doc);
        let pagerank_map = compute_pagerank(&engine.doc, 0.85, 25)
            .into_iter()
            .map(|s| (s.id, s.score))
            .collect();

        Self {
            engine,
            light_mode: false,
            active_tab: ActiveTab::Canvas,
            camera_offset: Vec2::ZERO,
            zoom: 1.0,
            selected_node_id: Some("saschirob".into()),
            dragging_node_id: None,
            physics_running: false,
            search_query: String::new(),
            new_node_label: String::new(),
            new_node_kind: NodeKind::Concept,
            new_edge_source: "sandra".into(),
            new_edge_target: "graphcraft".into(),
            new_edge_relation: "relates_to".into(),
            path_source: "boomy".into(),
            path_target: "storytold".into(),
            highlighted_path: Vec::new(),
            highlighted_edges: HashSet::new(),
            stats,
            pagerank_map,
            export_format: "mermaid".into(),
            export_preview: String::new(),
            status_message: "Ready - Pure-Rust GraphCraft Workspace".into(),
            show_about: false,
            show_shortcuts: false,
            show_command_palette: false,
            palette_filter: String::new(),
        }
    }

    fn refresh_analytics(&mut self) {
        self.stats = compute_stats(&self.engine.doc);
        self.pagerank_map = compute_pagerank(&self.engine.doc, 0.85, 25)
            .into_iter()
            .map(|s| (s.id, s.score))
            .collect();
    }

    pub fn run_cmd(&mut self, cmd: Command) -> serde_json::Value {
        let res = dispatch(&mut self.engine, cmd);
        self.refresh_analytics();
        if let Some(err) = res["error"].as_str() {
            self.status_message = format!("Error: {err}");
        }
        res
    }

    fn compute_path_highlight(&mut self) {
        match find_shortest_path(
            &self.engine.doc,
            &self.path_source,
            &self.path_target,
            false,
        ) {
            Ok(res) if res.found => {
                self.highlighted_edges.clear();
                for pair in res.path.windows(2) {
                    if let [a, b] = pair {
                        self.highlighted_edges.insert((a.clone(), b.clone()));
                        self.highlighted_edges.insert((b.clone(), a.clone()));
                    }
                }
                self.status_message = format!(
                    "Shortest path ({} hops, cost {:.2}): {}",
                    res.path.len().saturating_sub(1),
                    res.total_cost,
                    res.path.join(" -> ")
                );
                self.highlighted_path = res.path;
            }
            Ok(_) => {
                self.highlighted_path.clear();
                self.highlighted_edges.clear();
                self.status_message = format!(
                    "No path between '{}' and '{}'.",
                    self.path_source, self.path_target
                );
            }
            Err(err) => {
                self.highlighted_path.clear();
                self.highlighted_edges.clear();
                self.status_message = format!("Path error: {err}");
            }
        }
    }

    fn clear_path_highlight(&mut self) {
        self.highlighted_path.clear();
        self.highlighted_edges.clear();
    }

    fn refresh_export_preview(&mut self) {
        let res = dispatch(
            &mut self.engine,
            Command::Export {
                format: self.export_format.clone(),
            },
        );
        self.export_preview = res["data"]["content"]
            .as_str()
            .unwrap_or("Export error")
            .to_string();
    }

    fn world_to_screen(&self, center: Pos2, world: graphcraft_core::model::Vec2Pos) -> Pos2 {
        Pos2::new(
            center.x + (world.x + self.camera_offset.x) * self.zoom,
            center.y + (world.y + self.camera_offset.y) * self.zoom,
        )
    }

    fn node_color(&self, kind: NodeKind) -> Color32 {
        if self.light_mode {
            match kind {
                NodeKind::Hardware => Color32::from_rgb(180, 83, 9), // Amber 700
                NodeKind::Project => Color32::from_rgb(3, 105, 161), // Sky 700
                NodeKind::Person => Color32::from_rgb(21, 128, 61),  // Green 700
                NodeKind::Cluster => Color32::from_rgb(126, 34, 206), // Purple 700
                NodeKind::Document => Color32::from_rgb(190, 18, 60), // Rose 700
                NodeKind::Concept => Color32::from_rgb(51, 65, 85),  // Slate 700
            }
        } else {
            match kind {
                NodeKind::Hardware => Color32::from_rgb(245, 158, 11), // Amber 500
                NodeKind::Project => Color32::from_rgb(56, 189, 248),  // Sky 400
                NodeKind::Person => Color32::from_rgb(74, 222, 128),   // Green 400
                NodeKind::Cluster => Color32::from_rgb(192, 132, 252), // Purple 400
                NodeKind::Document => Color32::from_rgb(251, 113, 133), // Rose 400
                NodeKind::Concept => Color32::from_rgb(148, 163, 184), // Slate 400
            }
        }
    }

    fn render_menu_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Empty Graph").clicked() {
                    self.clear_path_highlight();
                    self.selected_node_id = None;
                    let _ = self.run_cmd(Command::ClearGraph);
                    self.status_message = "Created empty graph.".into();
                    ui.close();
                }
                if ui.button("Load Vienna Robotics Demo").clicked() {
                    self.clear_path_highlight();
                    let _ = self.run_cmd(Command::LoadDemoSeed);
                    self.selected_node_id = Some("saschirob".into());
                    self.status_message = "Loaded Vienna Open Robotics demo graph.".into();
                    ui.close();
                }
                ui.separator();
                if ui.button("Export Mermaid / DOT / GEXF...").clicked() {
                    self.active_tab = ActiveTab::Export;
                    self.refresh_export_preview();
                    ui.close();
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.menu_button("Edit", |ui| {
                if ui
                    .add_enabled(self.engine.can_undo(), egui::Button::new("Undo"))
                    .clicked()
                {
                    let _ = self.run_cmd(Command::Undo);
                    ui.close();
                }
                if ui
                    .add_enabled(self.engine.can_redo(), egui::Button::new("Redo"))
                    .clicked()
                {
                    let _ = self.run_cmd(Command::Redo);
                    ui.close();
                }
            });

            ui.menu_button("View", |ui| {
                if ui.button("Reset Camera (Center)").clicked() {
                    self.camera_offset = Vec2::ZERO;
                    self.zoom = 1.0;
                    ui.close();
                }
                if ui.button("Zoom In (+)").clicked() {
                    self.zoom = (self.zoom * 1.15).clamp(0.25, 3.0);
                    ui.close();
                }
                if ui.button("Zoom Out (-)").clicked() {
                    self.zoom = (self.zoom / 1.15).clamp(0.25, 3.0);
                    ui.close();
                }
                ui.separator();
                let theme_label = if self.light_mode {
                    "Switch to Dark Mode"
                } else {
                    "Switch to Light Mode"
                };
                if ui.button(theme_label).clicked() {
                    self.light_mode = !self.light_mode;
                    apply_theme(ctx, self.light_mode);
                    ui.close();
                }
            });

            ui.menu_button("Graph", |ui| {
                if ui.button("Run Force Physics (30 steps)").clicked() {
                    let _ = self.run_cmd(Command::RunLayout {
                        mode: "force".into(),
                        steps: Some(30),
                    });
                    ui.close();
                }
                if ui.button("Apply Circular Layout").clicked() {
                    let _ = self.run_cmd(Command::RunLayout {
                        mode: "circular".into(),
                        steps: None,
                    });
                    ui.close();
                }
                if ui.button("Apply Grid Layout").clicked() {
                    let _ = self.run_cmd(Command::RunLayout {
                        mode: "grid".into(),
                        steps: None,
                    });
                    ui.close();
                }
            });

            ui.menu_button("Help", |ui| {
                if ui.button("Command Palette (Ctrl+K)").clicked() {
                    self.show_command_palette = true;
                    ui.close();
                }
                if ui.button("Keyboard Shortcuts").clicked() {
                    self.show_shortcuts = true;
                    ui.close();
                }
                if ui.button("Project Files & Docs").clicked() {
                    self.active_tab = ActiveTab::Help;
                    ui.close();
                }
                ui.separator();
                if ui.button("About GraphCraft").clicked() {
                    self.show_about = true;
                    ui.close();
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let theme_btn = if self.light_mode { "Dark" } else { "Light" };
                if ui
                    .button(theme_btn)
                    .on_hover_text("Toggle Light/Dark theme")
                    .clicked()
                {
                    self.light_mode = !self.light_mode;
                    apply_theme(ctx, self.light_mode);
                }

                ui.label(format!(
                    "Zoom: {:.0}% | Nodes: {} | Edges: {}",
                    self.zoom * 100.0,
                    self.stats.node_count,
                    self.stats.edge_count
                ));
            });
        });
    }

    fn render_left_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.heading("Graph Studio");
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.active_tab, ActiveTab::Canvas, "Canvas");
            ui.selectable_value(&mut self.active_tab, ActiveTab::Table, "Nodes");
            ui.selectable_value(&mut self.active_tab, ActiveTab::Export, "Export");
            ui.selectable_value(&mut self.active_tab, ActiveTab::Help, "Help");
        });
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            let filter_lbl = ui.strong("Quick Filter");
            ui.add(egui::TextEdit::singleline(&mut self.search_query).hint_text("Filter nodes..."))
                .labelled_by(filter_lbl.id);
            ui.add_space(6.0);

            ui.strong("Physics & Layout");
            ui.horizontal(|ui| {
                let phys_lbl = if self.physics_running {
                    "Pause Physics"
                } else {
                    "Live Physics"
                };
                if ui.button(phys_lbl).clicked() {
                    self.physics_running = !self.physics_running;
                }
                if ui.button("Circle").clicked() {
                    let _ = self.run_cmd(Command::RunLayout {
                        mode: "circular".into(),
                        steps: None,
                    });
                }
                if ui.button("Grid").clicked() {
                    let _ = self.run_cmd(Command::RunLayout {
                        mode: "grid".into(),
                        steps: None,
                    });
                }
            });
            ui.add_space(8.0);

            ui.strong("Add Node");
            ui.horizontal(|ui| {
                let lbl = ui.label("Label:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_node_label).hint_text("Node title"),
                )
                .labelled_by(lbl.id);
            });
            egui::ComboBox::from_label("Kind")
                .selected_text(self.new_node_kind.label())
                .show_ui(ui, |ui| {
                    for &k in NodeKind::all() {
                        ui.selectable_value(&mut self.new_node_kind, k, k.label());
                    }
                });
            if ui.button("+ Create Node").clicked() && !self.new_node_label.trim().is_empty() {
                let label = self.new_node_label.clone();
                let kind = self.new_node_kind;
                let res = self.run_cmd(Command::AddNode {
                    id: None,
                    label,
                    kind: Some(kind),
                    notes: None,
                    tags: None,
                    x: Some(0.0),
                    y: Some(0.0),
                });
                if let Some(new_id) = res["data"]["added"]["id"].as_str() {
                    self.selected_node_id = Some(new_id.to_string());
                    self.new_node_label.clear();
                }
            }

            ui.add_space(8.0);
            ui.strong("Connect Nodes");
            ui.horizontal(|ui| {
                let lbl = ui.label("From:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_edge_source).hint_text("Source ID"),
                )
                .labelled_by(lbl.id);
            });
            ui.horizontal(|ui| {
                let lbl = ui.label("To:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_edge_target).hint_text("Target ID"),
                )
                .labelled_by(lbl.id);
            });
            ui.horizontal(|ui| {
                let lbl = ui.label("Rel:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_edge_relation).hint_text("Relation"),
                )
                .labelled_by(lbl.id);
            });
            if ui.button("+ Connect Edge").clicked() {
                let _ = self.run_cmd(Command::AddEdge {
                    id: None,
                    source: self.new_edge_source.clone(),
                    target: self.new_edge_target.clone(),
                    relation: self.new_edge_relation.clone(),
                    weight: Some(1.0),
                });
            }

            ui.add_space(8.0);
            ui.strong("Shortest Path (Dijkstra)");
            ui.horizontal(|ui| {
                let lbl = ui.label("Src:");
                ui.add(egui::TextEdit::singleline(&mut self.path_source).hint_text("Start ID"))
                    .labelled_by(lbl.id);
            });
            ui.horizontal(|ui| {
                let lbl = ui.label("Dst:");
                ui.add(egui::TextEdit::singleline(&mut self.path_target).hint_text("End ID"))
                    .labelled_by(lbl.id);
            });
            ui.horizontal(|ui| {
                if ui.button("Find Path").clicked() {
                    self.compute_path_highlight();
                }
                if ui.button("Clear").clicked() {
                    self.clear_path_highlight();
                }
            });

            ui.add_space(8.0);
            ui.strong("Network Analytics");
            ui.label(format!("Components: {}", self.stats.connected_components));
            ui.label(format!("Has Cycles: {}", self.stats.has_cycles));
            ui.label(format!("Density: {:.3}", self.stats.density));
            ui.add_space(4.0);
            ui.label("Top PageRank Nodes:");
            for score in &self.stats.top_pagerank {
                ui.small(format!("  {} ({:.3})", score.label, score.score));
            }
        });
    }

    fn render_right_inspector(&mut self, ui: &mut egui::Ui) {
        ui.heading("Node Inspector");
        ui.separator();

        let Some(sel_id) = self.selected_node_id.clone() else {
            ui.label("Click any node on the canvas to inspect or edit its metadata.");
            return;
        };

        let Some(node_clone) = self
            .engine
            .doc
            .nodes
            .iter()
            .find(|n| n.id == sel_id)
            .cloned()
        else {
            ui.label("Selected node no longer exists.");
            return;
        };

        let mut label_buf = node_clone.label.clone();
        let mut notes_buf = node_clone.notes.clone();
        let mut kind_buf = node_clone.kind;

        ui.label(format!("ID: {}", node_clone.id));
        if let Some(pr) = self.pagerank_map.get(&node_clone.id) {
            ui.label(format!("PageRank Score: {:.3}", pr));
        }
        ui.label(format!(
            "Position: ({:.0}, {:.0})",
            node_clone.pos.x, node_clone.pos.y
        ));
        ui.add_space(6.0);

        ui.strong("Label");
        let label_changed = ui.text_edit_singleline(&mut label_buf).changed();

        ui.add_space(4.0);
        ui.strong("Semantic Kind");
        let mut kind_changed = false;
        egui::ComboBox::from_id_salt("inspector_kind")
            .selected_text(kind_buf.label())
            .show_ui(ui, |ui| {
                for &k in NodeKind::all() {
                    if ui.selectable_value(&mut kind_buf, k, k.label()).clicked() {
                        kind_changed = true;
                    }
                }
            });

        ui.add_space(4.0);
        ui.strong("Markdown Notes");
        let notes_changed = ui
            .add(
                egui::TextEdit::multiline(&mut notes_buf)
                    .desired_rows(6)
                    .desired_width(f32::INFINITY),
            )
            .changed();

        if label_changed || kind_changed || notes_changed {
            let _ = self.run_cmd(Command::UpdateNode {
                id: sel_id.clone(),
                label: if label_changed && !label_buf.trim().is_empty() {
                    Some(label_buf)
                } else {
                    None
                },
                kind: if kind_changed { Some(kind_buf) } else { None },
                notes: if notes_changed { Some(notes_buf) } else { None },
                tags: None,
                x: None,
                y: None,
            });
        }

        ui.add_space(10.0);
        ui.strong("Connections");
        let mut delete_edge_id: Option<String> = None;
        for edge in &self.engine.doc.edges {
            if edge.source == sel_id || edge.target == sel_id {
                ui.horizontal(|ui| {
                    let text = if edge.source == sel_id {
                        format!("--[{}]--> {}", edge.relation, edge.target)
                    } else {
                        format!("{} --[{}]-->", edge.source, edge.relation)
                    };
                    ui.small(text);
                    if ui.small_button("x").clicked() {
                        delete_edge_id = Some(edge.id.clone());
                    }
                });
            }
        }
        if let Some(eid) = delete_edge_id {
            let _ = self.run_cmd(Command::DeleteEdge { id: eid });
        }

        ui.add_space(14.0);
        if ui.button("Delete Selected Node").clicked() {
            let _ = self.run_cmd(Command::DeleteNode { id: sel_id });
            self.selected_node_id = None;
        }
    }

    fn render_canvas(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if self.physics_running {
            step_force_directed(&mut self.engine.doc, 0.35);
            ctx.request_repaint();
        }

        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        let bg = if self.light_mode {
            Color32::from_rgb(248, 250, 252)
        } else {
            Color32::from_rgb(15, 23, 42)
        };
        painter.rect_filled(rect, 0.0, bg);

        let center = rect.center();
        let query = self.search_query.trim().to_ascii_lowercase();

        // Subtle grid dots
        let grid_step = (40.0 * self.zoom).max(16.0);
        let grid_color = if self.light_mode {
            Color32::from_rgba_unmultiplied(100, 116, 139, 45)
        } else {
            Color32::from_rgba_unmultiplied(148, 163, 184, 30)
        };
        let mut gx =
            rect.left() + (center.x - rect.left() + self.camera_offset.x * self.zoom) % grid_step;
        while gx < rect.right() {
            let mut gy =
                rect.top() + (center.y - rect.top() + self.camera_offset.y * self.zoom) % grid_step;
            while gy < rect.bottom() {
                painter.circle_filled(Pos2::new(gx, gy), 1.0, grid_color);
                gy += grid_step;
            }
            gx += grid_step;
        }

        // Build position map
        let mut pos_map = HashMap::with_capacity(self.engine.doc.nodes.len());
        for node in &self.engine.doc.nodes {
            pos_map.insert(node.id.as_str(), self.world_to_screen(center, node.pos));
        }

        // 1. Draw Edges
        let edge_base_color = if self.light_mode {
            Color32::from_rgb(71, 85, 105)
        } else {
            Color32::from_rgb(100, 116, 139)
        };
        let edge_label_color = if self.light_mode {
            Color32::from_rgb(71, 85, 105)
        } else {
            Color32::from_rgb(203, 213, 225)
        };
        let highlight_color = Color32::from_rgb(56, 189, 248);

        for edge in &self.engine.doc.edges {
            if let (Some(&p1), Some(&p2)) = (
                pos_map.get(edge.source.as_str()),
                pos_map.get(edge.target.as_str()),
            ) {
                let is_hi = self
                    .highlighted_edges
                    .contains(&(edge.source.clone(), edge.target.clone()));
                let stroke = if is_hi {
                    Stroke::new(3.0 * self.zoom.sqrt(), highlight_color)
                } else {
                    Stroke::new(1.5 * self.zoom.sqrt(), edge_base_color)
                };

                painter.line_segment([p1, p2], stroke);

                // Arrowhead near target
                let dir = (p2 - p1).normalized();
                let tip = p2 - dir * (20.0 * self.zoom);
                let perp = Vec2::new(-dir.y, dir.x);
                let arrow_len = 8.0 * self.zoom;
                let arrow_w = 4.5 * self.zoom;
                painter.line_segment([tip, tip - dir * arrow_len + perp * arrow_w], stroke);
                painter.line_segment([tip, tip - dir * arrow_len - perp * arrow_w], stroke);

                // Relation label at midpoint
                let mid = Pos2::new((p1.x + p2.x) * 0.5, (p1.y + p2.y) * 0.5);
                if self.zoom >= 0.65 {
                    painter.text(
                        mid,
                        Align2::CENTER_CENTER,
                        &edge.relation,
                        FontId::proportional(10.0 * self.zoom.clamp(0.7, 1.3)),
                        if is_hi {
                            highlight_color
                        } else {
                            edge_label_color
                        },
                    );
                }
            }
        }

        // 2. Hit-test interaction (node click / drag vs background pan)
        let pointer_pos = ctx.input(|i| i.pointer.hover_pos());
        if response.drag_started_by(egui::PointerButton::Primary) {
            if let Some(mouse) = pointer_pos {
                let hit = self.engine.doc.nodes.iter().rev().find(|n| {
                    let sp = self.world_to_screen(center, n.pos);
                    sp.distance(mouse) <= 24.0 * self.zoom
                });
                if let Some(n) = hit {
                    self.dragging_node_id = Some(n.id.clone());
                    self.selected_node_id = Some(n.id.clone());
                } else {
                    self.dragging_node_id = None;
                }
            }
        }

        if response.dragged_by(egui::PointerButton::Primary) {
            let delta = response.drag_delta() / self.zoom;
            if let Some(ref drag_id) = self.dragging_node_id {
                if let Some(node) = self.engine.doc.nodes.iter_mut().find(|n| &n.id == drag_id) {
                    node.pos.x += delta.x;
                    node.pos.y += delta.y;
                }
            } else {
                self.camera_offset += delta;
            }
        }

        if response.drag_stopped() {
            self.dragging_node_id = None;
        }

        if response.clicked() {
            if let Some(mouse) = pointer_pos {
                let hit = self.engine.doc.nodes.iter().rev().find(|n| {
                    let sp = self.world_to_screen(center, n.pos);
                    sp.distance(mouse) <= 24.0 * self.zoom
                });
                self.selected_node_id = hit.map(|n| n.id.clone());
            }
        }

        // 3. Draw Nodes
        let text_color = if self.light_mode {
            Color32::from_rgb(15, 23, 42)
        } else {
            Color32::from_rgb(248, 250, 252)
        };
        let node_fill = if self.light_mode {
            Color32::WHITE
        } else {
            Color32::from_rgb(30, 41, 59)
        };

        for node in &self.engine.doc.nodes {
            let sp = self.world_to_screen(center, node.pos);
            let pr = self.pagerank_map.get(&node.id).copied().unwrap_or(0.1);
            let radius = ((16.0 + (pr as f32) * 55.0).clamp(15.0, 32.0)) * self.zoom;
            let accent = self.node_color(node.kind);

            let is_selected = self.selected_node_id.as_deref() == Some(&node.id);
            let is_in_path = self.highlighted_path.contains(&node.id);
            let matches_query = !query.is_empty()
                && (node.label.to_ascii_lowercase().contains(&query)
                    || node.id.to_ascii_lowercase().contains(&query));

            if is_selected || is_in_path || matches_query {
                let ring_color = if is_in_path {
                    highlight_color
                } else if matches_query {
                    Color32::from_rgb(34, 197, 94)
                } else {
                    Color32::from_rgb(251, 191, 36)
                };
                painter.circle_stroke(sp, radius + 4.0 * self.zoom, Stroke::new(2.5, ring_color));
            }

            painter.circle_filled(sp, radius, node_fill);
            painter.circle_stroke(sp, radius, Stroke::new(2.5 * self.zoom.sqrt(), accent));

            painter.text(
                Pos2::new(sp.x, sp.y + radius + 12.0 * self.zoom),
                Align2::CENTER_CENTER,
                &node.label,
                FontId::proportional(12.5 * self.zoom.clamp(0.7, 1.4)),
                text_color,
            );
        }
    }

    fn render_table_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Graph Nodes Directory");
        ui.separator();

        let query = self.search_query.trim().to_ascii_lowercase();
        let mut clicked_id = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("nodes_table")
                .striped(true)
                .min_col_width(120.0)
                .show(ui, |ui| {
                    ui.strong("ID");
                    ui.strong("Label");
                    ui.strong("Kind");
                    ui.strong("PageRank");
                    ui.strong("Notes");
                    ui.end_row();

                    for node in &self.engine.doc.nodes {
                        if !query.is_empty()
                            && !node.label.to_ascii_lowercase().contains(&query)
                            && !node.id.to_ascii_lowercase().contains(&query)
                        {
                            continue;
                        }
                        if ui.link(&node.id).clicked() {
                            clicked_id = Some(node.id.clone());
                        }
                        ui.label(&node.label);
                        ui.label(node.kind.label());
                        let pr = self.pagerank_map.get(&node.id).copied().unwrap_or(0.0);
                        ui.label(format!("{pr:.3}"));
                        ui.label(&node.notes);
                        ui.end_row();
                    }
                });
        });

        if let Some(id) = clicked_id {
            self.selected_node_id = Some(id);
            self.active_tab = ActiveTab::Canvas;
        }
    }

    fn render_export_view(&mut self, ui: &mut egui::Ui) {
        let export_hdr = ui.heading("Interchange & Export");
        ui.separator();

        ui.horizontal(|ui| {
            for fmt in ["mermaid", "dot", "gexf", "json"] {
                if ui
                    .selectable_label(self.export_format == fmt, fmt.to_uppercase())
                    .clicked()
                {
                    self.export_format = fmt.to_string();
                    self.refresh_export_preview();
                }
            }
            if ui.button("Refresh").clicked() {
                self.refresh_export_preview();
            }
            if ui.button("Copy to Clipboard").clicked() {
                ui.ctx().copy_text(self.export_preview.clone());
                self.status_message = format!("Copied {} export to clipboard.", self.export_format);
            }
        });

        if self.export_preview.is_empty() {
            self.refresh_export_preview();
        }

        ui.add_space(6.0);
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.export_preview)
                    .code_editor()
                    .hint_text("Export preview")
                    .desired_width(f32::INFINITY)
                    .desired_rows(26),
            )
            .labelled_by(export_hdr.id);
        });
    }

    fn render_help_view(&self, ui: &mut egui::Ui) {
        ui.heading("GraphCraft Documentation & Architecture");
        ui.separator();
        ui.label("Pure-Rust Knowledge Graph, Network Analysis & Interactive Node Canvas.");
        ui.add_space(8.0);
        ui.strong("Architecture Layers:");
        ui.label("- L1 (graphcraft-core): petgraph algorithms (PageRank, Dijkstra), physics layouts, DOT/Mermaid/GEXF export.");
        ui.label("- L2 (graphcraft-mcp): CLI-first binary (`--json`) + pure JSON-RPC MCP 2024-11-05 stdio server.");
        ui.label("- L3 (graphcraft-app): Pure-Rust eframe/egui native GPU desktop app calling L1 directly in-process.");
        ui.add_space(8.0);
        ui.strong("Controls:");
        ui.label("- Drag background: Pan camera");
        ui.label("- Drag node: Move node interactively");
        ui.label("- Ctrl + Mouse Wheel: Zoom canvas / UI scale");
        ui.label("- Ctrl + K: Open Command Palette");
    }
}

impl eframe::App for GraphCraftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Ctrl + Mouse Wheel zoom (Craft Fleet Rule)
        let scroll_zoom = ctx.input(|i| {
            if i.modifiers.ctrl {
                i.smooth_scroll_delta.y
            } else {
                0.0
            }
        });
        if scroll_zoom.abs() > 0.1 {
            let factor = if scroll_zoom > 0.0 { 1.08 } else { 0.92 };
            self.zoom = (self.zoom * factor).clamp(0.25, 3.5);
        }

        // Ctrl + K shortcut for Command Palette
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::K)) {
            self.show_command_palette = !self.show_command_palette;
        }

        egui::Panel::top("top_menu").show(ui, |ui| {
            self.render_menu_bar(&ctx, ui);
        });

        egui::Panel::bottom("status_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_message);
            });
        });

        egui::Panel::left("left_panel")
            .resizable(true)
            .default_size(250.0)
            .show(ui, |ui| {
                self.render_left_sidebar(ui);
            });

        egui::Panel::right("right_panel")
            .resizable(true)
            .default_size(260.0)
            .show(ui, |ui| {
                self.render_right_inspector(ui);
            });

        egui::CentralPanel::default().show(ui, |ui| match self.active_tab {
            ActiveTab::Canvas => self.render_canvas(&ctx, ui),
            ActiveTab::Table => self.render_table_view(ui),
            ActiveTab::Export => self.render_export_view(ui),
            ActiveTab::Help => self.render_help_view(ui),
        });

        // Modals
        if self.show_about {
            egui::Window::new("About GraphCraft")
                .collapsible(false)
                .resizable(false)
                .open(&mut self.show_about)
                .show(&ctx, |ui| {
                    ui.heading("GraphCraft v0.1.0");
                    ui.label("Pure-Rust Knowledge Graph & Network Studio");
                    ui.label("Dual-licensed MIT OR Apache-2.0");
                    ui.label("Clean-room ArtCraft community beta candidate.");
                });
        }

        if self.show_shortcuts {
            egui::Window::new("Keyboard Shortcuts")
                .collapsible(false)
                .resizable(false)
                .open(&mut self.show_shortcuts)
                .show(&ctx, |ui| {
                    ui.label("Ctrl + K : Command Palette");
                    ui.label("Ctrl + Scroll : Zoom Canvas");
                    ui.label("Primary Drag (Node) : Move Node");
                    ui.label("Primary Drag (Canvas) : Pan Camera");
                });
        }

        if self.show_command_palette {
            let mut open = self.show_command_palette;
            let mut close_now = false;
            egui::Window::new("Command Palette")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(&ctx, |ui| {
                    ui.text_edit_singleline(&mut self.palette_filter);
                    ui.separator();
                    let actions = [
                        ("Run Force Physics", "layout_force"),
                        ("Apply Circular Layout", "layout_circle"),
                        ("Apply Grid Layout", "layout_grid"),
                        ("Toggle Light/Dark Theme", "toggle_theme"),
                        ("Load Vienna Demo Graph", "load_demo"),
                        ("Open Export View", "open_export"),
                    ];
                    let pf = self.palette_filter.to_ascii_lowercase();
                    for (label, code) in actions {
                        if !pf.is_empty() && !label.to_ascii_lowercase().contains(&pf) {
                            continue;
                        }
                        if ui.button(label).clicked() {
                            match code {
                                "layout_force" => {
                                    let _ = self.run_cmd(Command::RunLayout {
                                        mode: "force".into(),
                                        steps: Some(30),
                                    });
                                }
                                "layout_circle" => {
                                    let _ = self.run_cmd(Command::RunLayout {
                                        mode: "circular".into(),
                                        steps: None,
                                    });
                                }
                                "layout_grid" => {
                                    let _ = self.run_cmd(Command::RunLayout {
                                        mode: "grid".into(),
                                        steps: None,
                                    });
                                }
                                "toggle_theme" => {
                                    self.light_mode = !self.light_mode;
                                    apply_theme(&ctx, self.light_mode);
                                }
                                "load_demo" => {
                                    let _ = self.run_cmd(Command::LoadDemoSeed);
                                }
                                "open_export" => {
                                    self.active_tab = ActiveTab::Export;
                                    self.refresh_export_preview();
                                }
                                _ => {}
                            }
                            close_now = true;
                        }
                    }
                });
            self.show_command_palette = open && !close_now;
        }
    }
}

pub fn apply_theme(ctx: &egui::Context, light: bool) {
    let mut visuals = if light {
        egui::Visuals::light()
    } else {
        egui::Visuals::dark()
    };

    if light {
        visuals.override_text_color = Some(Color32::from_rgb(15, 23, 42));
        visuals.panel_fill = Color32::from_rgb(255, 255, 255);
        visuals.window_fill = Color32::from_rgb(248, 250, 252);
        visuals.widgets.noninteractive.bg_stroke =
            Stroke::new(1.0, Color32::from_rgb(203, 213, 225));
    } else {
        visuals.override_text_color = Some(Color32::from_rgb(248, 250, 252));
        visuals.panel_fill = Color32::from_rgb(15, 23, 42);
        visuals.window_fill = Color32::from_rgb(30, 41, 59);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(51, 65, 85));
    }
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(4);
    ctx.set_visuals(visuals);
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::Harness;

    #[test]
    fn app_parity_matches_core_dispatch() {
        let mut app = GraphCraftApp::new_headless();
        let mut core = GraphEngine::new_with_demo();

        let app_res = app.run_cmd(Command::AddNode {
            id: Some("parity_n".into()),
            label: "Parity Node".into(),
            kind: Some(NodeKind::Concept),
            notes: None,
            tags: None,
            x: None,
            y: None,
        });
        let core_res = dispatch(
            &mut core,
            Command::AddNode {
                id: Some("parity_n".into()),
                label: "Parity Node".into(),
                kind: Some(NodeKind::Concept),
                notes: None,
                tags: None,
                x: None,
                y: None,
            },
        );
        assert_eq!(app_res, core_res);
    }

    #[test]
    fn headless_egui_kittest_renders_without_panic() {
        let mut app = GraphCraftApp::new_headless();
        let mut harness = Harness::new_ui(|ui| {
            apply_theme(ui.ctx(), app.light_mode);
            egui::CentralPanel::default().show(ui, |ui| {
                app.render_left_sidebar(ui);
            });
        });
        harness.run();
    }

    #[test]
    fn headless_egui_kittest_light_mode_export_view() {
        let mut app = GraphCraftApp::new_headless();
        app.light_mode = true;
        let mut harness = Harness::new_ui(|ui| {
            apply_theme(ui.ctx(), app.light_mode);
            egui::CentralPanel::default().show(ui, |ui| {
                app.render_export_view(ui);
            });
        });
        harness.run();
    }
}
