use std::f32::consts::PI;

use bevy::math::*;
use bevy::prelude::*;
use heron::prelude::*;
use rand::Rng;

use crate::assets::*;
use crate::game_sprites::*;
use crate::helper::*;
use crate::{GameLayer, Player, Velocity};

const TRACKING_SPEED: f32 = 0.03;
const TRACKING_MAX_SPEED: f32 = 1.0;

const SLOW_WALKING_SPEED: f32 = 0.02;
const SLOW_WALKING_MAX_SPEED: f32 = 1.0;

const RUNNING_SPEED: f32 = 0.05;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub enum EnemyKind {
    BlueFish,
    BigRedFish,
    Pumpkin,
    SkeletonHead,
    Knife,
}

#[derive(Component)]
pub struct EnemyWaveSize(pub usize);

#[derive(Component)]
pub struct EnemyWavesCount(pub usize);

#[derive(Bundle)]
pub struct EnemyWaveBundle {
    pub kind: EnemyKind,
    pub timer: Timer,
    pub size: EnemyWaveSize,
    pub count: EnemyWavesCount,
    pub movement_kind: MovementKind,
}

#[derive(Component)]
pub enum MovementKind {
    Tracking,
    SlowWalking,
    RunningGroup,
}

/// An enemy that follows the position of the player.
#[derive(Component)]
pub struct TrackingMovement;

/// An enemy that spawn far and walk towards the player, slowly.
#[derive(Component)]
pub struct SlowWalkingMovement;

/// An enemy group that spawn far and run to a fixed, previous, position of the player.
/// Find a new position to run to after having reached a far distance.
#[derive(Component)]
pub struct RunningGroupMovement {
    pub direction: Vec2,
}

#[derive(Bundle)]
pub struct EnemyBundle {
    #[bundle]
    pub game_sprite: GameSpriteBundle,
    pub velocity: Velocity,
    pub rigid_body: RigidBody,
    pub damping: Damping,
    pub physic_material: PhysicMaterial,
    pub collision_shape: CollisionShape,
    pub collision_layers: CollisionLayers,
    pub _marker: Enemy,
}

impl EnemyBundle {
    fn blue_fish(iconset_assets: &IconsetAssets, pos: Vec3) -> EnemyBundle {
        let game_sprite = GameSpriteBundle {
            sprite: SpriteSheetBundle {
                transform: Transform::from_translation(pos).with_scale(Vec3::splat(0.02)),
                sprite: TextureAtlasSprite::new(162),
                texture_atlas: iconset_assets.iconset_fantasy_standalone.clone(),
                ..Default::default()
            },
            base_rotation: BaseSpriteRotation(23.0 * PI / 14.0),
            ..Default::default()
        };

        EnemyBundle {
            game_sprite,
            velocity: Velocity::default(),
            rigid_body: RigidBody::Dynamic,
            damping: Damping::from_linear(0.5),
            physic_material: PhysicMaterial { density: 10.0, ..Default::default() },
            collision_shape: CollisionShape::Cuboid {
                half_extends: Vec3::new(0.3, 0.3, 0.),
                border_radius: None,
            },
            collision_layers: CollisionLayers::none().with_group(GameLayer::Enemies).with_masks(&[
                GameLayer::Enemies,
                GameLayer::Weapon,
                GameLayer::Player,
            ]),
            _marker: Enemy,
        }
    }

    fn big_red_fish(iconset_assets: &IconsetAssets, pos: Vec3) -> EnemyBundle {
        let game_sprite = GameSpriteBundle {
            sprite: SpriteSheetBundle {
                transform: Transform::from_translation(pos).with_scale(Vec3::splat(0.04)),
                sprite: TextureAtlasSprite::new(165),
                texture_atlas: iconset_assets.iconset_fantasy_standalone.clone(),
                ..Default::default()
            },
            base_rotation: BaseSpriteRotation(23.0 * PI / 14.0),
            ..Default::default()
        };

        EnemyBundle {
            game_sprite,
            velocity: Velocity::default(),
            rigid_body: RigidBody::Dynamic,
            damping: Damping::from_linear(0.5),
            physic_material: PhysicMaterial { density: 10.0, ..Default::default() },
            collision_shape: CollisionShape::Cuboid {
                half_extends: Vec3::new(0.6, 0.6, 0.),
                border_radius: None,
            },
            collision_layers: CollisionLayers::none().with_group(GameLayer::Enemies).with_masks(&[
                GameLayer::Enemies,
                GameLayer::Weapon,
                GameLayer::Player,
            ]),
            _marker: Enemy,
        }
    }

    fn pumpkin(iconset_assets: &IconsetAssets, pos: Vec3) -> EnemyBundle {
        let game_sprite = GameSpriteBundle {
            sprite: SpriteSheetBundle {
                transform: Transform::from_translation(pos).with_scale(Vec3::splat(0.02)),
                sprite: TextureAtlasSprite::new(0),
                texture_atlas: iconset_assets.iconset_halloween_standalone.clone(),
                ..Default::default()
            },
            base_rotation: BaseSpriteRotation(6.),
            ..Default::default()
        };

        EnemyBundle {
            game_sprite,
            velocity: Velocity::default(),
            rigid_body: RigidBody::Dynamic,
            damping: Damping::from_linear(0.5),
            physic_material: PhysicMaterial { density: 10.0, ..Default::default() },
            collision_shape: CollisionShape::Cuboid {
                half_extends: Vec3::new(0.3, 0.3, 0.),
                border_radius: None,
            },
            collision_layers: CollisionLayers::none().with_group(GameLayer::Enemies).with_masks(&[
                GameLayer::Enemies,
                GameLayer::Weapon,
                GameLayer::Player,
            ]),
            _marker: Enemy,
        }
    }

    fn skeleton_head(iconset_assets: &IconsetAssets, pos: Vec3) -> EnemyBundle {
        let game_sprite = GameSpriteBundle {
            sprite: SpriteSheetBundle {
                transform: Transform::from_translation(pos).with_scale(Vec3::splat(0.02)),
                sprite: TextureAtlasSprite::new(1),
                texture_atlas: iconset_assets.iconset_halloween_standalone.clone(),
                ..Default::default()
            },
            base_rotation: BaseSpriteRotation(6.),
            base_flip: BaseSpriteFlip { flip_x: true, ..Default::default() },
            ..Default::default()
        };

        EnemyBundle {
            game_sprite,
            velocity: Velocity::default(),
            rigid_body: RigidBody::Dynamic,
            damping: Damping::from_linear(0.5),
            physic_material: PhysicMaterial { density: 10.0, ..Default::default() },
            collision_shape: CollisionShape::Cuboid {
                half_extends: Vec3::new(0.3, 0.3, 0.),
                border_radius: None,
            },

            collision_layers: CollisionLayers::none().with_group(GameLayer::Enemies).with_masks(&[
                GameLayer::Enemies,
                GameLayer::Weapon,
                GameLayer::Player,
            ]),
            _marker: Enemy,
        }
    }

    fn knife(iconset_assets: &IconsetAssets, pos: Vec3) -> EnemyBundle {
        let game_sprite = GameSpriteBundle {
            sprite: SpriteSheetBundle {
                transform: Transform::from_translation(pos).with_scale(Vec3::splat(0.02)),
                sprite: TextureAtlasSprite::new(26),
                texture_atlas: iconset_assets.iconset_halloween_standalone.clone(),
                ..Default::default()
            },
            base_rotation: BaseSpriteRotation(23.0 * PI / 14.0),
            ..Default::default()
        };

        EnemyBundle {
            game_sprite,
            velocity: Velocity::default(),
            rigid_body: RigidBody::Dynamic,
            damping: Damping::from_linear(0.5),
            physic_material: PhysicMaterial { density: 10.0, ..Default::default() },
            collision_shape: CollisionShape::Cuboid {
                half_extends: Vec3::new(0.3, 0.3, 0.),
                border_radius: None,
            },

            collision_layers: CollisionLayers::none().with_group(GameLayer::Enemies).with_masks(&[
                GameLayer::Enemies,
                GameLayer::Weapon,
                GameLayer::Player,
            ]),
            _marker: Enemy,
        }
    }
}

pub fn spawn_enemy_waves(
    mut commands: Commands,
    time: Res<Time>,
    iconset_assets: Res<IconsetAssets>,
    mut enemy_waves_query: Query<(
        &mut Timer,
        &EnemyKind,
        &EnemyWaveSize,
        &EnemyWavesCount,
        &MovementKind,
    )>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player_transform = match player_query.iter().next() {
        Some(transform) => transform,
        None => return,
    };

    let mut rng = rand::thread_rng();
    for (mut timer, kind, size, count, movement_kind) in enemy_waves_query.iter_mut() {
        if timer.tick(time.delta()).just_finished() {
            // spawn enemy wave with enemy kind
            for _ in 0..count.0 {
                let origin = Vec2::new(rng.gen_range(-10.0..10.0), rng.gen_range(-10.0..10.0));
                let origin = move_from_deadzone(origin, 10.0);
                let offset = player_transform.translation + origin.extend(0.0);

                // TODO use spawn_batch for better performances
                for _ in 0..size.0 {
                    let pos = match movement_kind {
                        MovementKind::Tracking => {
                            random_in_radius(&mut rng, offset, 3.).extend(90.)
                        }
                        MovementKind::SlowWalking => {
                            let pos = random_in_radius(&mut rng, offset, 10.);
                            move_from_deadzone(pos, 3.).extend(90.)
                        }
                        MovementKind::RunningGroup => {
                            random_in_radius(&mut rng, offset, 10.).extend(90.)
                        }
                    };

                    let entity_id = match kind {
                        EnemyKind::BlueFish => {
                            commands.spawn_bundle(EnemyBundle::blue_fish(&iconset_assets, pos)).id()
                        }
                        EnemyKind::BigRedFish => commands
                            .spawn_bundle(EnemyBundle::big_red_fish(&iconset_assets, pos))
                            .id(),
                        EnemyKind::Pumpkin => {
                            commands.spawn_bundle(EnemyBundle::pumpkin(&iconset_assets, pos)).id()
                        }
                        EnemyKind::SkeletonHead => commands
                            .spawn_bundle(EnemyBundle::skeleton_head(&iconset_assets, pos))
                            .id(),
                        EnemyKind::Knife => {
                            commands.spawn_bundle(EnemyBundle::knife(&iconset_assets, pos)).id()
                        }
                    };

                    match movement_kind {
                        MovementKind::Tracking => {
                            commands.entity(entity_id).insert(TrackingMovement)
                        }
                        MovementKind::SlowWalking => {
                            commands.entity(entity_id).insert(SlowWalkingMovement)
                        }
                        MovementKind::RunningGroup => {
                            let direction =
                                (player_transform.translation.xy() - pos.xy()).normalize_or_zero();
                            commands.entity(entity_id).insert(RunningGroupMovement { direction })
                        }
                    };
                }
            }
        }
    }
}

pub fn tracking_movement(
    player_query: Query<&Transform, With<Player>>,
    mut ennemies_query: Query<
        (
            &mut Velocity,
            &mut Transform,
            &mut TextureAtlasSprite,
            &BaseSpriteRotation,
            &BaseSpriteFlip,
        ),
        (With<TrackingMovement>, Without<Player>),
    >,
) {
    let player_transform = match player_query.iter().next() {
        Some(transform) => transform,
        None => return,
    };

    for (mut velocity, mut transform, mut sprite, rotation, flip) in ennemies_query.iter_mut() {
        let player = player_transform.translation.xy();
        let enemy = transform.translation.xy();
        let direction = (player - enemy).normalize_or_zero();
        let strenght = player.distance(enemy).min(TRACKING_MAX_SPEED);
        velocity.linear += (direction * strenght * TRACKING_SPEED).extend(0.);

        let angle = angle_between(enemy, player);
        if enemy.x > player.x {
            sprite.flip_x = !flip.flip_x;
            transform.rotation = Quat::from_rotation_z(angle + PI - rotation.0);
        } else {
            sprite.flip_x = flip.flip_x;
            transform.rotation = Quat::from_rotation_z(angle + rotation.0);
        }
    }
}

pub fn slow_walking_movement(
    player_query: Query<&Transform, With<Player>>,
    mut ennemies_query: Query<
        (
            &mut Velocity,
            &mut Transform,
            &mut TextureAtlasSprite,
            &BaseSpriteRotation,
            &BaseSpriteFlip,
        ),
        (With<SlowWalkingMovement>, Without<Player>),
    >,
) {
    let player_transform = match player_query.iter().next() {
        Some(transform) => transform,
        None => return,
    };

    for (mut velocity, mut transform, mut sprite, rotation, flip) in ennemies_query.iter_mut() {
        let player = player_transform.translation.xy();
        let enemy = transform.translation.xy();
        let direction = (player - enemy).normalize_or_zero();
        let strenght = player.distance(enemy).min(SLOW_WALKING_MAX_SPEED);
        velocity.linear += (direction * strenght * SLOW_WALKING_SPEED).extend(0.);

        let angle = angle_between(enemy, player);
        if enemy.x > player.x {
            sprite.flip_x = !flip.flip_x;
            transform.rotation = Quat::from_rotation_z(angle + PI - rotation.0);
        } else {
            sprite.flip_x = flip.flip_x;
            transform.rotation = Quat::from_rotation_z(angle + rotation.0);
        }
    }
}

pub fn running_group_movement(
    player_query: Query<&Transform, With<Player>>,
    mut ennemies_query: Query<
        (
            &mut Velocity,
            &mut Transform,
            &mut TextureAtlasSprite,
            &mut RunningGroupMovement,
            &BaseSpriteRotation,
            &BaseSpriteFlip,
        ),
        Without<Player>,
    >,
) {
    let player_transform = match player_query.iter().next() {
        Some(transform) => transform,
        None => return,
    };

    for (mut velocity, mut transform, mut sprite, mut movement, rotation, flip) in
        ennemies_query.iter_mut()
    {
        let player = player_transform.translation.xy();
        let enemy = transform.translation.xy();
        let distance = player.distance(enemy);
        if distance > 20.0 {
            let pos = transform.translation;
            movement.direction = (player_transform.translation.xy() - pos.xy()).normalize_or_zero();
        }

        let direction = movement.direction;
        velocity.linear += (direction * RUNNING_SPEED).extend(0.);

        let angle = angle_between(enemy, player);
        if enemy.x > player.x {
            sprite.flip_x = !flip.flip_x;
            transform.rotation = Quat::from_rotation_z(angle + PI - rotation.0);
        } else {
            sprite.flip_x = flip.flip_x;
            transform.rotation = Quat::from_rotation_z(angle + rotation.0);
        }
    }
}
