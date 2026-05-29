use std::collections::{BTreeMap, BTreeSet};

use crate::components::{Position, WeaponType};
use crate::tuning;

use bevy::prelude::*;

#[derive(Debug, Resource)]
pub struct Follow(pub bool);

#[derive(Debug, Resource)]
pub struct Floor(pub i64);

#[derive(Debug, Resource)]
pub struct ScaleFactor(pub f32);

#[derive(Debug, Resource)]
pub struct MousePosition(pub Vec2);

#[derive(Debug, Resource)]
pub struct Tiles(pub BTreeMap<Position, CachedTile>);

#[derive(Debug, Copy, Clone)]
pub struct CachedTile {
    pub entity: Entity,
    pub passable: bool,
}

impl Tiles {
    pub fn new() -> Self {
        Tiles(BTreeMap::new())
    }

    pub fn insert(&mut self, key: Position, cached_tile: CachedTile) {
        self.0.insert(key, cached_tile);
    }

    pub fn get(&self, key: &Position) -> Option<CachedTile> {
        self.0.get(key).copied()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct CachedHealth {
    pub entity: Entity,
    pub health: i64,
}

#[derive(Debug, Resource)]
pub struct Healths(pub BTreeMap<Position, CachedHealth>);

impl Healths {
    pub fn new() -> Self {
        Healths(BTreeMap::new())
    }

    pub fn insert(&mut self, position: Position, cached_health: CachedHealth) {
        self.0.insert(position, cached_health);
    }

    pub fn remove(&mut self, position: Position) -> Option<CachedHealth> {
        self.0.remove(&position)
    }
}

/// Tracks dropped weapon pickups by grid position, mirroring `Healths`.
#[derive(Debug, Resource)]
pub struct WeaponDrops(pub BTreeMap<Position, Entity>);

impl WeaponDrops {
    pub fn new() -> Self {
        WeaponDrops(BTreeMap::new())
    }

    pub fn insert(&mut self, position: Position, entity: Entity) {
        self.0.insert(position, entity);
    }

    pub fn remove(&mut self, position: Position) -> Option<Entity> {
        self.0.remove(&position)
    }
}

#[derive(Debug, Resource)]
pub struct Enemies {
    entity_positions: BTreeMap<Entity, Position>,
    position_entities: BTreeMap<Position, BTreeSet<Entity>>,
}

#[derive(Debug, Resource)]
pub struct SpriteTexture(pub (Handle<Image>, Handle<TextureAtlasLayout>));

/// Set of every tile position the player has ever seen (line-of-sight history).
/// Used by the fog-of-war system: revealed tiles stay rendered, but enemies are
/// only shown when currently within line of sight.
#[derive(Debug, Resource, Default)]
pub struct Revealed(pub BTreeSet<Position>);

impl Revealed {
    pub fn new() -> Self {
        Revealed(BTreeSet::new())
    }
}

/// Set of tiles currently within the player's line of sight this frame. Rebuilt
/// every frame by the fog system and consumed by `set_visibility`.
#[derive(Debug, Resource, Default)]
pub struct VisibleTiles(pub BTreeSet<Position>);

impl VisibleTiles {
    pub fn new() -> Self {
        VisibleTiles(BTreeSet::new())
    }
}

/// The objective / exit tile for the current map, if the victory condition has
/// an arrival target. `None` for Extermination-only maps (the on-screen arrow
/// and exit highlight are skipped). Set by `setup_play`.
#[derive(Debug, Resource, Clone, Copy, Default)]
pub struct ObjectiveMarker(pub Option<Position>);

/// Player stats carried across floors during a multi-floor roguelike run.
/// When present, `setup_play` uses these instead of the map's defaults so the
/// player keeps their current health and accumulated strength between floors.
#[derive(Debug, Resource, Clone, Copy)]
pub struct CarryOver {
    pub health: i64,
    pub strength: i64,
}

/// Central player stat block (Wave 3). Stores base values (seeded from tuning at
/// run start) plus accumulated boon MODIFIERS. The `effective_*` helpers combine
/// base × modifier so combat/movement/dash systems read one source of truth.
///
/// This is the resource boons mutate. It persists across floors (it is NOT torn
/// down by floor transitions), so a run's build accumulates.
#[derive(Debug, Resource, Clone)]
pub struct PlayerStats {
    // --- bases (seeded from tuning at run start) ---
    pub base_max_hp: i64,
    pub base_move_speed: f32,
    pub base_dash_cooldown: f32,
    pub base_attack_range: f32,
    pub base_attack_half_angle: f32,
    pub base_attack_knockback: f32,

    // --- boon-driven modifiers ---
    /// Multiplier on outgoing damage (1.0 == base). +25% boon adds 0.25.
    pub damage_mult: f32,
    /// Flat bonus max HP added by boons.
    pub bonus_max_hp: i64,
    /// Fractional reduction of attack cooldown (0.0 == none, capped < 1).
    pub attack_cooldown_reduction: f32,
    /// Multiplier on move speed.
    pub move_speed_mult: f32,
    /// Fractional reduction of dash cooldown.
    pub dash_cooldown_reduction: f32,
    /// Multiplier on attack range.
    pub attack_range_mult: f32,
    /// Extra radians added to the melee cone half-angle.
    pub attack_arc_bonus: f32,
    /// Multiplier on knockback.
    pub knockback_mult: f32,
    /// Crit chance (0.0..1.0). A crit deals `CRIT_MULTIPLIER`x damage.
    pub crit_chance: f32,
    /// Lifesteal fraction of damage dealt healed back.
    pub lifesteal: f32,
    /// Extra projectiles fired by the ranged weapon (0 == single shot).
    pub extra_projectiles: i64,
    /// Thorns: fraction of incoming damage reflected to the attacker.
    pub thorns: f32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        PlayerStats {
            base_max_hp: 0, // set from the map's player_health in setup_play
            base_move_speed: tuning::PLAYER_SPEED_TILES,
            base_dash_cooldown: tuning::DASH_COOLDOWN,
            base_attack_range: tuning::ATTACK_RANGE_TILES,
            base_attack_half_angle: tuning::ATTACK_HALF_ANGLE,
            base_attack_knockback: tuning::ATTACK_KNOCKBACK_TILES,

            damage_mult: 1.0,
            bonus_max_hp: 0,
            attack_cooldown_reduction: 0.0,
            move_speed_mult: 1.0,
            dash_cooldown_reduction: 0.0,
            attack_range_mult: 1.0,
            attack_arc_bonus: 0.0,
            knockback_mult: 1.0,
            crit_chance: 0.0,
            lifesteal: 0.0,
            extra_projectiles: 0,
            thorns: 0.0,
        }
    }
}

impl PlayerStats {
    /// Effective max HP = base + boon bonuses.
    pub fn effective_max_hp(&self) -> i64 {
        self.base_max_hp + self.bonus_max_hp
    }
    /// Effective move speed in tiles/sec.
    pub fn effective_move_speed(&self) -> f32 {
        self.base_move_speed * self.move_speed_mult
    }
    /// Effective dash cooldown in seconds.
    pub fn effective_dash_cooldown(&self) -> f32 {
        (self.base_dash_cooldown * (1.0 - self.dash_cooldown_reduction)).max(0.05)
    }
    /// Effective melee/attack range in tiles.
    pub fn effective_attack_range(&self) -> f32 {
        self.base_attack_range * self.attack_range_mult
    }
    /// Effective melee cone half-angle in radians.
    pub fn effective_attack_half_angle(&self) -> f32 {
        self.base_attack_half_angle + self.attack_arc_bonus
    }
    /// Effective knockback impulse in tiles/sec.
    pub fn effective_knockback(&self) -> f32 {
        self.base_attack_knockback * self.knockback_mult
    }
    /// Effective attack cooldown (seconds) for a weapon type, applying the
    /// player's cooldown-reduction boon to that weapon's base cooldown.
    pub fn effective_attack_cooldown(&self, weapon_type: WeaponType) -> f32 {
        (weapon_type.base_cooldown() * (1.0 - self.attack_cooldown_reduction)).max(0.05)
    }
    /// Rolls one attack's damage from a base strength value, applying the damage
    /// multiplier and a crit roll. Returns `(damage, was_crit)`.
    pub fn roll_damage(&self, base_strength: i64) -> (i64, bool) {
        let crit = rand::random::<f32>() < self.crit_chance;
        let mut dmg = base_strength as f32 * self.damage_mult;
        if crit {
            dmg *= tuning::CRIT_MULTIPLIER;
        }
        (dmg.round().max(1.0) as i64, crit)
    }
}

/// The player's currently-equipped weapon (Wave 3). Persists across floors like
/// `PlayerStats`. Picking up a weapon drop swaps this out, changing the attack
/// style. Starts as a plain Melee weapon at run start.
#[derive(Debug, Resource, Clone)]
pub struct ActiveWeapon {
    pub weapon_type: WeaponType,
    pub name: &'static str,
}

impl Default for ActiveWeapon {
    fn default() -> Self {
        ActiveWeapon {
            weapon_type: WeaponType::Melee,
            name: "Fists",
        }
    }
}

/// The player's gold (Wave 3). Earned by killing enemies, spent on the boon
/// screen (reroll / heal). Persists across floors within a run.
#[derive(Debug, Resource, Clone, Copy, Default)]
pub struct Gold(pub i64);

/// Names of boons the player has acquired this run, for HUD display.
#[derive(Debug, Resource, Clone, Default)]
pub struct AcquiredBoons(pub Vec<&'static str>);

/// The three boons currently offered on the BoonSelect screen, plus whether a
/// reroll has happened. Rebuilt each time BoonSelect is entered.
#[derive(Debug, Resource, Clone, Default)]
pub struct BoonOffer {
    pub choices: Vec<crate::systems::boons::Boon>,
}

#[derive(Debug, Resource, Clone)]
pub struct Statistics {
    pub enemies_killed: i64,
    pub floors_completed: i64,
    pub damage_taken: i64,
    pub damage_dealt: i64,
    pub health_collected: i64,
}

impl Statistics {
    pub fn new() -> Self {
        Statistics {
            enemies_killed: 0,
            floors_completed: 0,
            damage_taken: 0,
            damage_dealt: 0,
            health_collected: 0,
        }
    }
}

impl Enemies {
    pub fn new() -> Self {
        Enemies {
            entity_positions: BTreeMap::new(),
            position_entities: BTreeMap::new(),
        }
    }

    /// Returns the set of enemies occupying a tile. Part of the public
    /// occupancy API; currently unused by systems (real-time AI reads
    /// `occupied_position` instead) but kept for callers/tests.
    #[allow(dead_code)]
    pub fn enemies_at(&self, position: Position) -> Option<&BTreeSet<Entity>> {
        self.position_entities.get(&position)
    }

    pub fn occupied_position(&self, position: Position) -> bool {
        self.position_entities
            .get(&position)
            .map_or_else(|| false, |set| !set.is_empty())
    }

    pub fn insert(&mut self, position: Position, entity: Entity) {
        match self.entity_positions.entry(entity) {
            // If we don't have a mapping Entity => Position, then we have to insert one
            std::collections::btree_map::Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(position);
                match self.position_entities.entry(position) {
                    std::collections::btree_map::Entry::Vacant(vacant_entry) => {
                        let mut set = BTreeSet::new();
                        set.insert(entity);
                        vacant_entry.insert(set);
                    }
                    std::collections::btree_map::Entry::Occupied(mut occupied_entry) => {
                        occupied_entry.get_mut().insert(entity);
                    }
                }
            }
            std::collections::btree_map::Entry::Occupied(mut occupied_entry) => {
                let current_position = occupied_entry.get().clone();
                occupied_entry.insert(position);
                match self.position_entities.entry(current_position) {
                    std::collections::btree_map::Entry::Vacant(_vacant_entry) => unreachable!(),
                    std::collections::btree_map::Entry::Occupied(mut occupied_entry) => {
                        occupied_entry.get_mut().remove(&entity);
                        match self.position_entities.entry(position) {
                            std::collections::btree_map::Entry::Vacant(vacant_entry) => {
                                let mut set = BTreeSet::new();
                                set.insert(entity);
                                vacant_entry.insert(set);
                            }
                            std::collections::btree_map::Entry::Occupied(mut occupied_entry) => {
                                occupied_entry.get_mut().insert(entity);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_enemies() {
    let mut enemies = Enemies::new();
    for i in 0..30u32 {
        enemies.insert(
            Position {
                x: (i / 4) as i64,
                y: (i / 4) as i64,
                z: (i / 4) as i64,
            },
            Entity::from_raw_u32(i).unwrap(),
        );
    }
    for i in 0..30u32 {
        enemies.insert(
            Position {
                x: (i / 4 + 1) as i64,
                y: (i / 4 + 1) as i64,
                z: (i / 4 + 1) as i64,
            },
            Entity::from_raw_u32(i).unwrap(),
        );
    }
}
