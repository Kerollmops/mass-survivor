use std::f32::consts::PI;
use std::time::Duration;

use benimator::*;
use bevy::prelude::*;
use bevy_asset_loader::AssetLoader;
use bevy_tweening::*;
use heron::prelude::*;
use ordered_float::NotNan;
use rand::Rng;

use self::assets::*;

mod assets;

const MAP_SIZE: u32 = 41;
const GRID_WIDTH: f32 = 0.05;
const PLAYER_SPEED: f32 = 10.0;

const HEALTHY_PLAYER_COLOR: Color = Color::rgb(0., 0.47, 1.);
const HIT_PLAYER_COLOR: Color = Color::rgb(0.9, 0.027, 0.);

fn main() {
    let mut app = App::new();
    AssetLoader::new(MyStates::AssetLoading)
        .continue_to_state(MyStates::Next)
        .with_collection::<AnimatedAssets>()
        .build(&mut app);

    app.add_state(MyStates::AssetLoading)
        .insert_resource(ClearColor(Color::rgb(0.53, 0.53, 0.53)))
        .add_plugins(DefaultPlugins)
        .add_plugin(AnimationPlugin::default())
        .add_plugin(TweeningPlugin)
        .add_plugin(PhysicsPlugin::default())
        .insert_resource(Gravity::from(Vec3::ZERO))
        .add_startup_system(setup)
        .add_startup_system(spawn_player)
        .add_system_to_stage(CoreStage::PostUpdate, camera_follow)
        .add_system_set(SystemSet::on_enter(MyStates::Next).with_system(spawn_ennemies))
        .add_system_set(
            SystemSet::on_update(MyStates::Next)
                .with_system(follow_nearest_player)
                .with_system(move_player)
                .with_system(change_player_color)
                .with_system(player_loot_gems),
        )
        .run();
}

fn setup(mut commands: Commands) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.scale = 1. / 50.;
    commands.spawn_bundle(camera_bundle);

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
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(0., 0., 100.)),
            sprite: Sprite {
                color: HEALTHY_PLAYER_COLOR,
                custom_size: Some(Vec2::new(1., 1.)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Velocity::default())
        .insert(RigidBody::Dynamic)
        .insert(CollisionShape::Cuboid { half_extends: Vec3::splat(0.5), border_radius: None })
        .insert(RotationConstraints::lock())
        .insert(CollisionLayers::none().with_group(GameLayer::Player).with_masks(&[
            GameLayer::Gem,
            GameLayer::Stuff,
            GameLayer::Enemy,
        ]))
        .insert(Player::default());
}

fn spawn_ennemies(
    mut commands: Commands,
    player_query: Query<&GlobalTransform, With<Player>>,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
    animated_assets: Res<AnimatedAssets>,
) {
    let mut i = 1;
    for x in -5..=5 {
        for y in -5..=5 {
            if x == 0 && y == 0 {
                continue;
            }

            let index = i * 25 + 1 + (1 * 4);
            let animation_handle = animations.add(SpriteSheetAnimation::from_range(
                index..=index + 3,
                Duration::from_secs_f64(1.0 / 6.0),
            ));

            commands
                .spawn_bundle(SpriteSheetBundle {
                    transform: Transform::from_translation(Vec3::new(x as f32, y as f32, 90.)),
                    sprite: TextureAtlasSprite {
                        custom_size: Some(Vec2::new(1., 1.)),
                        ..Default::default()
                    },
                    texture_atlas: animated_assets.magic_elementals.clone(),
                    ..Default::default()
                })
                // animation settings
                .insert(animation_handle)
                .insert(Play)
                .insert(Velocity::default())
                .insert(Acceleration::default())
                .insert(MaxSpeed(0.2))
                .insert(RigidBody::Dynamic)
                .insert(Damping::from_linear(1.))
                .insert(CollisionShape::Sphere { radius: 0.5 })
                .insert(RotationConstraints::lock())
                .insert(
                    CollisionLayers::none()
                        .with_group(GameLayer::Enemy)
                        .with_masks(&[GameLayer::Player, GameLayer::Enemy]),
                )
                .insert(InFleet)
                .insert(Enemy)
                .with_children(|parent| {
                    if x == 1 && y == 1 {
                        parent.spawn_bundle(SpriteBundle {
                            transform: Transform::from_translation(Vec3::new(0., 1.2, 150.))
                                .with_rotation(Quat::from_rotation_z(PI / 4.)),
                            sprite: Sprite {
                                color: Color::WHITE,
                                custom_size: Some(Vec2::new(0.8, 0.8)),
                                ..Default::default()
                            },
                            ..Default::default()
                        });

                        parent.spawn_bundle(SpriteSheetBundle {
                            transform: Transform::from_translation(Vec3::new(0., 1.25, 151.)),
                            sprite: TextureAtlasSprite {
                                index: 8 * 25,
                                custom_size: Some(Vec2::new(0.5, 0.5)),
                                ..Default::default()
                            },
                            texture_atlas: animated_assets.magic_elementals.clone(),
                            ..Default::default()
                        });
                    }
                });

            i = (i + 1) % 9 + 1;
        }
    }
}

// Find the nearest player by using the squared distance (faster to compute)
fn follow_nearest_player(
    player_query: Query<&GlobalTransform, With<Player>>,
    mut enemy_query: Query<
        (&Transform, &mut Velocity, &mut Acceleration, &MaxSpeed),
        With<FollowNearestPlayer>,
    >,
) {
    for (transform, mut velocity, mut acceleration, max_speed) in enemy_query.iter_mut() {
        let MaxSpeed(max_speed) = *max_speed;
        match player_query.iter().min_by_key(|pt| {
            NotNan::new(pt.translation.distance_squared(transform.translation)).unwrap()
        }) {
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
    mut player_query: Query<(Entity, &mut Sprite), With<Player>>,
) {
    let (entity, mut player_sprite) = match player_query.iter_mut().next() {
        Some(value) => value,
        None => return,
    };

    for event in events.iter() {
        if let CollisionEvent::Started(data1, data2) = event {
            let a = data1.collision_shape_entity();
            let b = data2.collision_shape_entity();

            if a == entity || b == entity {
                player_sprite.color = HIT_PLAYER_COLOR;
                return;
            }
        }
    }

    player_sprite.color = HEALTHY_PLAYER_COLOR;
}

fn player_loot_gems(
    mut commands: Commands,
    mut player_query: Query<&mut Player>,
    mut events: EventReader<CollisionEvent>,
) {
    let mut player = player_query.single_mut();

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
            player.xp += 1;
            commands.entity(gem_entity).despawn();
        });
}

fn is_player_layer(layers: CollisionLayers) -> bool {
    layers.contains_group(GameLayer::Player)
}

fn is_gem_layer(layers: CollisionLayers) -> bool {
    layers.contains_group(GameLayer::Gem)
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
pub struct Enemy;

#[derive(Component)]
pub struct Health {
    pub current: usize,
    pub max: usize,
}

impl Health {
    pub fn new(max: usize) -> Health {
        Health { current: max, max }
    }
}

#[derive(Component)]
pub struct MaxSpeed(f32);

#[derive(Component)]
pub struct InFleet;

#[derive(Default, Component)]
pub struct Fleet;

#[derive(Default, Bundle)]
pub struct FleetBundle {
    transform: Transform,
    global_transform: GlobalTransform,
    _fleet: Fleet,
}

#[derive(Default, Component)]
pub struct Player {
    xp: usize,
}

#[derive(Component)]
pub struct Gem;

#[derive(Default, Component)]
pub struct FollowNearestPlayer;

#[derive(PhysicsLayer)]
pub enum GameLayer {
    Player,
    Weapon,
    Enemy,
    Gem,
    Stuff,
}
