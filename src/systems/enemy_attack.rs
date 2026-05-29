use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::systems::dash::player_invulnerable;
use crate::systems::particle_system::spawn_particle;
use crate::tuning;

/// Real-time enemy attacks with a telegraph.
///
/// When an awake enemy is within `ENEMY_ATTACK_RANGE` of the player and off
/// cooldown, it enters a brief windup (telegraph): its sprite pulses larger so
/// the player can react/dodge. When the telegraph completes, if the player is
/// still in range it strikes for `Strength` damage (knockback + hit-flash on the
/// player), unless the player is dash-invulnerable. Then it goes on cooldown.
///
/// Disjoint queries: enemies (`With<Enemy>, Without<Player>`) vs player
/// (`With<Player>, Without<Enemy>`). The enemy query mutates `Transform` (for the
/// telegraph pulse) but the player query does not touch enemy transforms.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn enemy_attack(
    mut commands: Commands,
    time: Res<Time>,
    scale_factor: Res<ScaleFactor>,
    mut statistics: ResMut<Statistics>,
    mut juice: ResMut<Juice>,
    mut sfx: MessageWriter<SfxEvent>,
    player_stats: Res<PlayerStats>,
    mut enemy_query: Query<
        (
            &WorldPos,
            &Strength,
            &Awake,
            &mut EnemyAttack,
            &mut Transform,
            &mut Health,
        ),
        // Exclude special enemies: Charger/Bomber/Boss have their own attack +
        // telegraph-scale systems (enemy_special). Letting the generic melee here
        // also drive their Transform.scale caused flicker + a double attack.
        (
            With<Enemy>,
            Without<Player>,
            Without<ChargeState>,
            Without<BomberFuse>,
            Without<Boss>,
        ),
    >,
    mut player_query: Query<
        (Entity, &WorldPos, &mut Health, &mut Knockback, &Dash),
        (With<Player>, Without<Enemy>),
    >,
) {
    let Some((player_entity, player_world, mut player_health, mut player_knockback, dash)) =
        player_query.iter_mut().next()
    else {
        return;
    };
    let dt = time.delta();
    let scale = scale_factor.0;
    let range = tuning::ENEMY_ATTACK_RANGE_TILES * scale;
    let invulnerable = player_invulnerable(dash);

    for (enemy_world, strength, awake, mut attack, mut transform, mut enemy_health) in
        enemy_query.iter_mut()
    {
        attack.cooldown.tick(dt);

        let in_range = (player_world.0 - enemy_world.0).length() <= range;

        if attack.winding_up {
            attack.telegraph.tick(dt);
            // Pulse scale up during windup as the telegraph.
            let f = attack.telegraph.fraction();
            transform.scale = Vec3::splat(1.0 + 0.4 * f);

            if attack.telegraph.is_finished() {
                transform.scale = Vec3::ONE;
                attack.winding_up = false;
                attack.cooldown = Timer::from_seconds(
                    tuning::ENEMY_ATTACK_COOLDOWN,
                    TimerMode::Once,
                );

                // Land the hit only if the player is still in range and not
                // dodging through it.
                if in_range && !invulnerable {
                    player_health.0 -= strength.0;
                    statistics.damage_taken += strength.0;

                    // Floating damage number above the player.
                    crate::systems::damage_numbers::spawn_damage_number(
                        &mut commands,
                        player_world.0,
                        strength.0,
                        crate::systems::damage_numbers::DAMAGE_TO_PLAYER,
                    );

                    let push = (player_world.0 - enemy_world.0).normalize_or_zero();
                    if push != Vec2::ZERO {
                        player_knockback.0 += push * tuning::ENEMY_KNOCKBACK_TILES * scale;
                    }
                    commands.entity(player_entity).insert(HitFlash(
                        Timer::from_seconds(tuning::HIT_FLASH_DURATION, TimerMode::Once),
                    ));
                    spawn_particle(
                        &mut commands,
                        ParticleType::HitSpark,
                        Vec3::new(player_world.0.x, player_world.0.y, 0.06),
                    );
                    crate::systems::projectile::player_hurt_juice(&mut juice, &mut sfx);

                    // Thorns: reflect a fraction of the hit back at the attacker.
                    // (Death/gold bookkeeping is left to the cleanup + a separate
                    // reflected damage number; the attacker just loses HP here.)
                    if player_stats.thorns > 0.0 {
                        let reflected =
                            (strength.0 as f32 * player_stats.thorns).round().max(1.0) as i64;
                        enemy_health.0 -= reflected;
                        statistics.damage_dealt += reflected;
                        crate::systems::damage_numbers::spawn_damage_number(
                            &mut commands,
                            enemy_world.0,
                            reflected,
                            crate::systems::damage_numbers::DAMAGE_TO_ENEMY,
                        );
                    }

                    if player_health.0 <= 0 {
                        // Death particles at the player, then despawn (defeat
                        // system observes the missing player next frame).
                        spawn_particle(
                            &mut commands,
                            ParticleType::Death,
                            Vec3::new(player_world.0.x, player_world.0.y, 0.06),
                        );
                        commands.entity(player_entity).despawn();
                    }
                }
            }
        } else if awake.0 && in_range && attack.cooldown.is_finished() {
            // Begin telegraphing.
            attack.winding_up = true;
            attack.telegraph = Timer::from_seconds(tuning::ENEMY_TELEGRAPH, TimerMode::Once);
        } else {
            transform.scale = Vec3::ONE;
        }
    }
}
