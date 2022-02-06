use std::f32::consts::PI;
use std::mem;

use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::transform::TransformSystem;
use bevy_asset_loader::AssetLoader;
use impacted::CollisionShape;
use rand::Rng;

use self::assets::*;
use self::game_sprites::*;
use self::helper::*;

mod assets;
mod game_sprites;
mod helper;

const MAP_SIZE: u32 = 41;
const GRID_WIDTH: f32 = 0.05;
const SLOW_DOWN: f32 = 4.0;
const PLAYER_SPEED: f32 = 0.5;
const FISH_SPEED: f32 = 0.1;
const FISH_MAX_SPEED: f32 = 5.0;

const HEALTHY_PLAYER_COLOR: Color = Color::rgb(0., 0.47, 1.);
const HIT_PLAYER_COLOR: Color = Color::rgb(0.9, 0.027, 0.);

const AXE_HEAD_COLOR: Color = Color::rgb(0.52, 0.62, 0.8);
const AXE_HEAD_SPEED: f32 = 2.; // radian/s

fn main() {
    let mut app = App::new();
    AssetLoader::new(MyStates::AssetLoading)
        .continue_to_state(MyStates::Next)
        .with_collection::<IconsetAssets>()
        .build(&mut app);

    app.add_state(MyStates::AssetLoading)
        .insert_resource(ClearColor(Color::rgb(0.53, 0.53, 0.53)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_startup_system(spawn_player)
        // Player movement and camera tracking
        .add_system_to_stage(CoreStage::PostUpdate, move_player.chain(camera_follow))
        // Ennemies spawning
        .add_system_set(
            SystemSet::on_update(MyStates::Next)
                .with_system(spawn_ennemies)
                .with_system(create_loot) // Put the loot on the floor
                .with_system(player_loot_stuff)
                .with_system(axe_head_touch_ennemies), // Kill ennemies touched by axe head and spawn gems
        )
        // Apply the velocity to the transforms
        .add_system_to_stage(CoreStage::PostUpdate, move_ennemies.chain(apply_velocity))
        // Collision detection
        .add_system_to_stage(
            CoreStage::PostUpdate,
            update_shape_transforms // First update transforms
                .chain(rotate_axe_head) // Rotate the axe around the player
                .chain(change_player_color) // Change the colors
                .chain(player_loot_gems) // Remove and increment player XP
                .chain(ennemies_repulsion) // Repulse ennemies
                .chain(gems_player_attraction) // Player attract gems in vicinity
                .after(TransformSystem::TransformPropagate), // Better to consider the up-to-date transforms
        )
        .run();
}

fn setup(mut commands: Commands) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.scale = 1. / 50.;
    commands.spawn_bundle(camera_bundle);
    commands.insert_resource(EnnemyWaves::default());
    commands.insert_resource(LootAllGemsFor(Timer::from_seconds(0., false)));

    // Horizontal lines
    for i in 0..=MAP_SIZE {
        commands.spawn_bundle(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(
                0.,
                i as f32 - MAP_SIZE as f32 / 2.,
                10.,
            )),
            sprite: Sprite {
                color: Color::rgb(0.27, 0.27, 0.27),
                custom_size: Some(Vec2::new(MAP_SIZE as f32, GRID_WIDTH)),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    // Vertical lines
    for i in 0..=MAP_SIZE {
        commands.spawn_bundle(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(
                i as f32 - MAP_SIZE as f32 / 2.,
                0.,
                10.,
            )),
            sprite: Sprite {
                color: Color::rgb(0.27, 0.27, 0.27),
                custom_size: Some(Vec2::new(GRID_WIDTH, MAP_SIZE as f32)),
                ..Default::default()
            },
            ..Default::default()
        });
    }
}

fn spawn_player(mut commands: Commands) {
    let player_pos = Vec3::new(0., 0., 100.);
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_translation(player_pos),
            sprite: Sprite {
                color: HEALTHY_PLAYER_COLOR,
                custom_size: Some(Vec2::new(1., 1.)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Velocity::default())
        .insert(CollisionShape::new_rectangle(1.3, 1.3))
        .insert(Player::default());

    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_translation(player_pos + Vec3::new(0., -3., 0.)),
            sprite: Sprite {
                color: AXE_HEAD_COLOR,
                custom_size: Some(Vec2::new(0.8, 0.8)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(CollisionShape::new_circle(0.7))
        .insert(RotationRadian(0.))
        .insert(AxeHead);
}

fn move_player(keys: Res<Input<KeyCode>>, mut player_query: Query<&mut Velocity, With<Player>>) {
    for mut velocity in player_query.iter_mut() {
        let mut direction = Vec2::ZERO;

        if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
            direction.y += 1.;
        }
        if keys.any_pressed([KeyCode::Down, KeyCode::S]) {
            direction.y -= 1.;
        }
        if keys.any_pressed([KeyCode::Right, KeyCode::D]) {
            direction.x += 1.;
        }
        if keys.any_pressed([KeyCode::Left, KeyCode::A]) {
            direction.x -= 1.;
        }

        if direction == Vec2::ZERO {
            continue;
        }

        velocity.0 += direction * PLAYER_SPEED;
    }
}

fn apply_velocity(time: Res<Time>, mut transform_query: Query<(&mut Transform, &mut Velocity)>) {
    for (mut transform, mut velocity) in transform_query.iter_mut() {
        transform.translation += velocity.0.extend(0.0) * time.delta_seconds();
        velocity.0 /= Vec2::ONE + (time.delta_seconds() * SLOW_DOWN);
    }
}

/// Update the `CollisionShape` transform if the `GlobalTransform` has changed
fn update_shape_transforms(
    mut shapes: Query<(&mut CollisionShape, &GlobalTransform), Changed<GlobalTransform>>,
) {
    for (mut shape, transform) in shapes.iter_mut() {
        shape.set_transform(*transform);
    }
}

fn rotate_axe_head(
    time: Res<Time>,
    mut player_query: Query<&Transform, With<Player>>,
    mut axe_head_query: Query<
        (&mut Transform, &mut RotationRadian),
        (With<AxeHead>, Without<Player>),
    >,
) {
    let player_transform = match player_query.iter_mut().next() {
        Some(transform) => transform,
        None => return,
    };

    let (mut transform, mut rotation) = match axe_head_query.iter_mut().next() {
        Some(transform) => transform,
        None => return,
    };

    let radian = rotation.0 + AXE_HEAD_SPEED * time.delta_seconds();
    rotation.0 = if radian >= 2. * PI { 0. } else { radian };

    let x = rotation.0.cos() * 3.;
    let y = rotation.0.sin() * 3.;
    transform.translation = player_transform.translation + Vec3::new(x, y, 0.);
}

fn change_player_color(
    mut player_query: Query<(&mut Sprite, &CollisionShape), With<Player>>,
    ennemies_query: Query<&CollisionShape, With<Ennemy>>,
) {
    let (mut player_sprite, player_shape) = match player_query.iter_mut().next() {
        Some(value) => value,
        None => return,
    };

    if ennemies_query.iter().any(|shape| shape.is_collided_with(player_shape)) {
        player_sprite.color = HIT_PLAYER_COLOR;
    } else {
        player_sprite.color = HEALTHY_PLAYER_COLOR;
    }
}

fn player_loot_gems(
    mut commands: Commands,
    mut player_query: Query<(&mut Player, &CollisionShape)>,
    gems_query: Query<(Entity, &CollisionShape), With<Gem>>,
) {
    let (mut player, player_shape) = match player_query.iter_mut().next() {
        Some(value) => value,
        None => return,
    };

    for (gem_entity, shape) in gems_query.iter() {
        if shape.is_collided_with(player_shape) {
            player.xp += 1;
            commands.entity(gem_entity).despawn();
        }
    }
}

fn create_loot(
    mut commands: Commands,
    iconset_assets: Res<IconsetAssets>,
    mut player_query: Query<(&mut Player, &Transform)>,
) {
    let (mut player, transform) = match player_query.iter_mut().next() {
        Some(value) => value,
        None => return,
    };

    let mut rng = rand::thread_rng();
    if !player.lvl1_stuff_generated && player.xp >= 30 {
        player.lvl1_stuff_generated = true;
        let pos = random_in_radius(&mut rng, transform.translation, 5.);
        let pos = move_from_deadzone(pos, 3.).extend(95.);

        commands
            .spawn_bundle(SpriteSheetBundle {
                transform: Transform::from_translation(pos).with_scale(Vec3::splat(0.04)),
                sprite: TextureAtlasSprite::new(864),
                texture_atlas: iconset_assets.iconset_fantasy_castshadows.clone(),
                ..Default::default()
            })
            .insert(CollisionShape::new_rectangle(2., 2.))
            .insert(Stuff::FishingRod);
    }
}

fn player_loot_stuff(
    mut commands: Commands,
    mut loot_all_gems: ResMut<LootAllGemsFor>,
    mut player_query: Query<(&mut Player, &CollisionShape)>,
    stuff_query: Query<(Entity, &Stuff, &CollisionShape)>,
) {
    let (_player, player_shape) = match player_query.iter_mut().next() {
        Some(value) => value,
        None => return,
    };

    for (stuff_entity, stuff, shape) in stuff_query.iter() {
        if shape.is_collided_with(player_shape) {
            match stuff {
                Stuff::FishingRod => {
                    *loot_all_gems = LootAllGemsFor(Timer::from_seconds(2., false))
                }
            }
            commands.entity(stuff_entity).despawn();
        }
    }
}

fn axe_head_touch_ennemies(
    mut commands: Commands,
    iconset_assets: Res<IconsetAssets>,
    axe_head_query: Query<&CollisionShape, With<AxeHead>>,
    ennemies_query: Query<(Entity, &Transform, &CollisionShape), With<Ennemy>>,
) {
    let axe_head_shape = match axe_head_query.iter().next() {
        Some(shape) => shape,
        None => return,
    };

    for (entity, transform, shape) in ennemies_query.iter() {
        if shape.is_collided_with(axe_head_shape) {
            let pos = transform.translation.xy().extend(80.0);
            commands.entity(entity).despawn();
            commands
                .spawn_bundle(SpriteSheetBundle {
                    transform: Transform::from_translation(pos).with_scale(Vec3::splat(0.015)),
                    sprite: TextureAtlasSprite::new(474),
                    texture_atlas: iconset_assets.iconset_fantasy_standalone.clone(),
                    ..Default::default()
                })
                .insert(CollisionShape::new_rectangle(0.04, 0.04))
                .insert(Velocity::default())
                .insert(MoveToPlayer::default())
                .insert(Gem);
        }
    }
}

fn spawn_ennemies(
    time: Res<Time>,
    iconset_assets: Res<IconsetAssets>,
    mut ennemy_wave_spawned: ResMut<EnnemyWaves>,
    player_query: Query<&Transform, With<Player>>,
    mut commands: Commands,
) {
    match time.time_since_startup().as_secs() {
        2 if !ennemy_wave_spawned.spawn(0) => {
            let mut rng = rand::thread_rng();
            for player_transform in player_query.iter() {
                let origin = Vec2::new(rng.gen_range(-10.0..10.0), rng.gen_range(-10.0..10.0));
                let origin = move_from_deadzone(origin, 10.0);
                let offset = player_transform.translation + origin.extend(0.0);

                for _ in 0..40 {
                    let pos = random_in_radius(&mut rng, offset, 3.).extend(90.);

                    commands
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
                        .insert(Ennemy);
                }
            }
        }
        15 if !ennemy_wave_spawned.spawn(1) => {
            let mut rng = rand::thread_rng();
            for player_transform in player_query.iter() {
                for _ in 0..2 {
                    let origin = Vec2::new(rng.gen_range(-10.0..10.0), rng.gen_range(-10.0..10.0));
                    let origin = move_from_deadzone(origin, 10.0);
                    let offset = player_transform.translation + origin.extend(0.0);

                    for _ in 0..60 {
                        let pos = random_in_radius(&mut rng, offset, 3.).extend(90.0);

                        commands
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
                            .insert(Ennemy);
                    }
                }
            }
        }
        3 if !ennemy_wave_spawned.spawn(2) => {
            let mut rng = rand::thread_rng();
            for player_transform in player_query.iter() {
                for _ in 0..3 {
                    let origin = Vec2::new(rng.gen_range(-10.0..10.0), rng.gen_range(-10.0..10.0));
                    let origin = move_from_deadzone(origin, 10.0);
                    let offset = player_transform.translation + origin.extend(0.0);

                    for _ in 0..60 {
                        let pos = random_in_radius(&mut rng, offset, 3.).extend(90.0);

                        commands
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
                            .insert(Ennemy);
                    }
                }
            }
        }
        5 if !ennemy_wave_spawned.spawn(3) => {
            let mut rng = rand::thread_rng();
            for player_transform in player_query.iter() {
                for _ in 0..3 {
                    let origin = Vec2::new(rng.gen_range(-10.0..10.0), rng.gen_range(-10.0..10.0));
                    let origin = move_from_deadzone(origin, 10.0);
                    let offset = player_transform.translation + origin.extend(0.0);

                    for _ in 0..60 {
                        let pos = random_in_radius(&mut rng, offset, 3.).extend(90.0);

                        commands
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
                            .insert(Ennemy);
                    }
                }
            }
        }
        _ => (),
    }
}

fn move_ennemies(
    player_query: Query<&Transform, With<Player>>,
    mut ennemies_query: Query<
        (
            &mut Velocity,
            &mut Transform,
            &mut TextureAtlasSprite,
            &BaseSpriteRotation,
            &BaseSpriteFlip,
        ),
        (With<Ennemy>, Without<Player>),
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
        let strenght = player.distance(ennemy).min(FISH_MAX_SPEED);
        velocity.0 += direction * strenght * FISH_SPEED;

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

fn ennemies_repulsion(mut ennemies_query: Query<(&mut Velocity, &Transform, &Ennemy)>) {
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

fn gems_player_attraction(
    time: Res<Time>,
    mut loot_all_gems: ResMut<LootAllGemsFor>,
    player_query: Query<&Transform, With<Player>>,
    mut gems_query: Query<(&mut Transform, &mut MoveToPlayer), (With<Gem>, Without<Player>)>,
) {
    let player_transform = match player_query.iter().next() {
        Some(transform) => transform,
        None => return,
    };

    loot_all_gems.0.tick(time.delta());

    for (mut transform, mut move_to_player) in gems_query.iter_mut() {
        let dist = player_transform.translation.xy().distance(transform.translation.xy());
        if move_to_player.0 || !loot_all_gems.0.finished() || dist <= 2.0 {
            move_to_player.0 = true;
            let dir = (player_transform.translation.xy() - transform.translation.xy())
                .normalize_or_zero();
            transform.translation += (dir * time.delta_seconds() * 15.0).extend(0.);
        }
    }
}

fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    for player_transform in player_query.iter() {
        let pos = player_transform.translation;

        for mut transform in camera_query.iter_mut() {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}

#[derive(Default, Component)]
pub struct Player {
    lvl1_stuff_generated: bool,
    xp: usize,
}

#[derive(Component)]
pub struct Ennemy;

#[derive(Component)]
pub struct Gem;

#[derive(Component)]
pub enum Stuff {
    FishingRod,
}

#[derive(Component)]
pub struct AxeHead;

#[derive(Component)]
pub struct LootAllGemsFor(Timer);

#[derive(Component)]
pub struct RotationRadian(f32);

#[derive(Default, Component)]
pub struct Velocity(Vec2);

#[derive(Default)]
pub struct EnnemyWaves([bool; 4]);

#[derive(Default, Component)]
pub struct MoveToPlayer(bool);

impl EnnemyWaves {
    /// Set the next it to `true` and returns the previous value.
    pub fn spawn(&mut self, nth: usize) -> bool {
        let EnnemyWaves(spawned) = self;
        mem::replace(&mut spawned[nth], true)
    }
}

#[derive(Component)]
pub struct Health(pub usize);
