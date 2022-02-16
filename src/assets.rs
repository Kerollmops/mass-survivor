use std::time::Duration;

use benimator::SpriteSheetAnimation;
use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;
use rand::Rng;

#[derive(AssetCollection)]
pub struct AnimatedAssets {
    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 16., columns = 25, rows = 10,))]
    #[asset(path = "images/MagicElementals.png")]
    pub magic_elementals: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 16., columns = 4, rows = 37,))]
    #[asset(path = "images/Necromancers.png")]
    pub necromancers: Handle<TextureAtlas>,
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Elemental {
    Air = 1,
    Storm,
    Earth,
    Fire,
    Magma,
    Water,
    Ice,
    Magic,
    Psychic,
}

impl Elemental {
    pub fn from_rng<R: Rng>(rng: &mut R) -> Elemental {
        match rng.gen_range(0..9) {
            0 => Elemental::Air,
            1 => Elemental::Storm,
            2 => Elemental::Earth,
            3 => Elemental::Fire,
            4 => Elemental::Magma,
            5 => Elemental::Water,
            6 => Elemental::Ice,
            7 => Elemental::Magic,
            _ => Elemental::Psychic,
        }
    }

    pub fn head_sprite_index(&self) -> usize {
        *self as usize * 25
    }

    pub fn idle_animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 25 + 1;
        SpriteSheetAnimation::from_range(index..=index + 3, Duration::from_secs_f64(1.0 / 6.0))
    }

    pub fn walk_animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 25 + 1 + 4;
        SpriteSheetAnimation::from_range(index..=index + 3, Duration::from_secs_f64(1.0 / 6.0))
    }

    pub fn attack_animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 25 + 1 + 8;
        SpriteSheetAnimation::from_range(index..=index + 3, Duration::from_secs_f64(1.0 / 6.0))
    }

    pub fn hit_animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 25 + 1 + 12;
        SpriteSheetAnimation::from_range(index..=index + 3, Duration::from_secs_f64(1.0 / 6.0))
    }

    pub fn death_animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 25 + 1 + 16;
        SpriteSheetAnimation::from_range(index..=index + 3, Duration::from_secs_f64(1.0 / 6.0))
    }

    pub fn special_animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 25 + 1 + 20;
        SpriteSheetAnimation::from_range(index..=index + 3, Duration::from_secs_f64(1.0 / 6.0))
    }
}
