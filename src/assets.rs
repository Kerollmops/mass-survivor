#![allow(dead_code)]

use std::time::Duration;

use benimator::SpriteSheetAnimation;
use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;
use rand::Rng;

#[derive(AssetCollection)]
pub struct GameAssets {
    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 16., columns = 25, rows = 10))]
    #[asset(path = "images/MagicElementals.png")]
    pub magic_elementals: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 16., columns = 24, rows = 16))]
    #[asset(path = "images/Necromancers.png")]
    pub necromancers: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 16., columns = 7, rows = 2))]
    #[asset(path = "images/NecromancerIcons.png")]
    pub necromancer_icons: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 16., columns = 20, rows = 16))]
    #[asset(path = "images/Infernos.png")]
    pub infernos: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 16., columns = 7, rows = 2))]
    #[asset(path = "images/InfernosIcons.png")]
    pub infernos_icons: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 16., columns = 20, rows = 16))]
    #[asset(path = "images/Castle.png")]
    pub castle: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 16., columns = 7, rows = 2))]
    #[asset(path = "images/CastleIcons.png")]
    pub castle_icons: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 144., tile_size_y = 144., columns = 5, rows = 4))]
    #[asset(path = "images/SmokeEffect07.png")]
    pub smoke_effect_07: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 16., columns = 8, rows = 4))]
    #[asset(path = "images/PinkSelector01.png")]
    pub pink_selector_01: Handle<TextureAtlas>,
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

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Necromancer {
    MaleSkeleton = 1,
    MaleZombie,
    MaleSpider,
    MaleGhost,
    MaleVampire,
    MaleLich,
    MaleDeathKnight,
    FemaleSkeleton = 9,
    FemaleZombie,
    FemaleSpider,
    FemaleGhost,
    FemaleVampire,
    FemaleLich,
    FemaleDeathKnight,
}

impl Necromancer {
    pub fn from_rng<R: Rng>(rng: &mut R) -> Necromancer {
        match rng.gen_range(0..14) {
            0 => Necromancer::MaleSkeleton,
            1 => Necromancer::MaleZombie,
            2 => Necromancer::MaleSpider,
            3 => Necromancer::MaleGhost,
            4 => Necromancer::MaleVampire,
            5 => Necromancer::MaleLich,
            6 => Necromancer::MaleDeathKnight,
            7 => Necromancer::FemaleSkeleton,
            8 => Necromancer::FemaleZombie,
            9 => Necromancer::FemaleSpider,
            10 => Necromancer::FemaleGhost,
            11 => Necromancer::FemaleVampire,
            12 => Necromancer::FemaleLich,
            _ => Necromancer::FemaleDeathKnight,
        }
    }

    pub fn idle_animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 24;
        SpriteSheetAnimation::from_range(index..=index + 3, Duration::from_secs_f64(1.0 / 6.0))
    }

    pub fn walk_animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 24 + 4;
        SpriteSheetAnimation::from_range(index..=index + 3, Duration::from_secs_f64(1.0 / 6.0))
    }

    pub fn attack_animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 24 + 8;
        SpriteSheetAnimation::from_range(index..=index + 3, Duration::from_secs_f64(1.0 / 6.0))
    }

    pub fn hit_animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 24 + 12;
        SpriteSheetAnimation::from_range(index..=index + 3, Duration::from_secs_f64(1.0 / 6.0))
    }

    pub fn death_animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 24 + 16;
        SpriteSheetAnimation::from_range(index..=index + 3, Duration::from_secs_f64(1.0 / 6.0))
    }

    pub fn special_animation(&self) -> Option<SpriteSheetAnimation> {
        use self::Necromancer::*;
        if matches!(self, MaleVampire | MaleLich | FemaleVampire | FemaleLich) {
            let index = *self as usize * 24 + 20;
            Some(SpriteSheetAnimation::from_range(
                index..=index + 3,
                Duration::from_secs_f64(1.0 / 6.0),
            ))
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Inferno {
    MaleImp = 1,
    MaleGog,
    MaleHellHound,
    MaleDemon,
    MalePitField,
    MaleEfreet,
    MaleDevil,
    FemaleImp = 9,
    FemaleGog,
    FemaleHellHound,
    FemaleDemon,
    FemalePitField,
    FemaleEfreet,
    FemaleDevil,
}

impl Inferno {
    pub fn from_rng<R: Rng>(rng: &mut R) -> Inferno {
        match rng.gen_range(0..14) {
            0 => Inferno::MaleImp,
            1 => Inferno::MaleGog,
            2 => Inferno::MaleHellHound,
            3 => Inferno::MaleDemon,
            4 => Inferno::MalePitField,
            5 => Inferno::MaleEfreet,
            6 => Inferno::MaleDevil,
            7 => Inferno::FemaleImp,
            8 => Inferno::FemaleGog,
            9 => Inferno::FemaleHellHound,
            10 => Inferno::FemaleDemon,
            11 => Inferno::FemalePitField,
            12 => Inferno::FemaleEfreet,
            _ => Inferno::FemaleDevil,
        }
    }

    pub fn idle_animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 20;
        SpriteSheetAnimation::from_range(index..=index + 3, Duration::from_secs_f64(1.0 / 6.0))
    }

    pub fn walk_animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 20 + 4;
        SpriteSheetAnimation::from_range(index..=index + 3, Duration::from_secs_f64(1.0 / 6.0))
    }

    pub fn attack_animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 20 + 8;
        SpriteSheetAnimation::from_range(index..=index + 3, Duration::from_secs_f64(1.0 / 6.0))
    }

    pub fn hit_animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 20 + 12;
        SpriteSheetAnimation::from_range(index..=index + 3, Duration::from_secs_f64(1.0 / 6.0))
    }

    pub fn death_animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 20 + 16;
        SpriteSheetAnimation::from_range(index..=index + 3, Duration::from_secs_f64(1.0 / 6.0))
    }
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Castle {
    MalePikeman = 1,
    MaleArcher,
    MaleGriffin,
    MaleSwordsman,
    SimpleMonk,
    MaleCavalier,
    MalePaladin,
    FemalePikeman = 9,
    FemaleArcher,
    FemaleGriffin,
    FemaleSwordsman,
    SuperMonk,
    FemaleCavalier,
    FemalePaladin,
}

impl Castle {
    pub fn from_rng<R: Rng>(rng: &mut R) -> Castle {
        match rng.gen_range(0..14) {
            0 => Castle::MalePikeman,
            1 => Castle::MaleArcher,
            2 => Castle::MaleGriffin,
            3 => Castle::MaleSwordsman,
            4 => Castle::SimpleMonk,
            5 => Castle::MaleCavalier,
            6 => Castle::MalePaladin,
            7 => Castle::FemalePikeman,
            8 => Castle::FemaleArcher,
            9 => Castle::FemaleGriffin,
            10 => Castle::FemaleSwordsman,
            11 => Castle::SuperMonk,
            12 => Castle::FemaleCavalier,
            _ => Castle::FemalePaladin,
        }
    }

    pub fn idle_animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 20;
        SpriteSheetAnimation::from_range(index..=index + 3, Duration::from_secs_f64(1.0 / 6.0))
    }

    pub fn walk_animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 20 + 4;
        SpriteSheetAnimation::from_range(index..=index + 3, Duration::from_secs_f64(1.0 / 6.0))
    }

    pub fn attack_animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 20 + 8;
        SpriteSheetAnimation::from_range(index..=index + 3, Duration::from_secs_f64(1.0 / 6.0))
    }

    pub fn hit_animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 20 + 12;
        SpriteSheetAnimation::from_range(index..=index + 3, Duration::from_secs_f64(1.0 / 6.0))
    }

    pub fn death_animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 20 + 16;
        SpriteSheetAnimation::from_range(index..=index + 3, Duration::from_secs_f64(1.0 / 6.0))
    }
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum PinkSelector {
    Full = 0,
    TwoThird,
    OneThird,
    Empty,
}

impl PinkSelector {
    pub fn animation(&self) -> SpriteSheetAnimation {
        let index = *self as usize * 8;
        SpriteSheetAnimation::from_range(index..=index + 7, Duration::from_secs_f64(1.0 / 12.0))
    }
}
