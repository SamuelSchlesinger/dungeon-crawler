use bevy::prelude::*;

pub use positioning::Position;

use std::{
    collections::{BTreeSet, VecDeque},
};

#[derive(Component, Debug)]
pub struct HealthBar(pub Entity);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Strength(pub i64);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Health(pub i64);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct OriginalHealth(pub i64);

#[derive(Component, Debug)]
pub struct WakeZone(pub BTreeSet<Position>);

#[derive(Component, Debug)]
pub struct Awake(pub bool);

#[derive(Component, Debug, Clone, Copy)]
pub enum AIBehavior {
    Aggressive,   // Always chase player
    Defensive,    // Retreat when health < 30%
    Patrol,       // Random movement, chase when close
}

impl AIBehavior {
    pub fn for_enemy_type(enemy_type: EnemyType) -> Self {
        match enemy_type {
            EnemyType::Skeleton => AIBehavior::Aggressive,
            EnemyType::Orc => AIBehavior::Patrol,
            EnemyType::Ghost => AIBehavior::Defensive,
        }
    }
}

#[derive(Component, Debug)]
pub struct ZLevel(pub f32);

#[derive(Component, Debug)]
pub struct SpriteIndex(pub usize);

#[derive(Component, Debug)]
pub struct Tile;

/// The full-brightness color a tile should render at when in line of sight. The
/// fog system multiplies this down for explored-but-unseen ("dimmed") tiles, so
/// it needs the un-dimmed base to restore from. Set once at spawn.
#[derive(Component, Debug, Clone, Copy)]
pub struct TileBaseColor(pub Color);

/// Marks the victory / exit tile so it can be tinted distinctly and located for
/// the off-screen objective arrow. Only present on `VictoryCondition::Arrival`
/// target tiles.
#[derive(Component, Debug)]
pub struct ExitMarker;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyType {
    Skeleton,  // Fast, weak (sprite: 2700)
    Orc,       // Balanced (sprite: 2701)
    Ghost,     // Slow, strong (sprite: 2702)
}

impl EnemyType {
    pub fn get_stats(&self, floor: i64) -> (i64, i64) {
        // Returns (health, strength) scaled by floor
        let floor_multiplier = 1.0 + (floor as f32 * 0.15);  // 15% increase per floor
        let base_stats = match self {
            EnemyType::Skeleton => (3, 1),   // Fast but fragile
            EnemyType::Orc => (7, 2),        // Balanced
            EnemyType::Ghost => (10, 3),     // Strong and tanky
        };
        (
            ((base_stats.0 as f32) * floor_multiplier) as i64,
            ((base_stats.1 as f32) * floor_multiplier) as i64,
        )
    }

    pub fn sprite_index(&self) -> usize {
        match self {
            EnemyType::Skeleton => 2700,
            EnemyType::Orc => 2701,
            EnemyType::Ghost => 2702,
        }
    }

    pub fn random() -> Self {
        let r: f32 = rand::random();
        if r < 0.4 {
            EnemyType::Skeleton
        } else if r < 0.7 {
            EnemyType::Orc
        } else {
            EnemyType::Ghost
        }
    }
}

#[derive(Component, Debug)]
pub struct Enemy;

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug)]
pub struct CameraMarker;

/// Per-entity passability flag. Retained on actors/pickups as a data-model hint;
/// real-time collision now reads tile passability from the `Tiles` resource, so
/// the flag itself is not currently queried.
#[derive(Component, Debug)]
pub struct Passable(#[allow(dead_code)] pub bool);

#[test]
fn test_adjacency() {
    let position = Position { x: 5, y: 5, z: 0 };
    let other = Position { x: 4, y: 5, z: 0 };
    assert!(position.is_adjacent_to(other));
}

#[derive(Component, Debug)]
pub struct MovementPath {
    pub path: Option<VecDeque<Position>>,
}

#[derive(Component)]
pub struct HealthGain;

/// Marker for a dropped weapon pickup lying on the floor.
#[derive(Component)]
pub struct WeaponDrop;

/// Stats granted by a weapon pickup. Names come from a small fixed table.
#[derive(Component, Debug, Clone)]
pub struct WeaponStats {
    pub strength_bonus: i64,
    pub name: &'static str,
}

/// Fixed table of weapons that can drop from enemies. The sprite index reuses an
/// existing tilesheet icon (a sword-like glyph) so no new assets are required.
pub const WEAPON_TABLE: &[(&str, i64)] = &[
    ("Rusty Dagger", 1),
    ("Iron Sword", 2),
    ("Steel Mace", 3),
    ("War Axe", 4),
    ("Enchanted Blade", 6),
];

impl WeaponStats {
    /// Pick a random weapon from the fixed table.
    pub fn random() -> Self {
        let (name, strength_bonus) = WEAPON_TABLE[rand::random_range(0..WEAPON_TABLE.len())];
        WeaponStats {
            strength_bonus,
            name,
        }
    }
}

/// Sprite index used to render weapon drops on the floor. Reuses an existing
/// tilesheet glyph (same row family as the player sprite).
pub const WEAPON_SPRITE_INDEX: usize = 24 * 64 + 41;

// ---------------------------------------------------------------------------
// Real-time action components (Wave 1)
// ---------------------------------------------------------------------------

/// Continuous world-space position (in world units). Actors that move in real
/// time (player + enemies) carry this; their `Transform` is driven from it each
/// frame and their grid `Position` is synced by rounding. Entities WITH this
/// component are skipped by `animate_sprites` (which only grid-snaps statics).
#[derive(Component, Debug, Clone, Copy)]
pub struct WorldPos(pub Vec2);

/// Residual knockback velocity (world units/sec) that decays over time. Applied
/// on top of intentional movement so hits visibly shove actors around.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Knockback(pub Vec2);

/// Per-actor facing direction (unit vector). Used to orient swing visuals and
/// as a dash fallback when the player is standing still.
#[derive(Component, Debug, Clone, Copy)]
pub struct Facing(pub Vec2);

impl Default for Facing {
    fn default() -> Self {
        Facing(Vec2::new(1.0, 0.0))
    }
}

/// Player melee attack cooldown. `Timer` finished == ready to swing.
#[derive(Component, Debug)]
pub struct AttackCooldown(pub Timer);

/// Player dash/dodge state machine.
#[derive(Component, Debug)]
pub struct Dash {
    /// Counts down while the dash burst is active.
    pub active: Timer,
    /// Counts down while the player is invulnerable (i-frames).
    pub iframes: Timer,
    /// Counts down until the player may dash again.
    pub cooldown: Timer,
    /// Direction of the current dash burst (unit vector).
    pub dir: Vec2,
    pub dashing: bool,
}

/// Transient red tint applied to an actor that just took damage. Removed when the
/// timer finishes (which restores the sprite color to `ActorBaseColor`).
#[derive(Component, Debug)]
pub struct HitFlash(pub Timer);

/// The resting sprite color of an actor (player white, enemies per-type tint).
/// Used by `hit_flash` to restore the correct color after a hit flash instead of
/// clobbering per-type enemy tints back to plain white.
#[derive(Component, Debug, Clone, Copy)]
pub struct ActorBaseColor(pub Color);

/// Real-time enemy attack state machine: idle -> telegraph (windup) -> strike,
/// then cooldown back to idle.
#[derive(Component, Debug)]
pub struct EnemyAttack {
    pub telegraph: Timer,
    pub cooldown: Timer,
    pub winding_up: bool,
}

/// Periodic A* re-path timer for real-time enemy movement. When finished the
/// enemy recomputes its grid path to the player and the timer resets.
#[derive(Component, Debug)]
pub struct RepathTimer(pub Timer);

/// Marks a short-lived purely-visual effect (swing arc, telegraph flash). Carries
/// its own lifetime timer and is despawned when it expires or on floor teardown.
#[derive(Component, Debug)]
pub struct TransientVisual(pub Timer);

/// A short-lived floating damage number (Text2d in world space) that rises and
/// fades, then despawns. Spawned wherever damage is applied.
#[derive(Component, Debug)]
pub struct DamageNumber {
    pub timer: Timer,
    /// Base RGB color (alpha is animated by the fade).
    pub color: Color,
}

#[derive(Component)]
pub struct Particle {
    pub lifetime: f32,
    pub velocity: Vec2,
}

#[derive(Component, Clone, Copy)]
pub enum ParticleType {
    HitSpark,
    Death,
    HealthPickup,
}
