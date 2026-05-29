use bevy::prelude::*;

use crate::resources::Juice;
use crate::tuning;

/// Wave 5 -- hit-stop / freeze-frames.
///
/// When `Juice.hitstop_remaining > 0` (set by impactful events: kills, big/boss
/// hits, explosions), the global `Time::<Virtual>` relative speed is dropped to
/// `HITSTOP_SLOW_FACTOR` for a few frames, giving hits a satisfying "punch". The
/// remaining time is counted down on REAL time (`Time::<Real>`), NOT virtual
/// time -- otherwise the hit-stop would slow its own countdown and never end.
///
/// Crucially this only touches `Time::<Virtual>`, which drives gameplay systems
/// (`Res<Time>` resolves to virtual time). The egui UI (menu / boon / end / HUD)
/// runs in `EguiPrimaryContextPass` on real time, so menus and the boon screen
/// stay fully responsive during a hit-stop.
///
/// Runs every frame in `Update` (no state gate) so virtual time is reliably
/// restored to 1.0 the moment a hit-stop expires, including across a state
/// change that happens mid-stop.
pub fn hit_stop(
    real_time: Res<Time<Real>>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut juice: ResMut<Juice>,
) {
    if juice.hitstop_remaining > 0.0 {
        juice.hitstop_remaining -= real_time.delta_secs();
        if juice.hitstop_remaining > 0.0 {
            virtual_time.set_relative_speed(tuning::HITSTOP_SLOW_FACTOR.max(0.0));
            return;
        }
        juice.hitstop_remaining = 0.0;
    }
    // Restore normal speed (idempotent; cheap to set each frame).
    if virtual_time.relative_speed() != 1.0 {
        virtual_time.set_relative_speed(1.0);
    }
}
