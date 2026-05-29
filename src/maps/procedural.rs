use std::collections::BTreeSet;

use crate::{
    components::{EnemyType, Position},
    map::{Enemy, Health, Map, Room, Tile, VictoryCondition},
    tuning,
};

// Sprite indices reused from the hand-authored maps (see unbeatable.rs / avoidance.rs).
const FLOOR_SPRITE: u64 = 960; // passable floor
const WALL_SPRITE: u64 = 15 * 64 - 13; // 947, impassable wall
const HEALTH_SPRITE: u64 = 64 * 23 + 45; // 1517, health pickup
const PLAYER_SPRITE: u64 = 31 * 64 + 20; // 2004
/// Hazard tiles are PASSABLE (so they never block the only path) but are tagged
/// with a `Hazard` component + distinct tint at spawn. `setup_play` recognizes a
/// tile as a hazard by this sprite index. The DoT comes from the component, not
/// the tile passability, so hazards never block solvability.
pub const HAZARD_SPRITE: u64 = 64 * 7 + 1; // 449, a distinct glyph for lava/spikes

const GRID: i64 = 60;
const Z: i64 = 0;

#[derive(Clone, Copy, Debug)]
struct Rect {
    x: i64,
    y: i64,
    w: i64,
    h: i64,
}

impl Rect {
    fn center(&self) -> Position {
        Position::new(self.x + self.w / 2, self.y + self.h / 2, Z)
    }

    /// Inclusive overlap test, padded by one tile so rooms never share walls.
    fn intersects(&self, other: &Rect) -> bool {
        self.x - 1 <= other.x + other.w
            && self.x + self.w + 1 >= other.x
            && self.y - 1 <= other.y + other.h
            && self.y + self.h + 1 >= other.y
    }
}

/// Procedurally generate a dungeon: 6-12 non-overlapping rectangular rooms in a
/// ~60x60 grid connected by L-shaped corridors, populated with scaled enemies and
/// a handful of health pickups. Victory is reaching the room farthest from start.
pub fn procedural() -> Map {
    // Carve rooms.
    let target_rooms = rand::random_range(6..=12);
    let mut rooms: Vec<Rect> = Vec::new();
    let mut attempts = 0;
    while rooms.len() < target_rooms && attempts < 500 {
        attempts += 1;
        let w = rand::random_range(4..=10);
        let h = rand::random_range(4..=10);
        let x = rand::random_range(1..(GRID - w - 1));
        let y = rand::random_range(1..(GRID - h - 1));
        let candidate = Rect { x, y, w, h };
        if rooms.iter().all(|r| !r.intersects(&candidate)) {
            rooms.push(candidate);
        }
    }

    // Fallback: guarantee at least two rooms so the level is always winnable.
    if rooms.len() < 2 {
        rooms.clear();
        rooms.push(Rect { x: 2, y: 2, w: 8, h: 8 });
        rooms.push(Rect { x: 40, y: 40, w: 8, h: 8 });
    }

    // Collect floor tiles (a set so corridors and rooms can overlap freely).
    let mut floor_tiles: BTreeSet<Position> = BTreeSet::new();
    for room in &rooms {
        for dx in 0..room.w {
            for dy in 0..room.h {
                floor_tiles.insert(Position::new(room.x + dx, room.y + dy, Z));
            }
        }
    }

    // Connect each room to the next with an L-shaped corridor.
    for pair in rooms.windows(2) {
        let a = pair[0].center();
        let b = pair[1].center();
        // Randomize corridor elbow orientation for variety.
        if rand::random() {
            carve_h_corridor(&mut floor_tiles, a.x, b.x, a.y);
            carve_v_corridor(&mut floor_tiles, a.y, b.y, b.x);
        } else {
            carve_v_corridor(&mut floor_tiles, a.y, b.y, a.x);
            carve_h_corridor(&mut floor_tiles, a.x, b.x, b.y);
        }
    }

    let start = rooms[0].center();

    // Victory room = the room whose center is farthest (Manhattan) from start.
    let victory_room = rooms
        .iter()
        .skip(1)
        .max_by_key(|r| {
            let c = r.center();
            (c.x - start.x).abs() + (c.y - start.y).abs()
        })
        .copied()
        .unwrap_or(rooms[rooms.len() - 1]);
    let victory_position = victory_room.center();

    let mut room = Room::new(start);

    // Lay floor tiles, then surround the walkable area with wall tiles so the
    // dungeon has solid borders (used by line-of-sight blocking).
    for pos in &floor_tiles {
        room.add_tile(*pos, Tile::new(FLOOR_SPRITE, true));
    }
    let mut wall_positions: BTreeSet<Position> = BTreeSet::new();
    for pos in &floor_tiles {
        for dx in -1..=1 {
            for dy in -1..=1 {
                let neighbor = Position::new(pos.x + dx, pos.y + dy, Z);
                if !floor_tiles.contains(&neighbor) {
                    wall_positions.insert(neighbor);
                }
            }
        }
    }
    for pos in &wall_positions {
        room.add_tile(*pos, Tile::new(WALL_SPRITE, false));
    }

    // Scatter enemies across rooms (never on the start tile), and accumulate the
    // total enemy HP so the player can be scaled to roughly match the level.
    let mut total_enemy_hp: i64 = 0;
    for r in &rooms {
        // Skip the very first cells of the start room near the player.
        let n_enemies = rand::random_range(0..4usize); // 0..=3 per room
        for _ in 0..n_enemies {
            let ex = r.x + rand::random_range(0..r.w);
            let ey = r.y + rand::random_range(0..r.h);
            let pos = Position::new(ex, ey, Z);
            if pos == start || pos == victory_position {
                continue;
            }
            let enemy_type = EnemyType::random();
            let (health, strength) = enemy_type.get_stats(0);
            total_enemy_hp += health.max(1);
            room.add_enemy(
                pos,
                Enemy::new(
                    enemy_type.sprite_index() as u64,
                    health.max(1) as u64,
                    strength.max(1) as u64,
                    Enemy::circular_wake_zone(pos, 5),
                ),
            );
        }
    }

    // Scatter 1-3 health pickups across non-start rooms.
    let n_pickups = rand::random_range(1..=3); // 1..=3
    for _ in 0..n_pickups {
        let r = rooms[rand::random_range(0..rooms.len())];
        let hx = r.x + rand::random_range(0..r.w);
        let hy = r.y + rand::random_range(0..r.h);
        let pos = Position::new(hx, hy, Z);
        if pos == start {
            continue;
        }
        room.add_health(
            pos,
            Health {
                sprite_index: HEALTH_SPRITE,
                health: 50,
            },
        );
    }

    // Scatter a few hazard patches. Hazards are PASSABLE tiles (they replace the
    // floor sprite but stay walkable), so they never block the only path -- the
    // level stays solvable. We only place them inside ROOM interiors (never on
    // start/victory tiles), so corridors -- the connectivity spine -- are always
    // hazard-free, leaving an obvious clean route.
    scatter_hazards(&mut room, &rooms, start, victory_position);

    Map {
        player_health: compute_reasonable_player_health(total_enemy_hp),
        player_strength: compute_reasonable_player_strength(total_enemy_hp),
        room,
        player_sprite: PLAYER_SPRITE,
        victory_condition: VictoryCondition::Arrival(victory_position),
    }
}

/// Scatters `HAZARD_PATCH_COUNT` small hazard patches inside room interiors,
/// overwriting the floor tile sprite with `HAZARD_SPRITE` (still passable). Skips
/// the start and victory tiles. Because hazards remain walkable, the level is
/// always solvable regardless of placement.
fn scatter_hazards(room: &mut Room, rooms: &[Rect], start: Position, victory: Position) {
    if rooms.len() < 2 {
        return;
    }
    for _ in 0..tuning::HAZARD_PATCH_COUNT {
        // Choose a non-start room interior cell as the patch origin.
        let r = rooms[rand::random_range(1..rooms.len())];
        let ox = r.x + rand::random_range(0..r.w);
        let oy = r.y + rand::random_range(0..r.h);
        let patch_size = rand::random_range(1..=tuning::HAZARD_PATCH_MAX_TILES);
        for i in 0..patch_size {
            // Grow the patch as a short horizontal/vertical run within the room.
            let (dx, dy) = if i % 2 == 0 { (i as i64 / 2, 0) } else { (0, i as i64 / 2 + 1) };
            let hx = ox + dx;
            let hy = oy + dy;
            // Keep the patch inside this room's bounds.
            if hx < r.x || hx >= r.x + r.w || hy < r.y || hy >= r.y + r.h {
                continue;
            }
            let pos = Position::new(hx, hy, Z);
            if pos == start || pos == victory {
                continue;
            }
            // Only convert tiles that are floor (inside the room).
            room.add_tile(pos, Tile::new(HAZARD_SPRITE, true));
        }
    }
}

/// Generates a BOSS arena: one large open room with a single high-HP boss in the
/// center, plus a couple of weak adds. Victory is Extermination (kill the boss
/// and every add). Triggered on boss floors by `next_floor`.
pub fn boss_floor() -> Map {
    // A single large open arena, centered in the grid.
    let arena = Rect { x: 14, y: 14, w: 30, h: 30 };
    let start = Position::new(arena.x + arena.w / 2, arena.y + 2, Z);

    let mut floor_tiles: BTreeSet<Position> = BTreeSet::new();
    for dx in 0..arena.w {
        for dy in 0..arena.h {
            floor_tiles.insert(Position::new(arena.x + dx, arena.y + dy, Z));
        }
    }

    let mut room = Room::new(start);
    for pos in &floor_tiles {
        room.add_tile(*pos, Tile::new(FLOOR_SPRITE, true));
    }
    // Solid wall border.
    let mut wall_positions: BTreeSet<Position> = BTreeSet::new();
    for pos in &floor_tiles {
        for dx in -1..=1 {
            for dy in -1..=1 {
                let neighbor = Position::new(pos.x + dx, pos.y + dy, Z);
                if !floor_tiles.contains(&neighbor) {
                    wall_positions.insert(neighbor);
                }
            }
        }
    }
    for pos in &wall_positions {
        room.add_tile(*pos, Tile::new(WALL_SPRITE, false));
    }

    // The boss in the arena center. setup_play recomputes its scaled stats from
    // EnemyType::Boss; the stored health here is a placeholder. A large wake zone
    // ensures it engages as soon as the player enters.
    let boss_type = EnemyType::Boss;
    let (boss_hp, boss_str) = boss_type.get_stats(0);
    let boss_pos = Position::new(arena.x + arena.w / 2, arena.y + arena.h / 2, Z);
    room.add_enemy(
        boss_pos,
        Enemy::new(
            boss_type.sprite_index() as u64,
            boss_hp.max(1) as u64,
            boss_str.max(1) as u64,
            Enemy::circular_wake_zone(boss_pos, 30),
        ),
    );

    // A couple of weak adds flanking the boss.
    for (ax, ay) in [(-4i64, 2i64), (4, 2)] {
        let add_type = EnemyType::Skeleton;
        let (hp, st) = add_type.get_stats(0);
        let pos = Position::new(boss_pos.x + ax, boss_pos.y + ay, Z);
        room.add_enemy(
            pos,
            Enemy::new(
                add_type.sprite_index() as u64,
                hp.max(1) as u64,
                st.max(1) as u64,
                Enemy::circular_wake_zone(boss_pos, 30),
            ),
        );
    }

    // A health pickup tucked in a corner to reward aggressive play.
    room.add_health(
        Position::new(arena.x + 2, arena.y + arena.h - 3, Z),
        Health {
            sprite_index: HEALTH_SPRITE,
            health: 80,
        },
    );

    Map {
        // Boss floors get a generous player HP pool (boss fights are long).
        player_health: 220,
        player_strength: 14,
        room,
        player_sprite: PLAYER_SPRITE,
        victory_condition: VictoryCondition::Extermination,
    }
}

fn carve_h_corridor(floor: &mut BTreeSet<Position>, x1: i64, x2: i64, y: i64) {
    let (lo, hi) = (x1.min(x2), x1.max(x2));
    for x in lo..=hi {
        floor.insert(Position::new(x, y, Z));
    }
}

fn carve_v_corridor(floor: &mut BTreeSet<Position>, y1: i64, y2: i64, x: i64) {
    let (lo, hi) = (y1.min(y2), y1.max(y2));
    for y in lo..=hi {
        floor.insert(Position::new(x, y, Z));
    }
}

/// Give the player enough HP to plausibly survive the level's total enemy damage
/// while still being threatened. Scales with total enemy HP.
fn compute_reasonable_player_health(total_enemy_hp: i64) -> u64 {
    (total_enemy_hp * 3).clamp(50, 2000) as u64
}

/// Strength scaled so the player can clear the level in a reasonable number of
/// hits without trivializing tanky enemies.
fn compute_reasonable_player_strength(total_enemy_hp: i64) -> u64 {
    (3 + total_enemy_hp / 20).clamp(3, 30) as u64
}
