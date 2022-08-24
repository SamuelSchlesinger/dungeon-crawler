use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;

pub fn animate_sprites(
    mut query: Query<(
        &mut Transform,
        &mut TextureAtlasSprite,
        &Position,
        &ZLevel,
        &SpriteIndex,
    )>,
    scale_factor: Res<ScaleFactor>,
) {
    for (mut transform, mut sprite, pos, zlevel, sprite_index) in query.iter_mut() {
        *transform = Transform::from_xyz(
            (pos.x as f32 - 0.5) * scale_factor.0,
            (pos.y as f32 - 0.5) * scale_factor.0,
            zlevel.0,
        );
        sprite.index = sprite_index.0;
        sprite.custom_size = Some(Vec2::new(scale_factor.0, scale_factor.0));
    }
}
