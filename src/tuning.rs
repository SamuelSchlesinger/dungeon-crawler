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
        Archer => 2.8,   // medium, prefers to keep distance
        Charger => 2.6,  // slow walk, but dashes in a burst (see CHARGER_*)
        Bomber => 4.6,   // fast rusher
        Boss => 2.0,     // slow, deliberate
    }
}

// ----------------------------------------------------------------------------
// Wave 4 -- Archer (ranged enemy)
// ----------------------------------------------------------------------------

/// Distance (tiles) the archer tries to maintain from the player. Closer than
/// `ARCHER_RETREAT_RANGE` it retreats; within `ARCHER_FIRE_RANGE` it shoots.
pub const ARCHER_PREFERRED_RANGE: f32 = 5.0;
/// If the player gets this close (tiles), the archer actively backs away.
pub const ARCHER_RETREAT_RANGE: f32 = 3.0;
/// Maximum range (tiles) at which an archer will fire at the player.
pub const ARCHER_FIRE_RANGE: f32 = 7.5;
/// Seconds between archer shots.
pub const ARCHER_SHOOT_COOLDOWN: f32 = 1.6;
/// Speed of an archer's arrow, in tiles per second.
pub const ARCHER_PROJECTILE_SPEED_TILES: f32 = 9.0;
/// Maximum distance (tiles) an archer's arrow travels before fizzling.
pub const ARCHER_PROJECTILE_RANGE_TILES: f32 = 9.0;

// ----------------------------------------------------------------------------
// Wave 4 -- Charger (bruiser that telegraphs then dashes)
// ----------------------------------------------------------------------------

/// Distance (tiles) within which the charger begins its wind-up telegraph.
pub const CHARGER_TRIGGER_RANGE: f32 = 6.0;
/// Wind-up (telegraph) duration before the charger launches, in seconds.
pub const CHARGER_WINDUP: f32 = 0.5;
/// Charge dash speed, in tiles per second (fast straight-line lunge).
pub const CHARGER_DASH_SPEED_TILES: f32 = 16.0;
/// How long the charge lunge lasts, in seconds.
pub const CHARGER_DASH_DURATION: f32 = 0.45;
/// Recovery (stunned) time after a charge, in seconds.
pub const CHARGER_RECOVERY: f32 = 0.8;
/// Contact radius (tiles) for the charger's slam to hit the player mid-charge.
pub const CHARGER_HIT_RADIUS_TILES: f32 = 0.9;
/// Bonus damage multiplier applied to the charger's contact hit (vs its
/// base `Strength`), reflecting the high-impact slam.
pub const CHARGER_DAMAGE_MULT: f32 = 2.0;
/// Knockback dealt to the player by a charger slam, in tiles/sec.
pub const CHARGER_KNOCKBACK_TILES: f32 = 12.0;

// ----------------------------------------------------------------------------
// Wave 4 -- Bomber (exploder)
// ----------------------------------------------------------------------------

/// Distance (tiles) at which the bomber starts its fuse (visible flash/scale).
pub const BOMBER_FUSE_RANGE: f32 = 1.4;
/// Fuse (telegraph) duration before the bomber detonates, in seconds.
pub const BOMBER_FUSE: f32 = 0.55;
/// Explosion radius of a bomber, in tiles. Both the player and other enemies in
/// this radius take damage + knockback.
pub const BOMBER_EXPLOSION_RADIUS_TILES: f32 = 2.6;
/// Explosion damage multiplier vs the bomber's base `Strength`.
pub const BOMBER_DAMAGE_MULT: f32 = 3.0;
/// Knockback dealt by a bomber explosion, in tiles/sec.
pub const BOMBER_KNOCKBACK_TILES: f32 = 14.0;

// ----------------------------------------------------------------------------
// Wave 4 -- Boss (every BOSS_FLOOR_INTERVAL floors)
// ----------------------------------------------------------------------------

/// A boss arena is generated when the floor being entered (1-indexed, i.e.
/// `floors_completed + 1`) is a multiple of this.
pub const BOSS_FLOOR_INTERVAL: i64 = 5;
/// Boss base HP, before floor scaling (multiplied like normal enemies).
pub const BOSS_BASE_HP: i64 = 120;
/// Boss base strength, before floor scaling.
pub const BOSS_BASE_STRENGTH: i64 = 6;
/// Sprite scale multiplier for the boss (large, imposing).
pub const BOSS_SCALE: f32 = 2.4;
/// Boss tint (deep menacing red/purple).
pub const fn boss_tint() -> bevy::prelude::Color {
    bevy::prelude::Color::srgb(0.85, 0.20, 0.25)
}
/// Seconds between boss projectile bursts.
pub const BOSS_BURST_COOLDOWN: f32 = 2.4;
/// Number of projectiles in a boss radial burst.
pub const BOSS_BURST_COUNT: usize = 10;
/// Speed of a boss burst projectile, in tiles per second.
pub const BOSS_PROJECTILE_SPEED_TILES: f32 = 7.0;
/// Range (tiles) of a boss burst projectile before it fizzles.
pub const BOSS_PROJECTILE_RANGE_TILES: f32 = 12.0;
/// Seconds between boss charge attacks.
pub const BOSS_CHARGE_COOLDOWN: f32 = 5.0;
/// Boss charge wind-up, in seconds.
pub const BOSS_CHARGE_WINDUP: f32 = 0.6;
/// Boss charge dash speed, tiles/sec.
pub const BOSS_CHARGE_SPEED_TILES: f32 = 14.0;
/// Boss charge dash duration, seconds.
pub const BOSS_CHARGE_DURATION: f32 = 0.6;
/// Seconds between boss "summon adds" casts.
pub const BOSS_SUMMON_COOLDOWN: f32 = 8.0;
/// Number of weak adds spawned per summon.
pub const BOSS_SUMMON_COUNT: usize = 2;

// ----------------------------------------------------------------------------
// Wave 4 -- Room hazards (spikes / lava)
// ----------------------------------------------------------------------------

/// Damage dealt by a hazard tile each time its tick fires.
pub const HAZARD_DAMAGE: i64 = 4;
/// Seconds between hazard damage ticks for an actor standing on it.
pub const HAZARD_TICK_INTERVAL: f32 = 0.6;
/// Number of hazard patches scattered across a procedural floor.
pub const HAZARD_PATCH_COUNT: usize = 4;
/// Maximum number of tiles per hazard patch.
pub const HAZARD_PATCH_MAX_TILES: usize = 4;
/// Hazard tile tint (glowing lava/spike red-orange).
pub const fn hazard_tint() -> bevy::prelude::Color {
    bevy::prelude::Color::srgb(1.0, 0.45, 0.1)
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
        // Wave 4: amber/yellow ranged skirmisher, lean.
        Archer => (Color::srgb(0.95, 0.80, 0.30), 0.85),
        // Wave 4: hulking orange-red bruiser, large.
        Charger => (Color::srgb(0.95, 0.45, 0.25), 1.30),
        // Wave 4: pulsing magenta exploder, compact.
        Bomber => (Color::srgb(0.95, 0.30, 0.85), 0.92),
        // Wave 4: imposing crimson boss, huge.
        Boss => (boss_tint(), BOSS_SCALE),
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
