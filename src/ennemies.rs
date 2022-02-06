use std::f32::consts::PI;

use bevy::math::*;
use bevy::prelude::*;
use impacted::CollisionShape;
use rand::Rng;

use crate::assets::*;
use crate::game_sprites::*;
use crate::helper::*;
use crate::{Player, Velocity};

const TRACKING_SPEED: f32 = 0.1;
const TRACKING_MAX_SPEED: f32 = 5.0;

const SLOW_WALKING_SPEED: f32 = 0.03;
const SLOW_WALKING_MAX_SPEED: f32 = 2.0;

const RUNNING_SPEED: f32 = 0.5;

#[derive(Component)]
pub struct Ennemy;

#[derive(Component)]
pub enum EnnemyKind {
    BlueFish,
    BigRedFish,
    Pumpkin,
    SkeletonHead,
    Knife,
}

#[derive(Component)]
pub struct EnnemyWaveSize(pub usize);

#[derive(Component)]
pub struct EnnemyWavesCount(pub usize);

#[derive(Bundle)]
pub struct EnnemyWaveBundle {
    pub kind: EnnemyKind,
    pub timer: Timer,
    pub size: EnnemyWaveSize,
    pub count: EnnemyWavesCount,
    pub movement_kind: MovementKind,
}

#[derive(Component)]
pub enum MovementKind {
    Tracking,
    SlowWalking,
    Running,
}

/// An ennemy that follows the position of the player.
#[derive(Component)]
pub struct TrackingMovement;

/// An ennemy that spawn far and walk towards the player, slowly.
#[derive(Component)]
pub struct SlowWalkingMovement;

/// An ennemy that spawn far and run to a fixed (previously position of the player.
/// Find a new position to run to after having reached a far distance.
#[derive(Component)]
pub struct RunningMovement {
    pub direction: Vec2,
}

pub fn spawn_ennemy_waves(
    mut commands: Commands,
    time: Res<Time>,
    iconset_assets: Res<IconsetAssets>,
    mut ennemy_waves_query: Query<(
        &mut Timer,
        &EnnemyKind,
        &EnnemyWaveSize,
        &EnnemyWavesCount,
        &MovementKind,
    )>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player_transform = match player_query.iter().next() {
        Some(transform) => transform,
        None => return,
    };

    let mut rng = rand::thread_rng();
    for (mut timer, kind, size, count, movement_kind) in ennemy_waves_query.iter_mut() {
        if timer.tick(time.delta()).just_finished() {
            // spawn ennemy wave with ennemy kind
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
                        MovementKind::Running => {
                            random_in_radius(&mut rng, offset, 10.).extend(90.)
                        }
                    };

                    let entity_id = match kind {
                        EnnemyKind::BlueFish => commands
                            .spawn_bundle(GameSpriteBundle {
                                transform: Transform::from_translation(pos)
                                    .with_scale(Vec3::splat(0.02)),
                                sprite: TextureAtlasSprite::new(162),
                                texture_atlas: iconset_assets.iconset_fantasy_standalone.clone(),
                                base_rotation: BaseSpriteRotation(23.0 * PI / 14.0),
                                ..Default::default()
                            })
                            .insert(Velocity::default())
                            .insert(CollisionShape::new_rectangle(0.04, 0.04))
                            .insert(Ennemy)
                            .id(),
                        EnnemyKind::BigRedFish => commands
                            .spawn_bundle(GameSpriteBundle {
                                transform: Transform::from_translation(pos)
                                    .with_scale(Vec3::splat(0.04)),
                                sprite: TextureAtlasSprite::new(165),
                                texture_atlas: iconset_assets.iconset_fantasy_standalone.clone(),
                                base_rotation: BaseSpriteRotation(23.0 * PI / 14.0),
                                ..Default::default()
                            })
                            .insert(Velocity::default())
                            .insert(CollisionShape::new_rectangle(0.08, 0.08))
                            .insert(Ennemy)
                            .id(),
                        EnnemyKind::Pumpkin => commands
                            .spawn_bundle(GameSpriteBundle {
                                transform: Transform::from_translation(pos)
                                    .with_scale(Vec3::splat(0.02)),
                                sprite: TextureAtlasSprite::new(0),
                                texture_atlas: iconset_assets.iconset_halloween_standalone.clone(),
                                base_rotation: BaseSpriteRotation(6.),
                                ..Default::default()
                            })
                            .insert(Velocity::default())
                            .insert(CollisionShape::new_rectangle(0.04, 0.04))
                            .insert(Ennemy)
                            .id(),
                        EnnemyKind::SkeletonHead => commands
                            .spawn_bundle(GameSpriteBundle {
                                transform: Transform::from_translation(pos)
                                    .with_scale(Vec3::splat(0.02)),
                                sprite: TextureAtlasSprite::new(1),
                                texture_atlas: iconset_assets.iconset_halloween_standalone.clone(),
                                base_rotation: BaseSpriteRotation(6.),
                                base_flip: BaseSpriteFlip { flip_x: true, ..Default::default() },
                                ..Default::default()
                            })
                            .insert(Velocity::default())
                            .insert(CollisionShape::new_rectangle(0.04, 0.04))
                            .insert(Ennemy)
                            .id(),
                        EnnemyKind::Knife => commands
                            .spawn_bundle(GameSpriteBundle {
                                transform: Transform::from_translation(pos)
                                    .with_scale(Vec3::splat(0.02)),
                                sprite: TextureAtlasSprite::new(26),
                                texture_atlas: iconset_assets.iconset_halloween_standalone.clone(),
                                base_rotation: BaseSpriteRotation(23.0 * PI / 14.0),
                                ..Default::default()
                            })
                            .insert(Velocity::default())
                            .insert(CollisionShape::new_rectangle(0.03, 0.03))
                            .insert(Ennemy)
                            .id(),
                    };

                    match movement_kind {
                        MovementKind::Tracking => {
                            commands.entity(entity_id).insert(TrackingMovement)
                        }
                        MovementKind::SlowWalking => {
                            commands.entity(entity_id).insert(SlowWalkingMovement)
                        }
                        MovementKind::Running => {
                            let direction =
                                (player_transform.translation.xy() - pos.xy()).normalize_or_zero();
                            commands.entity(entity_id).insert(RunningMovement { direction })
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
        let ennemy = transform.translation.xy();
        let direction = (player - ennemy).normalize_or_zero();
        let strenght = player.distance(ennemy).min(TRACKING_MAX_SPEED);
        velocity.0 += direction * strenght * TRACKING_SPEED;

        let angle = angle_between(ennemy, player);
        if ennemy.x > player.x {
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
        let ennemy = transform.translation.xy();
        let direction = (player - ennemy).normalize_or_zero();
        let strenght = player.distance(ennemy).min(SLOW_WALKING_MAX_SPEED);
        velocity.0 += direction * strenght * SLOW_WALKING_SPEED;

        let angle = angle_between(ennemy, player);
        if ennemy.x > player.x {
            sprite.flip_x = !flip.flip_x;
            transform.rotation = Quat::from_rotation_z(angle + PI - rotation.0);
        } else {
            sprite.flip_x = flip.flip_x;
            transform.rotation = Quat::from_rotation_z(angle + rotation.0);
        }
    }
}

pub fn running_movement(
    player_query: Query<&Transform, With<Player>>,
    mut ennemies_query: Query<
        (
            &mut Velocity,
            &mut Transform,
            &mut TextureAtlasSprite,
            &mut RunningMovement,
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
        let ennemy = transform.translation.xy();
        let distance = player.distance(ennemy);
        if distance > 20.0 {
            let pos = transform.translation;
            movement.direction = (player_transform.translation.xy() - pos.xy()).normalize_or_zero();
        }

        let direction = movement.direction;
        velocity.0 += direction * RUNNING_SPEED;

        let angle = angle_between(ennemy, player);
        if ennemy.x > player.x {
            sprite.flip_x = !flip.flip_x;
            transform.rotation = Quat::from_rotation_z(angle + PI - rotation.0);
        } else {
            sprite.flip_x = flip.flip_x;
            transform.rotation = Quat::from_rotation_z(angle + rotation.0);
        }
    }
}

pub fn ennemies_repulsion(mut ennemies_query: Query<(&mut Velocity, &Transform, &Ennemy)>) {
    let mut combinations = ennemies_query.iter_combinations_mut();
    while let Some([(mut avel, atransf, _), (_, btransf, _)]) = combinations.fetch_next() {
        let dist = atransf.translation.xy().distance(btransf.translation.xy());
        if dist <= 2.0 {
            let strenght = 2.0 - dist;
            let dir = (btransf.translation.xy() - atransf.translation.xy()).normalize_or_zero();
            avel.0 -= dir * (strenght / 50.0);
        }
    }
}
