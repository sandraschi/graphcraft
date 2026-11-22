use crate::model::{GraphDocument, Vec2Pos};
use std::collections::HashMap;
use std::f32::consts::TAU;

/// Executes one iteration of a Fruchterman-Reingold force-directed layout step.
pub fn step_force_directed(doc: &mut GraphDocument, dt: f32) {
    let n = doc.nodes.len();
    if n < 2 {
        return;
    }

    let k = 150.0_f32;
    let repulse_strength = k * k;
    let attract_strength = 0.08_f32;
    let center_gravity = 0.025_f32;
    let max_step = 35.0_f32 * dt.clamp(0.1, 2.0);

    let mut forces = vec![Vec2Pos::new(0.0, 0.0); n];
    let mut id_to_idx = HashMap::with_capacity(n);
    for (i, node) in doc.nodes.iter().enumerate() {
        id_to_idx.insert(node.id.as_str(), i);
    }

    // 1. Pairwise repulsive force
    for i in 0..n {
        for j in (i + 1)..n {
            let dx = doc.nodes[i].pos.x - doc.nodes[j].pos.x;
            let dy = doc.nodes[i].pos.y - doc.nodes[j].pos.y;
            let dist_sq = (dx * dx + dy * dy).max(16.0);
            let dist = dist_sq.sqrt();
            let force = repulse_strength / dist_sq;
            let fx = (dx / dist) * force;
            let fy = (dy / dist) * force;

            forces[i].x += fx;
            forces[i].y += fy;
            forces[j].x -= fx;
            forces[j].y -= fy;
        }
    }

    // 2. Edge spring attraction
    for edge in &doc.edges {
        if let (Some(&u), Some(&v)) = (
            id_to_idx.get(edge.source.as_str()),
            id_to_idx.get(edge.target.as_str()),
        ) {
            if u == v {
                continue;
            }
            let dx = doc.nodes[v].pos.x - doc.nodes[u].pos.x;
            let dy = doc.nodes[v].pos.y - doc.nodes[u].pos.y;
            let dist = (dx * dx + dy * dy).sqrt().max(1.0);
            let stretch = (dist - k) * attract_strength;
            let fx = (dx / dist) * stretch;
            let fy = (dy / dist) * stretch;

            forces[u].x += fx;
            forces[u].y += fy;
            forces[v].x -= fx;
            forces[v].y -= fy;
        }
    }

    // 3. Gentle pull toward origin so disconnected components do not drift away
    for (i, node) in doc.nodes.iter().enumerate() {
        forces[i].x -= node.pos.x * center_gravity;
        forces[i].y -= node.pos.y * center_gravity;
    }

    // 4. Integrate with step clamping
    for (i, node) in doc.nodes.iter_mut().enumerate() {
        let fx = forces[i].x;
        let fy = forces[i].y;
        let mag = (fx * fx + fy * fy).sqrt();
        if mag > 0.001 {
            let step = mag.min(max_step);
            node.pos.x += (fx / mag) * step;
            node.pos.y += (fy / mag) * step;
        }
    }
}

/// Arranges all nodes evenly on a circle of radius proportional to node count.
pub fn apply_circular_layout(doc: &mut GraphDocument) {
    let n = doc.nodes.len();
    if n == 0 {
        return;
    }
    if n == 1 {
        doc.nodes[0].pos = Vec2Pos::new(0.0, 0.0);
        return;
    }

    let radius = (n as f32 * 36.0).max(160.0);
    for (i, node) in doc.nodes.iter_mut().enumerate() {
        let angle = (i as f32 / n as f32) * TAU;
        node.pos.x = angle.cos() * radius;
        node.pos.y = angle.sin() * radius;
    }
}

/// Arranges all nodes in a clean, readable grid layout.
pub fn apply_grid_layout(doc: &mut GraphDocument) {
    let n = doc.nodes.len();
    if n == 0 {
        return;
    }
    let cols = (n as f32).sqrt().ceil().max(1.0) as usize;
    let spacing_x = 220.0_f32;
    let spacing_y = 140.0_f32;
    let offset_x = ((cols.saturating_sub(1)) as f32 * spacing_x) * 0.5;
    let rows = n.div_ceil(cols);
    let offset_y = ((rows.saturating_sub(1)) as f32 * spacing_y) * 0.5;

    for (i, node) in doc.nodes.iter_mut().enumerate() {
        let col = i % cols;
        let row = i / cols;
        node.pos.x = (col as f32) * spacing_x - offset_x;
        node.pos.y = (row as f32) * spacing_y - offset_y;
    }
}
