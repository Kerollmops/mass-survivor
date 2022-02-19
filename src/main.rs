use std::time::Duration;

use benimator::*;
use bevy::prelude::*;
use bevy_asset_loader::AssetLoader;
use bevy_tweening::*;
use heron::prelude::*;
use ordered_float::OrderedFloat;
use rand::Rng;

use self::assets::*;

mod assets;

const MAP_SIZE: u32 = 41;
const GRID_WIDTH: f32 = 0.05;
const PLAYER_SPEED: f32 = 3.0;
const UNITS_Z_INDEX: f32 = 90.0;
const CONVERTING_WEAPON_DISTANCE: f32 = 3.5;
const PINK_CONVERTING_WEAPON: Color = Color::rgba(1., 0.584, 0.753, 1.);

const HIT_PLAYER_COLOR: Color = Color::rgb(0.9, 0.027, 0.);

fn main() {
    let mut app = App::new();
    AssetLoader::new(MyStates::AssetLoading)
        .continue_to_state(MyStates::Next)
        .with_collection::<GameAssets>()
        .build(&mut app);

    app.add_state(MyStates::AssetLoading)
        .insert_resource(ClearColor(Color::rgb(0.53, 0.53, 0.53)))
        .insert_resource(ConvertingWeaponTimer(Timer::new(Duration::from_secs(2), false)))
        .add_plugins(DefaultPlugins)
        .add_plugin(AnimationPlugin::default())
        .add_plugin(TweeningPlugin)
        .add_plugin(PhysicsPlugin::default())
        .insert_resource(Gravity::from(Vec3::ZERO))
        .add_startup_system(setup)
        .add_system_to_stage(CoreStage::PostUpdate, camera_follow)
        .add_system_set(
            SystemSet::on_enter(MyStates::Next)
                .with_system(spawn_player)
                .with_system(spawn_ennemies),
        )
        .add_system_set(
            SystemSet::on_update(MyStates::Next)
                .with_system(follow_nearest_player)
                .with_system(follow_nearest_enemy)
                .with_system(move_player)
                .with_system(change_player_color)
                .with_system(player_loot_gems)
                .with_system(move_converting_weapon_from_velocity)
                .with_system(mark_enemies_under_converting_weapon)
                .with_system(convert_enemies_under_converting_weapon)
                .with_system(change_love_animation),
        )
        .add_system_to_stage(CoreStage::PostUpdate, delete_converting_weapon_animation)
        .add_system_to_stage(CoreStage::PostUpdate, change_animation_from_velocity)
        .add_system_to_stage(CoreStage::PostUpdate, reorder_sprite_units)
        .run();
}

fn setup(mut commands: Commands, mut animations: ResMut<Assets<SpriteSheetAnimation>>) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.scale = 1. / 50.;
    commands.spawn_bundle(camera_bundle);

    // generate the pink selector animations
    commands.insert_resource(PinkSelectorAnimationsSet {
        full: animations.add(PinkSelector::Full.animation()),
        two_third: animations.add(PinkSelector::TwoThird.animation()),
        one_third: animations.add(PinkSelector::OneThird.animation()),
        empty: animations.add(PinkSelector::Empty.animation()),
    });

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

fn spawn_player(
    mut commands: Commands,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
    game_assets: Res<GameAssets>,
) {
    let character = Castle::SimpleMonk;
    let animations_set = CharactersAnimationsSet {
        idle: animations.add(character.idle_animation()),
        walk: animations.add(character.walk_animation()),
        attack: animations.add(character.attack_animation()),
    };

    commands
        .spawn()
        .insert(Transform::from_translation(Vec3::new(0., 0., UNITS_Z_INDEX)))
        .insert(GlobalTransform::default())
        .insert(Velocity::default())
        .insert(RigidBody::Dynamic)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(0.30, 0.25, 0.),
            border_radius: None,
        })
        .insert(RotationConstraints::lock())
        .insert(CollisionLayers::none().with_group(GameLayer::Player).with_masks(&[
            GameLayer::Gem,
            GameLayer::Stuff,
            GameLayer::Enemy,
        ]))
        .insert(Player)
        .with_children(|parent| {
            // spawn the sprite
            parent
                .spawn_bundle(SpriteSheetBundle {
                    transform: Transform::from_translation(Vec3::new(0., 0.25, 0.)),
                    sprite: TextureAtlasSprite {
                        custom_size: Some(Vec2::new(1., 1.)),
                        ..Default::default()
                    },
                    texture_atlas: game_assets.castle.clone(),
                    ..Default::default()
                })
                // animation settings
                .insert(animations_set.idle.clone())
                .insert(animations_set)
                .insert(Play);

            // spawn the converting weapon
            parent
                .spawn()
                .insert(Transform::from_translation(Vec3::new(
                    CONVERTING_WEAPON_DISTANCE,
                    0.,
                    UNITS_Z_INDEX,
                )))
                .insert(GlobalTransform::default())
                .insert(RigidBody::Sensor)
                .insert(CollisionShape::Sphere { radius: 1.8 })
                .insert(RotationConstraints::lock())
                .insert(
                    CollisionLayers::none()
                        .with_group(GameLayer::ConvertingWeapon)
                        .with_mask(GameLayer::Enemy),
                )
                .insert(ConvertingWeapon);
        });
}

fn spawn_ennemies(
    mut commands: Commands,
    _player_query: Query<&GlobalTransform, With<Player>>,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
    game_assets: Res<GameAssets>,
) {
    let mut rng = rand::thread_rng();

    for x in -15..=-5 {
        for y in -5..=5 {
            let (texture_atlas, animations_set) = if rng.gen() {
                let elemental = Elemental::from_rng(&mut rng);
                let animation = CharactersAnimationsSet {
                    idle: animations.add(elemental.idle_animation()),
                    walk: animations.add(elemental.walk_animation()),
                    attack: animations.add(elemental.attack_animation()),
                };
                (game_assets.magic_elementals.clone(), animation)
            } else if rng.gen() {
                let inferno = Inferno::from_rng(&mut rng);
                let animation = CharactersAnimationsSet {
                    idle: animations.add(inferno.idle_animation()),
                    walk: animations.add(inferno.walk_animation()),
                    attack: animations.add(inferno.attack_animation()),
                };
                (game_assets.infernos.clone(), animation)
            } else {
                let necromancer = Necromancer::from_rng(&mut rng);
                let animation = CharactersAnimationsSet {
                    idle: animations.add(necromancer.idle_animation()),
                    walk: animations.add(necromancer.walk_animation()),
                    attack: animations.add(necromancer.attack_animation()),
                };
                (game_assets.necromancers.clone(), animation)
            };

            commands
                .spawn()
                .insert(Transform::from_translation(Vec3::new(x as f32, y as f32, UNITS_Z_INDEX)))
                .insert(GlobalTransform::default())
                .insert(Velocity::default())
                .insert(Acceleration::default())
                .insert(RigidBody::Dynamic)
                .insert(Damping::from_linear(1.))
                .insert(CollisionShape::Cuboid {
                    half_extends: Vec3::new(0.32, 0.25, 0.),
                    border_radius: None,
                })
                .insert(RotationConstraints::lock())
                .insert(CollisionLayers::none().with_group(GameLayer::Enemy).with_masks(&[
                    GameLayer::Player,
                    GameLayer::Enemy,
                    GameLayer::ConvertingWeapon,
                ]))
                .insert(Enemy)
                .insert(FollowNearest { max_speed: rng.gen_range(0.2..=1.2) })
                .insert(UnderConvertingWeapon(false))
                .with_children(|parent| {
                    parent
                        .spawn_bundle(SpriteSheetBundle {
                            transform: Transform::from_translation(Vec3::new(0., 0.25, 0.)),
                            sprite: TextureAtlasSprite {
                                custom_size: Some(Vec2::new(1., 1.)),
                                ..Default::default()
                            },
                            texture_atlas,
                            ..Default::default()
                        })
                        // animation settings
                        .insert(animations_set.idle.clone())
                        .insert(animations_set)
                        .insert(Play);
                });
        }
    }
}

fn reorder_sprite_units(mut shape_query: Query<&mut Transform, With<CollisionShape>>) {
    for mut transform in shape_query.iter_mut() {
        transform.translation[2] = UNITS_Z_INDEX - transform.translation[1] * 0.1;
    }
}

fn change_animation_from_velocity(
    parent_query: Query<(&Velocity, &Children)>,
    mut child_query: Query<(
        &mut TextureAtlasSprite,
        &mut Handle<SpriteSheetAnimation>,
        &CharactersAnimationsSet,
    )>,
) {
    for (velocity, children) in parent_query.iter() {
        for &child in children.iter() {
            if let Ok((mut sprite, mut animation, animations_set)) = child_query.get_mut(child) {
                if velocity.linear[0] > 0. {
                    sprite.flip_x = false;
                } else if velocity.linear[0] < 0. {
                    sprite.flip_x = true;
                }

                if velocity.linear.length() < 0.1 {
                    *animation = animations_set.idle.clone();
                } else {
                    *animation = animations_set.walk.clone();
                }
            }
        }
    }
}

fn move_converting_weapon_from_velocity(
    parent_query: Query<(&Velocity, &Children)>,
    mut child_query: Query<&mut Transform, With<ConvertingWeapon>>,
) {
    for (velocity, children) in parent_query.iter() {
        for &child in children.iter() {
            if let Ok(mut transform) = child_query.get_mut(child) {
                if velocity.linear[0] > 0. {
                    transform.translation[0] = CONVERTING_WEAPON_DISTANCE;
                } else if velocity.linear[0] < 0. {
                    transform.translation[0] = -CONVERTING_WEAPON_DISTANCE;
                } else if velocity.linear[1] != 0. {
                    transform.translation[0] = 0.;
                }

                if velocity.linear[1] > 0. {
                    transform.translation[1] = CONVERTING_WEAPON_DISTANCE;
                } else if velocity.linear[1] < 0. {
                    transform.translation[1] = -CONVERTING_WEAPON_DISTANCE;
                } else if velocity.linear[0] != 0. {
                    transform.translation[1] = 0.;
                }
            }
        }
    }
}

/// Find the nearest player by using the squared distance (faster to compute).
fn follow_nearest_player(
    player_query: Query<&GlobalTransform, With<Player>>,
    mut enemy_query: Query<
        (&Transform, &mut Velocity, &mut Acceleration, &FollowNearest),
        With<Enemy>,
    >,
) {
    for (transform, mut velocity, mut acceleration, follow) in enemy_query.iter_mut() {
        let max_speed = follow.max_speed;
        match player_query
            .iter()
            .min_by_key(|pt| OrderedFloat(pt.translation.distance_squared(transform.translation)))
        {
            Some(pt) => {
                let direction = (pt.translation - transform.translation).normalize_or_zero();
                let limit = Vec3::splat(max_speed);
                velocity.linear = velocity.linear.clamp(-limit, limit);
                acceleration.linear = direction * 7.;
            }
            None => acceleration.linear = Vec3::ZERO,
        }
    }
}

/// Find the nearest enemy by using the squared distance (faster to compute).
fn follow_nearest_enemy(
    enemy_query: Query<&GlobalTransform, With<Enemy>>,
    mut ally_query: Query<
        (&Transform, &mut Velocity, &mut Acceleration, &FollowNearest),
        With<Ally>,
    >,
) {
    for (transform, mut velocity, mut acceleration, follow) in ally_query.iter_mut() {
        let max_speed = follow.max_speed;
        match enemy_query
            .iter()
            .min_by_key(|pt| OrderedFloat(pt.translation.distance_squared(transform.translation)))
        {
            Some(pt) => {
                let direction = (pt.translation - transform.translation).normalize_or_zero();
                let limit = Vec3::splat(max_speed);
                velocity.linear = velocity.linear.clamp(-limit, limit);
                acceleration.linear = direction * 7.;
            }
            None => acceleration.linear = Vec3::ZERO,
        }
    }
}

fn move_player(keys: Res<Input<KeyCode>>, mut player_query: Query<&mut Velocity, With<Player>>) {
    for mut velocity in player_query.iter_mut() {
        let y = if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
            1.
        } else if keys.any_pressed([KeyCode::Down, KeyCode::S]) {
            -1.
        } else {
            0.
        };

        let x = if keys.any_pressed([KeyCode::Right, KeyCode::D]) {
            1.
        } else if keys.any_pressed([KeyCode::Left, KeyCode::A]) {
            -1.
        } else {
            0.
        };

        velocity.linear = Vec2::new(x, y).normalize_or_zero().extend(0.) * PLAYER_SPEED;
    }
}

fn change_player_color(
    mut events: EventReader<CollisionEvent>,
    parent_query: Query<(&Children, Entity), With<Player>>,
    mut child_query: Query<&mut TextureAtlasSprite>,
) {
    let (children, entity) = parent_query.single();

    for &child in children.iter() {
        if let Ok(mut sprite) = child_query.get_mut(child) {
            for event in events.iter() {
                if let CollisionEvent::Started(data1, data2) = event {
                    let a = data1.collision_shape_entity();
                    let b = data2.collision_shape_entity();

                    if a == entity || b == entity {
                        sprite.color = HIT_PLAYER_COLOR;
                        return;
                    }
                }
            }

            sprite.color = Color::default();
        }
    }
}

fn player_loot_gems(
    mut commands: Commands,
    _player_query: Query<&mut Player>,
    mut events: EventReader<CollisionEvent>,
) {
    // let mut player = player_query.single_mut();

    events
        .iter()
        .filter(|e| e.is_started())
        .filter_map(|event| {
            let (entity_1, entity_2) = event.rigid_body_entities();
            let (layers_1, layers_2) = event.collision_layers();
            if is_player_layer(layers_1) && is_gem_layer(layers_2) {
                Some(entity_2)
            } else if is_player_layer(layers_2) && is_gem_layer(layers_1) {
                Some(entity_1)
            } else {
                None
            }
        })
        .for_each(|gem_entity| {
            // player.xp += 1;
            commands.entity(gem_entity).despawn();
        });
}

fn mark_enemies_under_converting_weapon(
    mut events: EventReader<CollisionEvent>,
    mut enemies_query: Query<&mut UnderConvertingWeapon, With<Enemy>>,
) {
    events
        .iter()
        .filter_map(|event| {
            let (entity_1, entity_2) = event.rigid_body_entities();
            let (layers_1, layers_2) = event.collision_layers();
            if is_converting_weapon_layer(layers_1) && is_enemy_layer(layers_2) {
                Some((event, entity_2))
            } else if is_converting_weapon_layer(layers_2) && is_enemy_layer(layers_1) {
                Some((event, entity_1))
            } else {
                None
            }
        })
        .for_each(|(event, enemy_entity)| {
            if let Ok(mut under_converting_weapon) = enemies_query.get_mut(enemy_entity) {
                under_converting_weapon.0 = event.is_started();
            }
        });
}

fn convert_enemies_under_converting_weapon(
    time: Res<Time>,
    mut commands: Commands,
    mut converting_weapon_timer: ResMut<ConvertingWeaponTimer>,
    converting_weapon_query: Query<&GlobalTransform, With<ConvertingWeapon>>,
    mut enemies_query: Query<(Entity, &mut UnderConvertingWeapon), With<Enemy>>,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
    game_assets: Res<GameAssets>,
) {
    if converting_weapon_timer.0.tick(time.delta()).finished() {
        converting_weapon_timer.0.reset();

        // Spawn the one-time animation
        let global_transform = converting_weapon_query.single();
        let animation =
            SpriteSheetAnimation::from_range(0..=19, Duration::from_secs_f64(1.0 / 18.0)).once();
        commands
            .spawn_bundle(SpriteSheetBundle {
                transform: Transform::from_translation(global_transform.translation),
                sprite: TextureAtlasSprite {
                    color: PINK_CONVERTING_WEAPON,
                    custom_size: Some(Vec2::new(5., 5.)),
                    ..Default::default()
                },
                texture_atlas: game_assets.smoke_effect_07.clone(),
                ..Default::default()
            })
            .insert(animations.add(animation))
            .insert(Play)
            .insert(ConvertingWeaponAnimation);

        let mut rng = rand::thread_rng();
        for (entity, under_converting_weapon) in enemies_query.iter_mut() {
            if under_converting_weapon.0 && rng.gen::<f32>() < 0.2 {
                commands
                    .entity(entity)
                    .with_children(|parent| {
                        // Spawn the love animation
                        parent
                            .spawn_bundle(SpriteSheetBundle {
                                transform: Transform::from_translation(Vec3::new(0., 1.1, 0.)),
                                sprite: TextureAtlasSprite {
                                    custom_size: Some(Vec2::new(0.7, 0.7)),
                                    ..Default::default()
                                },
                                texture_atlas: game_assets.pink_selector_01.clone(),
                                ..Default::default()
                            })
                            .insert(animations.add(PinkSelector::Full.animation()))
                            .insert(Play);
                    })
                    .insert(Ally)
                    .remove::<Enemy>();
            }
        }
    }
}

fn delete_converting_weapon_animation(
    mut commands: Commands,
    removals: RemovedComponents<Play>,
    animation_query: Query<&ConvertingWeaponAnimation>,
) {
    for entity in removals.iter() {
        if let Ok(_) = animation_query.get(entity) {
            commands.entity(entity).despawn();
        }
    }
}

fn change_love_animation(
    pink_selector_animations: Res<PinkSelectorAnimationsSet>,
    ally_query: Query<(&Children, &Health), (With<Ally>, Changed<Health>)>,
    mut child_query: Query<&mut Handle<SpriteSheetAnimation>>,
) {
    for (children, health) in ally_query.iter() {
        if let Ok(mut animation) = child_query.get_mut(children[0]) {
            *animation = match health {
                Health::Full => pink_selector_animations.full.clone(),
                Health::OneThird => pink_selector_animations.one_third.clone(),
                Health::TwoThird => pink_selector_animations.two_third.clone(),
                Health::Empty => pink_selector_animations.empty.clone(),
            };
        }
    }
}

fn is_player_layer(layers: CollisionLayers) -> bool {
    layers.contains_group(GameLayer::Player)
}

fn is_gem_layer(layers: CollisionLayers) -> bool {
    layers.contains_group(GameLayer::Gem)
}

fn is_enemy_layer(layers: CollisionLayers) -> bool {
    layers.contains_group(GameLayer::Enemy)
}

fn is_converting_weapon_layer(layers: CollisionLayers) -> bool {
    layers.contains_group(GameLayer::ConvertingWeapon)
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

#[derive(Component)]
pub struct CharactersAnimationsSet {
    idle: Handle<SpriteSheetAnimation>,
    walk: Handle<SpriteSheetAnimation>,
    attack: Handle<SpriteSheetAnimation>,
}

#[derive(Component)]
pub struct PinkSelectorAnimationsSet {
    full: Handle<SpriteSheetAnimation>,
    two_third: Handle<SpriteSheetAnimation>,
    one_third: Handle<SpriteSheetAnimation>,
    empty: Handle<SpriteSheetAnimation>,
}

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Ally;

#[derive(Component)]
pub enum Health {
    Full,
    TwoThird,
    OneThird,
    Empty,
}

#[derive(Default, Component)]
pub struct Player;

#[derive(Component)]
pub struct Gem;

#[derive(Component)]
pub struct FollowNearest {
    max_speed: f32,
}

impl Default for FollowNearest {
    fn default() -> FollowNearest {
        FollowNearest { max_speed: 0.5 }
    }
}

#[derive(PhysicsLayer)]
pub enum GameLayer {
    Player,
    Weapon,
    ConvertingWeapon,
    Enemy,
    Gem,
    Stuff,
}

#[derive(Component)]
pub struct ConvertingWeapon;

#[derive(Component)]
pub struct ConvertingWeaponTimer(Timer);

#[derive(Component)]
pub struct ConvertingWeaponAnimation;

#[derive(Component)]
pub struct UnderConvertingWeapon(bool);
