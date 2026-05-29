use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::resources::Juice;
use crate::tuning;

/// Wave 5 -- ticks the active screen flash down on REAL time (so it stays
/// smooth during a hit-stop) and clears it when finished. Runs in `Update`.
pub fn update_screen_flash(real_time: Res<Time<Real>>, mut juice: ResMut<Juice>) {
    if let Some(flash) = juice.flash.as_mut() {
        flash.remaining -= real_time.delta_secs();
        if flash.remaining <= 0.0 {
            juice.flash = None;
        }
    }
}

/// Draws the active screen flash as a full-screen egui overlay (Wave 5).
///
/// - Player-hurt: a red EDGE VIGNETTE (corners darkened red, center clear) so it
///   reads as "you got hit" without blanking the screen.
/// - Explosion / boss-burst: a flat WHITE full-screen flash.
/// - Floor transition: a flat DARK full-screen fade ("descending").
///
/// Rendered in `EguiPrimaryContextPass` (like the HUD) using a transparent,
/// non-interactive `Area` painted at the screen layer so it overlays the game
/// but never eats input. Alpha decays over the flash's lifetime.
pub fn draw_screen_flash(mut contexts: EguiContexts, juice: Res<Juice>) -> Result {
    let Some(flash) = juice.flash else {
        return Ok(());
    };

    let ctx = contexts.ctx_mut()?;
    let screen = ctx.content_rect();

    // Fade fraction: 1.0 at start -> 0.0 at end.
    let frac = (flash.remaining / flash.total.max(0.0001)).clamp(0.0, 1.0);
    let alpha = (flash.peak_alpha * frac).clamp(0.0, 1.0);
    let base = flash.color;
    let to_u8 = |c: f32| (c.clamp(0.0, 1.0) * 255.0) as u8;

    egui::Area::new(egui::Id::new("screen_flash_overlay"))
        .order(egui::Order::Foreground)
        .interactable(false)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let painter = ui.painter();
            if flash.vignette {
                // Edge vignette: four gradient-ish bands around the screen edges,
                // strongest at the border, fading toward the center. Approximated
                // with a few stacked translucent rects (cheap, no shader).
                let bands = 6;
                let thickness = screen.height().min(screen.width()) * 0.16;
                for i in 0..bands {
                    let t = i as f32 / bands as f32; // 0 (outer) .. ~1 (inner)
                    let band_alpha = alpha * (1.0 - t);
                    let col = egui::Color32::from_rgba_unmultiplied(
                        to_u8(base.red),
                        to_u8(base.green),
                        to_u8(base.blue),
                        to_u8(band_alpha),
                    );
                    let inset = thickness * t;
                    let th = thickness / bands as f32 + 1.0;
                    // Top
                    painter.rect_filled(
                        egui::Rect::from_min_max(
                            egui::pos2(screen.min.x, screen.min.y + inset),
                            egui::pos2(screen.max.x, screen.min.y + inset + th),
                        ),
                        0.0,
                        col,
                    );
                    // Bottom
                    painter.rect_filled(
                        egui::Rect::from_min_max(
                            egui::pos2(screen.min.x, screen.max.y - inset - th),
                            egui::pos2(screen.max.x, screen.max.y - inset),
                        ),
                        0.0,
                        col,
                    );
                    // Left
                    painter.rect_filled(
                        egui::Rect::from_min_max(
                            egui::pos2(screen.min.x + inset, screen.min.y),
                            egui::pos2(screen.min.x + inset + th, screen.max.y),
                        ),
                        0.0,
                        col,
                    );
                    // Right
                    painter.rect_filled(
                        egui::Rect::from_min_max(
                            egui::pos2(screen.max.x - inset - th, screen.min.y),
                            egui::pos2(screen.max.x - inset, screen.max.y),
                        ),
                        0.0,
                        col,
                    );
                }
            } else {
                // Flat full-screen fill (white explosion flash / dark floor fade).
                let col = egui::Color32::from_rgba_unmultiplied(
                    to_u8(base.red),
                    to_u8(base.green),
                    to_u8(base.blue),
                    to_u8(alpha),
                );
                painter.rect_filled(screen, 0.0, col);
            }
        });

    Ok(())
}

/// `OnEnter(Playing)` -- start the dark "descending" fade so each new floor
/// reads as a descent. Harmless on the very first floor (the menu transition
/// already covered the screen), and it never blocks gameplay (non-interactive
/// overlay that fades out).
pub fn floor_transition_flash(mut juice: ResMut<Juice>) {
    juice.flash(
        bevy::color::Srgba::new(0.0, 0.0, 0.0, 1.0),
        tuning::FLASH_FLOOR_FADE_ALPHA,
        tuning::FLASH_FLOOR_FADE_DURATION,
        false,
    );
}
