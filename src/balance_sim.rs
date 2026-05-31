//! Headless balance simulation (test-only).
//!
//! Models full 10-floor runs using the REAL `EnemyType::get_stats`, the `tuning`
//! constants, and the boon magnitudes, to estimate the difficulty curve, win
//! rate, and where/how runs end. It is a MODEL (sequential room fights, cone
//! multi-hit, a tunable `dodge` factor for player skill, bombers as one-shot
//! blasts), so the ABSOLUTE win rate is approximate -- the value is the curve
//! SHAPE and the death clustering (which floors / enemy types end runs), and
//! how those shift when the tuning numbers change.
//!
//! Run:  cargo test --bin dungeon-crawler balance_sim -- --ignored --nocapture
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::needless_range_loop
)]

use crate::components::EnemyType;
use crate::tuning;

#[derive(Clone, Copy)]
struct E {
    ty: EnemyType,
    hp: f32,
    strength: f32,
}

/// Per-second damage an enemy applies while the player is engaged with its room.
fn enemy_dps(e: &E) -> f32 {
    match e.ty {
        EnemyType::Skeleton | EnemyType::Orc | EnemyType::Ghost => {
            e.strength / tuning::ENEMY_ATTACK_COOLDOWN
        }
        EnemyType::Archer => e.strength / tuning::ARCHER_SHOOT_COOLDOWN,
        // Charger: one slam (CHARGER_DAMAGE_MULT) per ~2s cycle.
        EnemyType::Charger => e.strength * tuning::CHARGER_DAMAGE_MULT / 2.0,
        // Bomber handled as a one-time blast in fight(), not continuous DPS.
        EnemyType::Bomber => 0.0,
        // Boss: burst + charge + contact, rough aggregate.
        EnemyType::Boss => e.strength * 3.0,
    }
}

/// Burst (non-continuous) damage an enemy deals once during a room fight.
fn enemy_burst(e: &E) -> f32 {
    if e.ty == EnemyType::Bomber {
        e.strength * tuning::BOMBER_DAMAGE_MULT
    } else {
        0.0
    }
}


struct Player {
    hp: f32,
    max_hp: f32,
    base_strength: f32,
    damage_mult: f32,
    atk_cd: f32,
    crit: f32,
    lifesteal: f32,
    dodge: f32,
    cone_bonus: f32,
}

impl Player {
    fn dmg_per_swing(&self) -> f32 {
        let crit_factor = 1.0 + self.crit * (tuning::CRIT_MULTIPLIER - 1.0);
        self.base_strength * self.damage_mult * crit_factor
    }
}

/// Apply a random boon (models the player's pick each floor) using real magnitudes.
fn apply_random_boon(p: &mut Player) {
    match rand::random_range(0..12) {
        0 => p.damage_mult += tuning::BOON_DAMAGE,
        1 => {
            p.max_hp += tuning::BOON_MAX_HP as f32;
            p.hp += tuning::BOON_MAX_HP as f32;
        }
        2 => p.atk_cd *= 1.0 - tuning::BOON_ATTACK_COOLDOWN,
        3 => p.dodge = (p.dodge + 0.05).min(0.92), // move speed -> dodge more
        4 => p.damage_mult += 0.10,                // +projectile (melee model: minor dmg)
        5 => p.lifesteal += tuning::BOON_LIFESTEAL,
        6 => p.dodge = (p.dodge + 0.05).min(0.92), // dash cd -> dodge more
        7 => p.cone_bonus += 0.5,                  // +range
        8 => p.dodge = (p.dodge + 0.02).min(0.92), // +knockback (minor survivability)
        9 => p.crit = (p.crit + tuning::BOON_CRIT).min(1.0),
        10 => p.damage_mult += 0.10,               // thorns ~ faster clears
        _ => p.cone_bonus += 0.5,                  // +arc
    }
}

/// Mirrors maps::procedural enemy scatter: 6-12 rooms, 0-3 enemies each, weighted type.
fn gen_floor(depth: i64) -> Vec<Vec<E>> {
    let rooms = rand::random_range(6..=12);
    let mut out = Vec::new();
    for _ in 0..rooms {
        let n = rand::random_range(0..4);
        let mut room = Vec::new();
        for _ in 0..n {
            let ty = EnemyType::random();
            let (hp, st) = ty.get_stats(depth);
            room.push(E { ty, hp: hp.max(1) as f32, strength: st.max(1) as f32 });
        }
        if !room.is_empty() {
            out.push(room);
        }
    }
    out
}

fn gen_boss_floor(depth: i64) -> Vec<Vec<E>> {
    let (bhp, bst) = EnemyType::Boss.get_stats(depth);
    let (shp, sst) = EnemyType::Skeleton.get_stats(depth);
    vec![vec![
        E { ty: EnemyType::Boss, hp: bhp.max(1) as f32, strength: bst.max(1) as f32 },
        E { ty: EnemyType::Skeleton, hp: shp.max(1) as f32, strength: sst.max(1) as f32 },
        E { ty: EnemyType::Skeleton, hp: shp.max(1) as f32, strength: sst.max(1) as f32 },
    ]]
}

enum Outcome {
    Won,
    Died { floor: i64, cause: EnemyType },
}

fn sim_run(base_dodge: f32) -> Outcome {
    // Floor-1 roster (depth 0) sets the player's base HP / strength.
    let f1 = gen_floor(0);
    let total1: i64 = f1.iter().flatten().map(|e| e.hp as i64).sum();
    let mut p = Player {
        hp: crate::maps::procedural::compute_reasonable_player_health(total1) as f32,
        max_hp: crate::maps::procedural::compute_reasonable_player_health(total1) as f32,
        base_strength: crate::maps::procedural::compute_reasonable_player_strength(total1) as f32,
        damage_mult: 1.0,
        atk_cd: tuning::ATTACK_COOLDOWN,
        crit: 0.0,
        lifesteal: 0.0,
        dodge: base_dodge,
        cone_bonus: 0.0,
    };

    for floor in 1..=10i64 {
        let depth = floor - 1;
        let rooms = if floor % tuning::BOSS_FLOOR_INTERVAL == 0 {
            gen_boss_floor(depth)
        } else {
            gen_floor(depth)
        };

        for room in &rooms {
            if room.is_empty() {
                continue;
            }
            let room_hp: f32 = room.iter().map(|e| e.hp).sum();
            let cone = (room.len() as f32).min(3.0) + p.cone_bonus;
            let player_dps = (p.dmg_per_swing() * cone / p.atk_cd).max(0.1);
            let clear_time = room_hp / player_dps;

            let cont: f32 = room.iter().map(enemy_dps).sum::<f32>() * clear_time;
            let burst: f32 = room.iter().map(enemy_burst).sum();
            p.hp -= (cont + burst) * (1.0 - p.dodge);
            p.hp = (p.hp + p.lifesteal * room_hp).min(p.max_hp);

            if p.hp <= 0.0 {
                let cause = room
                    .iter()
                    .max_by(|a, b| {
                        (enemy_dps(a) + enemy_burst(a))
                            .partial_cmp(&(enemy_dps(b) + enemy_burst(b)))
                            .unwrap()
                    })
                    .map(|e| e.ty)
                    .unwrap_or(EnemyType::Skeleton);
                return Outcome::Died { floor, cause };
            }
        }

        // Floor cleared: grab ~half of 0-2 health pickups, descent heal, pick a boon.
        let pickups = rand::random_range(0..=2) as f32 * 50.0 * 0.5;
        p.hp = (p.hp + pickups + 12.0).min(p.max_hp);
        if floor < 10 {
            apply_random_boon(&mut p);
        }
    }
    Outcome::Won
}

#[test]
#[ignore]
fn balance_sim() {
    const N: usize = 5000;
    for dodge in [0.30f32, 0.45, 0.60] {
        let mut wins = 0usize;
        let mut by_floor = [0usize; 11];
        let mut by_cause: std::collections::BTreeMap<String, usize> = Default::default();
        let mut reached = 0i64;
        for _ in 0..N {
            match sim_run(dodge) {
                Outcome::Won => {
                    wins += 1;
                    reached += 10;
                }
                Outcome::Died { floor, cause } => {
                    by_floor[floor as usize] += 1;
                    *by_cause.entry(format!("{cause:?}")).or_default() += 1;
                    reached += floor;
                }
            }
        }
        println!(
            "\n=== BALANCE SIM | dodge {:.2} | {} runs ===",
            dodge, N
        );
        println!(
            "win rate {:>5.1}%   avg floor reached {:.2}",
            100.0 * wins as f32 / N as f32,
            reached as f32 / N as f32
        );
        println!("deaths by floor:");
        for f in 1..=10 {
            let d = by_floor[f];
            let pct = 100.0 * d as f32 / N as f32;
            println!("  f{:>2} {:>5} {:>4.1}% {}", f, d, pct, "#".repeat((pct as usize).min(40)));
        }
        let mut causes: Vec<_> = by_cause.into_iter().collect();
        causes.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
        let total_deaths = (N - wins).max(1);
        println!("death causes (share of deaths):");
        for (ty, c) in causes {
            println!("  {:>9} {:>5} {:>4.1}%", ty, c, 100.0 * c as f32 / total_deaths as f32);
        }
    }
}
