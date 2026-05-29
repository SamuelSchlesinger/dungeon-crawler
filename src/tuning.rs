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

// ----------------------------------------------------------------------------
// Fog of war / line-of-sight (Wave 2)
// ----------------------------------------------------------------------------

/// How far the player can see, in tiles (Euclidean radius). Tiles within this
/// radius AND not blocked by an opaque wall are in current line of sight.
/// Smaller values make the fog tighter / more claustrophobic.
pub const FOG_RADIUS: i64 = 6;

/// Tint multiplier applied to tiles that have been explored but are NOT currently
/// in line of sight (the "remembered but dark" tier). 1.0 == full brightness.
pub const FOG_DIM_FACTOR: f32 = 0.45;

// ----------------------------------------------------------------------------
// Minimap (Wave 2)
// ----------------------------------------------------------------------------

/// Pixel size of a single tile cell drawn on the egui minimap overlay. Larger
/// values make the minimap bigger (it auto-fits the explored bounds).
pub const MINIMAP_CELL_SIZE: f32 = 3.0;

/// Maximum on-screen width/height of the minimap panel, in pixels. The minimap
/// scales its cell size down to stay within this box on large floors.
pub const MINIMAP_MAX_SIZE: f32 = 200.0;

// ----------------------------------------------------------------------------
// Floating damage numbers (Wave 2)
// ----------------------------------------------------------------------------

/// How long a floating damage number stays on screen before despawning, seconds.
pub const DAMAGE_NUMBER_LIFETIME: f32 = 0.7;

/// How fast a floating damage number rises, in tiles per second.
pub const DAMAGE_NUMBER_RISE_TILES: f32 = 1.5;

// ----------------------------------------------------------------------------
// Enemy visuals (Wave 2)
// ----------------------------------------------------------------------------

/// Per-enemy-type sprite tint and scale so the three types read at a glance.
/// Returns `(color, scale_multiplier)`. Scale multiplies the base actor sprite
/// size set at spawn.
pub fn enemy_visual(enemy_type: crate::components::EnemyType) -> (bevy::prelude::Color, f32) {
    use bevy::prelude::Color;
    use crate::components::EnemyType::*;
    match enemy_type {
        // Pale bone-white, small.
        Skeleton => (Color::srgb(0.90, 0.90, 0.80), 0.78),
        // Sickly green, large and bulky.
        Orc => (Color::srgb(0.45, 0.85, 0.35), 1.18),
        // Translucent ice-blue, medium.
        Ghost => (Color::srgba(0.55, 0.75, 1.0, 0.62), 0.95),
    }
}

// ----------------------------------------------------------------------------
// Gold / economy (Wave 3)
// ----------------------------------------------------------------------------

/// Base gold awarded per enemy kill (before per-type and floor scaling).
pub const GOLD_PER_KILL_BASE: i64 = 5;

/// Extra gold per cleared floor added to each kill reward (depth bonus).
pub const GOLD_PER_KILL_FLOOR_BONUS: i64 = 2;

/// Base cost (floor 1) to reroll the three offered boons. Scales with floor.
pub const SHOP_REROLL_BASE_COST: i64 = 15;

/// Added to the reroll cost per cleared floor.
pub const SHOP_REROLL_FLOOR_COST: i64 = 5;

/// Base cost (floor 1) to fully heal the player on the boon screen. Scales.
pub const SHOP_HEAL_BASE_COST: i64 = 25;

/// Added to the heal cost per cleared floor.
pub const SHOP_HEAL_FLOOR_COST: i64 = 10;

/// Reroll cost for a given run depth (floors cleared).
pub fn reroll_cost(floor: i64) -> i64 {
    SHOP_REROLL_BASE_COST + SHOP_REROLL_FLOOR_COST * floor.max(0)
}

/// Full-heal cost for a given run depth (floors cleared).
pub fn heal_cost(floor: i64) -> i64 {
    SHOP_HEAL_BASE_COST + SHOP_HEAL_FLOOR_COST * floor.max(0)
}

// ----------------------------------------------------------------------------
// Boon magnitudes (Wave 3) -- see systems::boons for the pool definition.
// ----------------------------------------------------------------------------

/// +damage boon: multiplier added to the damage multiplier (0.25 == +25%).
pub const BOON_DAMAGE: f32 = 0.25;
/// +max HP boon: flat HP added (and healed) when picked.
pub const BOON_MAX_HP: i64 = 20;
/// Attack-cooldown boon: fraction the cooldown is reduced by (0.20 == -20%).
pub const BOON_ATTACK_COOLDOWN: f32 = 0.20;
/// +move-speed boon: multiplier added to the move-speed multiplier.
pub const BOON_MOVE_SPEED: f32 = 0.20;
/// +1 projectile boon: extra projectiles added per ranged shot.
pub const BOON_PROJECTILE: i64 = 1;
/// Lifesteal boon: fraction of damage dealt healed back.
pub const BOON_LIFESTEAL: f32 = 0.10;
/// Dash-cooldown boon: fraction the dash cooldown is reduced by.
pub const BOON_DASH_COOLDOWN: f32 = 0.35;
/// +attack-range boon: multiplier added to the attack-range multiplier.
pub const BOON_ATTACK_RANGE: f32 = 0.40;
/// +knockback boon: multiplier added to the knockback multiplier.
pub const BOON_KNOCKBACK: f32 = 0.50;
/// Crit-chance boon: chance added (0.15 == +15%). A crit deals double damage.
pub const BOON_CRIT: f32 = 0.15;
/// Thorns boon: fraction of incoming damage reflected back to the attacker.
pub const BOON_THORNS: f32 = 0.50;
/// +attack-arc boon: radians added to the melee cone half-angle.
pub const BOON_ATTACK_ARC: f32 = PI / 9.0; // +20 degrees

/// Crit damage multiplier applied when an attack rolls a critical hit.
pub const CRIT_MULTIPLIER: f32 = 2.0;

// ----------------------------------------------------------------------------
// Weapon types / projectiles (Wave 3)
// ----------------------------------------------------------------------------

/// Speed of a ranged projectile, in tiles per second.
pub const PROJECTILE_SPEED_TILES: f32 = 14.0;

/// Maximum distance a projectile travels (tiles) before despawning.
pub const PROJECTILE_RANGE_TILES: f32 = 9.0;

/// Collision radius of a projectile vs. an enemy, in tiles.
pub const PROJECTILE_HIT_RADIUS_TILES: f32 = 0.5;

/// Angular spread (radians) between extra projectiles when the projectile-count
/// boon fires more than one shot.
pub const PROJECTILE_SPREAD: f32 = PI / 12.0; // 15 degrees

/// Radius of the AoE weapon's radial burst, in tiles.
pub const AOE_RADIUS_TILES: f32 = 2.8;

/// Cooldown of the AoE weapon, in seconds (longer than melee/ranged).
pub const AOE_COOLDOWN: f32 = 0.9;

/// Cooldown of the ranged weapon, in seconds.
pub const RANGED_COOLDOWN: f32 = 0.45;

/// Knockback applied by the AoE burst, in tiles/sec.
pub const AOE_KNOCKBACK_TILES: f32 = 12.0;
