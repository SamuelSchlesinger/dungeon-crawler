use bevy::prelude::*;

use crate::components::*;
use crate::tuning;

/// Updates all particles - moves them, applies a little drag, reduces lifetime,
/// fades them out over their OWN lifetime, and despawns dead ones (no leaks).
///
/// B0001: the particle query is the only `&mut` query in this system. Particles
/// carry no actor markers (`Position`/`Enemy`/`Player`), so they are excluded
/// from every collision/AI/combat query elsewhere -- they never alias an actor.
pub fn update_particles(
    mut commands: Commands,
    mut particle_query: Query<(Entity, &mut Transform, &mut Particle, &mut Sprite)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut transform, mut particle, mut sprite) in particle_query.iter_mut() {
        // Move + a gentle exponential drag so bursts "settle" instead of flying
        // off forever.
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;
        particle.velocity *= (1.0 - 3.0 * dt).max(0.0);

        particle.lifetime -= dt;

        // Fade out relative to the particle's own initial lifetime.
        let denom = particle.initial_lifetime.max(0.0001);
        let alpha = (particle.lifetime / denom).clamp(0.0, 1.0);
        sprite.color = sprite.color.with_alpha(alpha);

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Visual parameters for a particle burst: base color, size, lifetime, count,
/// and base speed range. Centralizes the per-type "feel" so `spawn_particle`
/// stays a thin dispatcher.
struct BurstSpec {
    color: Color,
    size: f32,
    lifetime: f32,
    count: usize,
    min_speed: f32,
    max_speed: f32,
}

/// Spawns a burst of particles at `position`. Reuses the single `Particle`
/// component + `update_particles` for fade/despawn so there are no leaks and
/// the particles are excluded from collision/AI queries (no actor markers).
pub fn spawn_particle(commands: &mut Commands, particle_type: ParticleType, position: Vec3) {
    let spec = match particle_type {
        ParticleType::HitSpark => BurstSpec {
            color: Color::srgb(1.0, 1.0, 0.55),
            size: 5.0,
            lifetime: 0.3,
            count: tuning::PARTICLE_HITSPARK_COUNT,
            min_speed: 60.0,
            max_speed: 150.0,
        },
        ParticleType::Death => BurstSpec {
            color: Color::srgb(0.7, 0.1, 0.1),
            size: 8.0,
            lifetime: 0.6,
            count: tuning::PARTICLE_DEATH_COUNT,
            min_speed: 50.0,
            max_speed: 160.0,
        },
        ParticleType::HealthPickup => BurstSpec {
            color: Color::srgb(0.2, 1.0, 0.3),
            size: 6.0,
            lifetime: 0.5,
            count: tuning::PARTICLE_PICKUP_COUNT,
            min_speed: 40.0,
            max_speed: 110.0,
        },
        ParticleType::GoldPickup => BurstSpec {
            color: Color::srgb(1.0, 0.82, 0.0),
            size: 5.0,
            lifetime: 0.5,
            count: tuning::PARTICLE_PICKUP_COUNT,
            min_speed: 40.0,
            max_speed: 120.0,
        },
        ParticleType::WeaponPickup => BurstSpec {
            color: Color::srgb(0.55, 0.8, 1.0),
            size: 6.0,
            lifetime: 0.55,
            count: tuning::PARTICLE_PICKUP_COUNT,
            min_speed: 40.0,
            max_speed: 120.0,
        },
        ParticleType::DashTrail => BurstSpec {
            color: Color::srgba(0.7, 0.85, 1.0, 0.8),
            size: 9.0,
            lifetime: tuning::PARTICLE_DASH_TRAIL_LIFETIME,
            count: tuning::PARTICLE_DASH_TRAIL_COUNT,
            // Slow, lingering puffs (an afterimage trail, not a spray).
            min_speed: 0.0,
            max_speed: 25.0,
        },
        ParticleType::BossDeath => BurstSpec {
            color: Color::srgb(1.0, 0.7, 0.2),
            size: 11.0,
            lifetime: 1.0,
            count: tuning::PARTICLE_BOSS_DEATH_COUNT,
            min_speed: 80.0,
            max_speed: 320.0,
        },
    };

    for _ in 0..spec.count {
        let angle = rand::random::<f32>() * std::f32::consts::TAU;
        let speed = spec.min_speed + rand::random::<f32>() * (spec.max_speed - spec.min_speed);
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
        // Small per-particle size jitter so bursts don't look uniform.
        let size = spec.size * (0.7 + rand::random::<f32>() * 0.6);

        commands.spawn((
            Sprite {
                color: spec.color,
                custom_size: Some(Vec2::new(size, size)),
                ..default()
            },
            Transform::from_translation(position),
            Particle {
                lifetime: spec.lifetime,
                velocity,
                initial_lifetime: spec.lifetime,
            },
            particle_type,
        ));
    }
}
