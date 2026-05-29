//! Data-driven boon pool for the floor-clear "choose 1 of 3" meta-loop (Wave 3).
//!
//! Boons are pure data: a `name`, a `description`, and a `BoonKind` enum tag.
//! `apply` mutates the player's `PlayerStats` (and, for the +max-HP boon, heals
//! the live player). Keeping the catalog as a `const` slice makes it trivial to
//! extend: add a `BoonKind` variant + a `BOON_POOL` entry + an `apply` arm.

use crate::resources::PlayerStats;
use crate::tuning;

/// The distinct effects a boon can have. Each maps to one arm of `apply`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoonKind {
    Damage,
    MaxHp,
    AttackCooldown,
    MoveSpeed,
    Projectile,
    Lifesteal,
    DashCooldown,
    AttackRange,
    Knockback,
    Thorns,
    Crit,
    AttackArc,
}

/// A single offerable boon: display text + the effect tag.
#[derive(Debug, Clone, Copy)]
pub struct Boon {
    pub name: &'static str,
    pub description: &'static str,
    pub kind: BoonKind,
}

/// The full boon catalog. Three are sampled (without replacement) each floor.
pub const BOON_POOL: &[Boon] = &[
    Boon {
        name: "Berserker",
        description: "+25% attack damage",
        kind: BoonKind::Damage,
    },
    Boon {
        name: "Vitality",
        description: "+20 max HP (and heal 20)",
        kind: BoonKind::MaxHp,
    },
    Boon {
        name: "Swift Strikes",
        description: "-20% attack cooldown",
        kind: BoonKind::AttackCooldown,
    },
    Boon {
        name: "Fleet Footed",
        description: "+20% move speed",
        kind: BoonKind::MoveSpeed,
    },
    Boon {
        name: "Multishot",
        description: "+1 projectile (ranged weapons)",
        kind: BoonKind::Projectile,
    },
    Boon {
        name: "Vampiric",
        description: "Heal 10% of damage dealt",
        kind: BoonKind::Lifesteal,
    },
    Boon {
        name: "Quick Reflexes",
        description: "-35% dash cooldown",
        kind: BoonKind::DashCooldown,
    },
    Boon {
        name: "Long Reach",
        description: "+40% attack range",
        kind: BoonKind::AttackRange,
    },
    Boon {
        name: "Heavy Hitter",
        description: "+50% knockback",
        kind: BoonKind::Knockback,
    },
    Boon {
        name: "Spiked Armor",
        description: "Reflect 50% of damage taken (thorns)",
        kind: BoonKind::Thorns,
    },
    Boon {
        name: "Deadeye",
        description: "+15% critical hit chance",
        kind: BoonKind::Crit,
    },
    Boon {
        name: "Wide Swing",
        description: "+20deg melee arc",
        kind: BoonKind::AttackArc,
    },
];

/// Sample `n` distinct boons from the pool (without replacement).
pub fn sample(n: usize) -> Vec<Boon> {
    let mut indices: Vec<usize> = (0..BOON_POOL.len()).collect();
    // Fisher-Yates partial shuffle using rand 0.9.
    let len = indices.len();
    for i in 0..len.min(n) {
        let j = i + rand::random_range(0..(len - i));
        indices.swap(i, j);
    }
    indices
        .into_iter()
        .take(n.min(len))
        .map(|i| BOON_POOL[i])
        .collect()
}

/// Applies a boon's effect to the player's stats. Returns the amount of HP the
/// player should be healed by as a result (only the MaxHp boon heals; the
/// caller applies it to the live `Health` component since stats don't own HP).
pub fn apply(boon: &Boon, stats: &mut PlayerStats) -> i64 {
    match boon.kind {
        BoonKind::Damage => {
            stats.damage_mult += tuning::BOON_DAMAGE;
            0
        }
        BoonKind::MaxHp => {
            stats.bonus_max_hp += tuning::BOON_MAX_HP;
            tuning::BOON_MAX_HP
        }
        BoonKind::AttackCooldown => {
            // Stack multiplicatively so it can't reach/exceed 100% reduction.
            stats.attack_cooldown_reduction =
                1.0 - (1.0 - stats.attack_cooldown_reduction) * (1.0 - tuning::BOON_ATTACK_COOLDOWN);
            0
        }
        BoonKind::MoveSpeed => {
            stats.move_speed_mult += tuning::BOON_MOVE_SPEED;
            0
        }
        BoonKind::Projectile => {
            stats.extra_projectiles += tuning::BOON_PROJECTILE;
            0
        }
        BoonKind::Lifesteal => {
            stats.lifesteal += tuning::BOON_LIFESTEAL;
            0
        }
        BoonKind::DashCooldown => {
            stats.dash_cooldown_reduction =
                1.0 - (1.0 - stats.dash_cooldown_reduction) * (1.0 - tuning::BOON_DASH_COOLDOWN);
            0
        }
        BoonKind::AttackRange => {
            stats.attack_range_mult += tuning::BOON_ATTACK_RANGE;
            0
        }
        BoonKind::Knockback => {
            stats.knockback_mult += tuning::BOON_KNOCKBACK;
            0
        }
        BoonKind::Thorns => {
            stats.thorns += tuning::BOON_THORNS;
            0
        }
        BoonKind::Crit => {
            stats.crit_chance = (stats.crit_chance + tuning::BOON_CRIT).min(1.0);
            0
        }
        BoonKind::AttackArc => {
            stats.attack_arc_bonus += tuning::BOON_ATTACK_ARC;
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_returns_distinct_choices() {
        let s = sample(3);
        assert_eq!(s.len(), 3);
        // Distinct names (sampling is without replacement).
        for i in 0..s.len() {
            for j in (i + 1)..s.len() {
                assert_ne!(s[i].name, s[j].name);
            }
        }
    }

    #[test]
    fn sample_clamps_to_pool_size() {
        let s = sample(1000);
        assert_eq!(s.len(), BOON_POOL.len());
    }

    #[test]
    fn damage_boon_increases_multiplier() {
        let mut stats = PlayerStats::default();
        let boon = BOON_POOL.iter().find(|b| b.kind == BoonKind::Damage).unwrap();
        let before = stats.damage_mult;
        let healed = apply(boon, &mut stats);
        assert_eq!(healed, 0);
        assert!(stats.damage_mult > before);
    }

    #[test]
    fn maxhp_boon_heals_and_raises_cap() {
        let mut stats = PlayerStats::default();
        let boon = BOON_POOL.iter().find(|b| b.kind == BoonKind::MaxHp).unwrap();
        let healed = apply(boon, &mut stats);
        assert_eq!(healed, tuning::BOON_MAX_HP);
        assert_eq!(stats.bonus_max_hp, tuning::BOON_MAX_HP);
    }

    #[test]
    fn cooldown_reduction_never_reaches_full() {
        let mut stats = PlayerStats::default();
        let boon = BOON_POOL
            .iter()
            .find(|b| b.kind == BoonKind::AttackCooldown)
            .unwrap();
        for _ in 0..50 {
            apply(boon, &mut stats);
        }
        assert!(stats.attack_cooldown_reduction < 1.0);
    }
}
