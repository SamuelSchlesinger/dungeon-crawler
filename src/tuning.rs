//! Central place for all real-time action tuning constants.
//!
//! Wave 1 turned this grid crawler into a real-time action roguelike. All of the
//! "feel" knobs (movement speed, attack range/cooldown, dash timing, enemy stats)
//! live here so they are easy to find and tweak during playtest iteration.
//!
//! World units: one tile == `ScaleFactor` world units (currently 50). Speeds are
//! expressed in TILES per second and multiplied by the scale factor at use sites,
//! so they stay meaningful even if the tile size changes.

use std::f32::consts::PI;

// ----------------------------------------------------------------------------
// Player movement
// ----------------------------------------------------------------------------

/// Player movement speed, in tiles per second.
pub const PLAYER_SPEED_TILES: f32 = 5.0;

/// Radius of the player's collision circle, as a fraction of a tile. Used for
/// wall collision so the player can't clip into solid tiles.
pub const PLAYER_RADIUS_TILES: f32 = 0.35;

// ----------------------------------------------------------------------------
// Camera
// ----------------------------------------------------------------------------

/// How quickly the camera lerps toward the player each frame (higher == snappier).
/// Applied as `1 - exp(-CAMERA_LERP * dt)` so it is framerate independent.
pub const CAMERA_LERP: f32 = 8.0;

// ----------------------------------------------------------------------------
// Melee attack
// ----------------------------------------------------------------------------

/// Reach of the melee swing, in tiles, measured from the player center.
pub const ATTACK_RANGE_TILES: f32 = 1.6;

/// Half-angle of the melee cone, in radians (~60 degrees => 120 degree arc).
pub const ATTACK_HALF_ANGLE: f32 = PI / 3.0;

/// Seconds between melee swings.
pub const ATTACK_COOLDOWN: f32 = 0.35;

/// How long the swing visual stays on screen, in seconds.
pub const SWING_VISUAL_LIFETIME: f32 = 0.12;

/// Knockback impulse applied to enemies hit by the player, in tiles/sec.
pub const ATTACK_KNOCKBACK_TILES: f32 = 9.0;

// ----------------------------------------------------------------------------
// Dash / dodge
// ----------------------------------------------------------------------------

/// Dash burst speed, in tiles per second (added on top of normal movement while
/// the dash is active).
pub const DASH_SPEED_TILES: f32 = 18.0;

/// How long the dash burst lasts, in seconds.
pub const DASH_DURATION: f32 = 0.15;

/// Invulnerability window granted by a dash, in seconds.
pub const DASH_IFRAMES: f32 = 0.2;

/// Cooldown before the player can dash again, in seconds.
pub const DASH_COOLDOWN: f32 = 0.8;

// ----------------------------------------------------------------------------
// Hit feedback
// ----------------------------------------------------------------------------

/// How long a damaged actor flashes red, in seconds.
pub const HIT_FLASH_DURATION: f32 = 0.15;

/// How quickly residual knockback velocity decays (per second exponential).
pub const KNOCKBACK_DECAY: f32 = 12.0;

// ----------------------------------------------------------------------------
// Enemy AI (real-time)
// ----------------------------------------------------------------------------

/// How often (seconds) an enemy recomputes its A* path to the player.
pub const ENEMY_REPATH_INTERVAL: f32 = 0.6;

/// Distance (in tiles) at which an enemy begins its attack telegraph.
pub const ENEMY_ATTACK_RANGE_TILES: f32 = 1.1;

/// Telegraph (windup) duration before an enemy strikes, in seconds.
pub const ENEMY_TELEGRAPH: f32 = 0.3;

/// Cooldown between enemy attacks, in seconds.
pub const ENEMY_ATTACK_COOLDOWN: f32 = 1.0;

/// Knockback applied to the player when an enemy hits, in tiles/sec.
pub const ENEMY_KNOCKBACK_TILES: f32 = 6.0;

/// Per-enemy-type real-time movement speed, in tiles per second.
pub fn enemy_speed_tiles(enemy_type: crate::components::EnemyType) -> f32 {
    use crate::components::EnemyType::*;
    match enemy_type {
        Skeleton => 4.2, // fast, fragile
        Orc => 3.0,      // medium
        Ghost => 2.2,    // slow, tanky / keeps distance
    }
}
