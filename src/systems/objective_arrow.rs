use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::components::*;
use crate::resources::*;
use crate::utils::grid_to_world_center;

/// Draws an on-screen edge arrow pointing toward the exit/objective tile while it
/// is off-screen. Extermination-only maps (no `ObjectiveMarker`) are skipped.
///
/// The arrow is painted with egui in screen space: we project the exit's world
/// position through the camera; if it falls outside the viewport (or behind), we
/// clamp a marker to the screen edge in the direction of the exit and draw a
/// triangle pointing that way.
///
/// B0001: camera and player queries are disjoint via `With<CameraMarker>` /
/// `With<Player>` (mutually exclusive markers).
#[allow(clippy::type_complexity)]
pub fn objective_arrow(
    mut contexts: EguiContexts,
    floor: Res<Floor>,
    objective: Res<ObjectiveMarker>,
    scale_factor: Res<ScaleFactor>,
    camera_query: Query<(&Camera, &GlobalTransform), With<CameraMarker>>,
) -> Result {
    let Some(exit) = objective.0 else {
        return Ok(()); // Extermination map: no arrow.
    };
    if exit.z != floor.0 {
        return Ok(()); // Exit is on another floor.
    }

    let Some((camera, cam_transform)) = camera_query.iter().next() else {
        return Ok(());
    };

    let exit_world = grid_to_world_center(exit.x, exit.y, scale_factor.0);

    let ctx = contexts.ctx_mut()?;
    let screen = ctx.content_rect();
    let center = screen.center();

    // Project the exit into viewport (logical) pixels.
    let projected = camera.world_to_viewport(cam_transform, exit_world.extend(0.0));

    // Determine whether the exit is comfortably on-screen; if so, no arrow.
    if let Ok(p) = projected {
        let margin = 48.0;
        let inside = p.x > screen.min.x + margin
            && p.x < screen.max.x - margin
            && p.y > screen.min.y + margin
            && p.y < screen.max.y - margin;
        if inside {
            return Ok(());
        }
    }

    // Direction from screen center toward the exit, in egui screen space.
    let dir = match projected {
        Ok(p) => {
            let d = egui::vec2(p.x - center.x, p.y - center.y);
            if d.length() < 1.0 {
                egui::vec2(0.0, -1.0)
            } else {
                d.normalized()
            }
        }
        Err(_) => egui::vec2(0.0, -1.0),
    };

    // Clamp the arrow tip to a rectangle inset from the screen edge.
    let inset = 36.0;
    let half = egui::vec2(
        (screen.width() / 2.0 - inset).max(1.0),
        (screen.height() / 2.0 - inset).max(1.0),
    );
    // Scale dir so it touches the inset box (whichever axis hits first).
    let tx = if dir.x.abs() > 1e-3 {
        half.x / dir.x.abs()
    } else {
        f32::INFINITY
    };
    let ty = if dir.y.abs() > 1e-3 {
        half.y / dir.y.abs()
    } else {
        f32::INFINITY
    };
    let t = tx.min(ty);
    let tip = center + dir * t;

    // Draw a triangular arrow pointing along `dir`, using a foreground painter so
    // it sits above the world (but below modal panels).
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("objective_arrow"),
    ));
    let size = 18.0;
    let perp = egui::vec2(-dir.y, dir.x);
    let p0 = tip + dir * size; // point
    let p1 = tip - dir * size * 0.5 + perp * size * 0.6;
    let p2 = tip - dir * size * 0.5 - perp * size * 0.6;
    let color = egui::Color32::from_rgb(255, 215, 50);
    painter.add(egui::Shape::convex_polygon(
        vec![p0, p1, p2],
        color,
        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(120, 90, 0)),
    ));
    painter.text(
        tip - dir * (size + 10.0),
        egui::Align2::CENTER_CENTER,
        "EXIT",
        egui::FontId::proportional(13.0),
        color,
    );

    Ok(())
}
