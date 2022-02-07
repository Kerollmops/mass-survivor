use bevy::prelude::*;

#[derive(Default, Clone, Bundle)]
pub struct GameSpriteBundle {
    #[bundle]
    pub sprite: SpriteSheetBundle,
    pub base_rotation: BaseSpriteRotation,
    pub base_flip: BaseSpriteFlip,
}

#[derive(Default, Clone, Component)]
pub struct BaseSpriteRotation(pub f32); // radian

#[derive(Default, Clone, Component)]
pub struct BaseSpriteFlip {
    pub flip_x: bool,
    pub flip_y: bool,
}
