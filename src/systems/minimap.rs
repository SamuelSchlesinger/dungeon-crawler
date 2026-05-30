use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::components::*;
use crate::resources::*;
use crate::tuning;

/// Small top-right minimap overlay drawn with egui.
///
/// Shows the explored tiles of the current floor as cells (brighter where
/// currently in line of sight), the player marker, the exit/objective tile, and
/// dots for enemies that are currently in line of sight. The map auto-fits the
/// explored bounds and scales cell size down to stay within `MINIMAP_MAX_SIZE`.
///
/// B0001 / borrow safety: this system only reads resources and immutable
/// component queries; nothing else mutates these in the egui pass.
#[allow(clippy::type_complexity)]
pub fn minimap(
    mut contexts: EguiContexts,
    floor: Res<Floor>,
    revealed: Res<Revealed>,
    visible_tiles: Res<VisibleTiles>,
    objective: Res<ObjectiveMarker>,
    player_query: Query<&Position, With<Player>>,
    enemy_query: Query<&Position, (With<Enemy>, Without<Player>)>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    let z = floor.0;
    // Explored tiles on the current floor only.
    let explored: Vec<Position> = revealed
        .0
        .iter()
        .filter(|p| p.z == z)
        .copied()
        .collect();
    if explored.is_empty() {
        return Ok(());
    }

    // Bounds of the explored region (grid coords).
    let (mut min_x, mut min_y, mut max_x, mut max_y) =
        (i64::MAX, i64::MAX, i64::MIN, i64::MIN);
    for p in &explored {
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }
    let cols = (max_x - min_x + 1) as f32;
    let rows = (max_y - min_y + 1) as f32;

    // Cell size: tuned default, scaled down so the whole map fits the max box.
    let cell = tuning::MINIMAP_CELL_SIZE
        .min(tuning::MINIMAP_MAX_SIZE / cols.max(1.0))
        .min(tuning::MINIMAP_MAX_SIZE / rows.max(1.0))
        .max(1.0);
    let width = cols * cell;
    let height = rows * cell;

    let player_pos = player_query.iter().next().copied();

    egui::Window::new("Map")
        .title_bar(false)
        .resizable(false)
        .interactable(false)
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-12.0, 12.0))
        .frame(
            egui::Frame::NONE
                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 170))
                .inner_margin(egui::Margin::same(6))
                .corner_radius(4.0),
        )
        .show(ctx, |ui| {
            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
            let painter = ui.painter_at(rect);

            // egui screen Y grows downward; the world grid Y grows upward, so we
            // flip Y when mapping grid -> minimap pixels.
            let to_screen = |gx: i64, gy: i64| -> egui::Pos2 {
                let fx = (gx - min_x) as f32;
                let fy = (max_y - gy) as f32;
                egui::pos2(rect.min.x + fx * cell, rect.min.y + fy * cell)
            };
            let cell_rect = |gx: i64, gy: i64| -> egui::Rect {
                let tl = to_screen(gx, gy);
                egui::Rect::from_min_size(tl, egui::vec2(cell, cell))
            };

            // Explored tiles: brighter where currently in sight.
            for p in &explored {
                let in_los = visible_tiles.0.contains(p);
                let color = if in_los {
                    egui::Color32::from_rgb(150, 165, 185)
                } else {
                    egui::Color32::from_rgb(70, 78, 92)
                };
                painter.rect_filled(cell_rect(p.x, p.y), 0.0, color);
            }

            // Exit / objective tile -- only once the player has actually seen it
            // (gated on fog, so the minimap doesn't reveal the exit from frame 1).
            if let Some(exit) = objective.0 {
                if exit.z == z && revealed.0.contains(&exit) {
                    painter.rect_filled(
                        cell_rect(exit.x, exit.y),
                        0.0,
                        egui::Color32::from_rgb(255, 215, 50),
                    );
                }
            }

            // In-sight enemies as red dots.
            for ep in enemy_query.iter() {
                if ep.z == z && visible_tiles.0.contains(ep) {
                    let c = cell_rect(ep.x, ep.y).center();
                    painter.circle_filled(
                        c,
                        (cell * 0.45).max(1.5),
                        egui::Color32::from_rgb(230, 60, 60),
                    );
                }
            }

            // Player marker (bright green dot).
            if let Some(p) = player_pos {
                if p.z == z {
                    let c = cell_rect(p.x, p.y).center();
                    painter.circle_filled(
                        c,
                        (cell * 0.5).max(2.0),
                        egui::Color32::from_rgb(60, 230, 90),
                    );
                }
            }
        });

    Ok(())
}
