use bevy::prelude::*;

use crate::components::*;

/// Updates all particles - moves them, reduces lifetime, despawns dead ones
pub fn update_particles(
    mut commands: Commands,
    mut particle_query: Query<(Entity, &mut Transform, &mut Particle, &mut Sprite)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut particle, mut sprite) in particle_query.iter_mut() {
        // Update position based on velocity
        transform.translation.x += particle.velocity.x * time.delta_secs();
        transform.translation.y += particle.velocity.y * time.delta_secs();

        // Reduce lifetime
        particle.lifetime -= time.delta_secs();

        // Fade out sprite
        let alpha = (particle.lifetime / 0.5).clamp(0.0, 1.0);
        sprite.color = sprite.color.with_alpha(alpha);

        // Despawn when lifetime expires
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Spawns particles at the specified location
pub fn spawn_particle(
    commands: &mut Commands,
    particle_type: ParticleType,
    position: Vec3,
) {
    let (color, size, lifetime, count) = match particle_type {
        ParticleType::HitSpark => (Color::srgb(1.0, 1.0, 0.0), 5.0, 0.3, 3),
        ParticleType::Death => (Color::srgb(0.5, 0.0, 0.0), 8.0, 0.6, 8),
        ParticleType::HealthPickup => (Color::srgb(0.0, 1.0, 0.0), 6.0, 0.4, 5),
    };

    for _ in 0..count {
        let angle = rand::random::<f32>() * std::f32::consts::TAU;
        let speed = 50.0 + rand::random::<f32>() * 50.0;
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(size, size)),
                ..default()
            },
            Transform::from_translation(position),
            Particle { lifetime, velocity },
            particle_type,
        ));
    }
}
