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

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AIBehavior {
    Aggressive,   // Always chase player
    Defensive,    // Retreat when health < 30%
    Patrol,       // Random movement, chase when close
    /// Wave 4: keeps `ARCHER_PREFERRED_RANGE` from the player, firing arrows and
    /// retreating when the player closes in. Movement handled in `enemy_move`,
    /// shooting in `archer_shoot`.
    Kiting,
    /// Wave 4: walks toward the player, then telegraphs + dashes in a straight
    /// line (see `charger_ai`). Normal `enemy_move` chase steering is suppressed
    /// while a charge state machine is active.
    Charging,
    /// Wave 4: rushes the player and explodes on contact or death (`bomber_ai`).
    Exploding,
    /// Wave 4: the boss runs its own combined attack pattern (`boss_ai`); its
    /// movement is a slow chase via the normal mover.
    BossPattern,
}

impl AIBehavior {
    pub fn for_enemy_type(enemy_type: EnemyType) -> Self {
        match enemy_type {
            EnemyType::Skeleton => AIBehavior::Aggressive,
            EnemyType::Orc => AIBehavior::Patrol,
            EnemyType::Ghost => AIBehavior::Defensive,
            EnemyType::Archer => AIBehavior::Kiting,
            EnemyType::Charger => AIBehavior::Charging,
            EnemyType::Bomber => AIBehavior::Exploding,
            EnemyType::Boss => AIBehavior::BossPattern,
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
    /// Wave 4: ranged kiter that fires enemy projectiles and keeps its distance.
    Archer,
    /// Wave 4: bruiser that telegraphs then dashes in a straight line.
    Charger,
    /// Wave 4: exploder that rushes and detonates (AoE + knockback).
    Bomber,
    /// Wave 4: the floor boss (one per boss floor) with a combined attack pattern.
    Boss,
}

impl EnemyType {
    pub fn get_stats(&self, floor: i64) -> (i64, i64) {
        // Returns (health, strength) scaled by floor
        let floor_multiplier = 1.0 + (floor as f32 * 0.15);  // 15% increase per floor
        let base_stats = match self {
            EnemyType::Skeleton => (3, 1),   // Fast but fragile
            EnemyType::Orc => (7, 2),        // Balanced
            EnemyType::Ghost => (10, 3),     // Strong and tanky
            EnemyType::Archer => (5, 2),     // Fragile-ish ranged attacker
            EnemyType::Charger => (12, 3),   // Tanky bruiser
            EnemyType::Bomber => (4, 2),     // Fragile; the threat is the boom
            EnemyType::Boss => (
                crate::tuning::BOSS_BASE_HP,
                crate::tuning::BOSS_BASE_STRENGTH,
            ),
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
            // Reuse existing tilesheet glyphs (no new assets); distinct tints +
            // scales (see tuning::enemy_visual) make the types read at a glance.
            EnemyType::Archer => 2703,
            EnemyType::Charger => 2704,
            EnemyType::Bomber => 2705,
            EnemyType::Boss => 2706,
        }
    }

    /// Random NON-boss enemy type, weighted for procedural spawns. Bosses are
    /// never produced here -- they are placed explicitly on boss floors.
    pub fn random() -> Self {
        let r: f32 = rand::random();
        // Weights: Skeleton .28, Orc .20, Ghost .14, Archer .16, Charger .12,
        // Bomber .10 (sum 1.0).
        if r < 0.28 {
            EnemyType::Skeleton
        } else if r < 0.48 {
            EnemyType::Orc
        } else if r < 0.62 {
            EnemyType::Ghost
        } else if r < 0.78 {
            EnemyType::Archer
        } else if r < 0.90 {
            EnemyType::Charger
        } else {
            EnemyType::Bomber
        }
    }

    /// Recovers an `EnemyType` from a stored sprite index, if it is one of the
    /// known enemy-type sprites. Hand-authored maps (unbeatable/avoidance) store
    /// arbitrary sprites that aren't enemy types, so `setup_play` falls back to a
    /// random type for those; procedural floors store real type sprites so the
    /// intended type (including the Boss) round-trips exactly.
    pub fn from_sprite_index(index: usize) -> Option<Self> {
        match index {
            2700 => Some(EnemyType::Skeleton),
            2701 => Some(EnemyType::Orc),
            2702 => Some(EnemyType::Ghost),
            2703 => Some(EnemyType::Archer),
            2704 => Some(EnemyType::Charger),
            2705 => Some(EnemyType::Bomber),
            2706 => Some(EnemyType::Boss),
            _ => None,
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

/// How a weapon changes the player's attack behavior (Wave 3 build variety).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponType {
    /// Aimed melee cone toward the cursor (the Wave 1 baseline).
    Melee,
    /// Fires a projectile (or a spread when the projectile-count boon is held)
    /// toward the cursor; the projectile damages the first enemy it hits.
    Ranged,
    /// Radial burst around the player that damages + knocks back all enemies in
    /// `AOE_RADIUS_TILES`, on a longer cooldown.
    Aoe,
}

impl WeaponType {
    /// Short label shown in the HUD.
    pub fn label(&self) -> &'static str {
        match self {
            WeaponType::Melee => "Melee",
            WeaponType::Ranged => "Ranged",
            WeaponType::Aoe => "AoE",
        }
    }

    /// Base attack cooldown (seconds) for this weapon type, before the
    /// player's attack-cooldown modifier is applied.
    pub fn base_cooldown(&self) -> f32 {
        match self {
            WeaponType::Melee => crate::tuning::ATTACK_COOLDOWN,
            WeaponType::Ranged => crate::tuning::RANGED_COOLDOWN,
            WeaponType::Aoe => crate::tuning::AOE_COOLDOWN,
        }
    }
}

/// Stats granted by a weapon pickup. Names come from a small fixed table, and
/// each weapon carries a `WeaponType` that dictates its attack behavior.
#[derive(Component, Debug, Clone)]
pub struct WeaponStats {
    pub strength_bonus: i64,
    pub name: &'static str,
    pub weapon_type: WeaponType,
}

/// Fixed table of weapons that can drop from enemies. The sprite index reuses an
/// existing tilesheet icon (a sword-like glyph) so no new assets are required.
/// Each entry: (name, strength_bonus, weapon_type).
pub const WEAPON_TABLE: &[(&str, i64, WeaponType)] = &[
    ("Rusty Dagger", 1, WeaponType::Melee),
    ("Iron Sword", 2, WeaponType::Melee),
    ("Steel Mace", 3, WeaponType::Melee),
    ("War Axe", 4, WeaponType::Aoe),
    ("Enchanted Blade", 6, WeaponType::Melee),
    ("Short Bow", 2, WeaponType::Ranged),
    ("Hunter's Crossbow", 4, WeaponType::Ranged),
    ("Frost Wand", 3, WeaponType::Ranged),
    ("Thunder Hammer", 5, WeaponType::Aoe),
];

impl WeaponStats {
    /// Pick a random weapon from the fixed table.
    pub fn random() -> Self {
        let (name, strength_bonus, weapon_type) =
            WEAPON_TABLE[rand::random_range(0..WEAPON_TABLE.len())];
        WeaponStats {
            strength_bonus,
            name,
            weapon_type,
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

/// Which side fired a projectile. Player projectiles damage enemies; enemy
/// projectiles damage the player (respecting dash i-frames). Used by
/// `move_projectiles` to pick the correct (disjoint) target query, avoiding
/// B0001 query-conflict panics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileFaction {
    Player,
    Enemy,
}

/// A ranged projectile (Wave 3 player shots; Wave 4 enemy shots). Travels in
/// `velocity` until it hits a valid target, hits a wall, or exceeds its
/// remaining travel distance, then despawns. Carries the (already crit-rolled)
/// damage and knockback to apply.
#[derive(Component, Debug)]
pub struct Projectile {
    /// World-units/sec travel velocity.
    pub velocity: Vec2,
    /// Remaining world-unit distance before the projectile fizzles out.
    pub remaining: f32,
    /// Damage applied to the first valid target hit.
    pub damage: i64,
    /// Knockback impulse (world units/sec) applied to the target hit.
    pub knockback: f32,
    /// Whether this shot rolled a critical hit (for damage-number coloring).
    pub crit: bool,
    /// Who fired this shot (decides what it can hit).
    pub faction: ProjectileFaction,
}

/// Wave 4 archer shooting cadence: ticks while the archer is alive; when finished
/// (and the player is in fire range) it fires an enemy arrow and resets.
#[derive(Component, Debug)]
pub struct ArcherShoot(pub Timer);

/// Wave 4 charger state machine: walk -> windup (telegraph) -> dash -> recover.
#[derive(Component, Debug)]
pub struct ChargeState {
    pub phase: ChargePhase,
    pub timer: Timer,
    /// Locked-in dash direction (unit vector), captured at the end of windup.
    pub dir: Vec2,
    /// True once the slam has connected during this dash (so it hits once).
    pub hit_landed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargePhase {
    Walking,
    WindingUp,
    Dashing,
    Recovering,
}

/// Wave 4 bomber fuse state. `armed` becomes true once the fuse starts (player in
/// range or bomber killed); when the timer finishes the bomber explodes.
#[derive(Component, Debug)]
pub struct BomberFuse {
    pub timer: Timer,
    pub armed: bool,
}

/// Wave 4 boss attack-pattern cadences (burst / charge / summon), each an
/// independent cooldown so the boss interleaves all three over time.
#[derive(Component, Debug)]
pub struct BossAttacks {
    pub burst: Timer,
    pub charge_cd: Timer,
    pub summon: Timer,
    /// Charge sub-state reuses `ChargeState` semantics inline.
    pub charge: ChargeState,
}

/// Marker for the floor boss. Drives the prominent boss health bar in the HUD and
/// the boss-floor extermination victory.
#[derive(Component, Debug)]
pub struct Boss;

/// Marks a tile that periodically damages any actor standing on it (Wave 4
/// hazard: spikes / lava). Carries its own DoT tick timer.
#[derive(Component, Debug)]
pub struct Hazard(pub Timer);

/// A pending explosion (bomber detonation) resolved by `resolve_explosions`. Kept
/// as a separate entity so the bomber's death and the AoE damage are decoupled
/// from any single actor query, keeping queries disjoint (B0001-safe).
#[derive(Component, Debug)]
pub struct Explosion {
    pub center: Vec2,
    pub radius: f32,
    pub damage: i64,
    pub knockback: f32,
}

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
