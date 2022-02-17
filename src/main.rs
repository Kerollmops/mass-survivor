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
const PLAYER_SPEED: f32 = 3.0;
const UNITS_Z_INDEX: f32 = 90.0;

const HEALTHY_PLAYER_COLOR: Color = Color::rgb(0., 0.47, 1.);
const HIT_PLAYER_COLOR: Color = Color::rgb(0.9, 0.027, 0.);

fn main() {
    let mut app = App::new();
    AssetLoader::new(MyStates::AssetLoading)
        .continue_to_state(MyStates::Next)
        .with_collection::<GameAssets>()
        .build(&mut app);

    app.add_state(MyStates::AssetLoading)
        .insert_resource(ClearColor(Color::rgb(0.53, 0.53, 0.53)))
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
                .with_system(move_player)
                .with_system(change_player_color)
                .with_system(player_loot_gems),
        )
        .add_system_to_stage(CoreStage::PostUpdate, change_animation_from_velocity)
        .add_system_to_stage(CoreStage::PostUpdate, reorder_sprite_units)
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

fn spawn_player(
    mut commands: Commands,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
    game_assets: Res<GameAssets>,
) {
    let texture_atlas = game_assets.castle.clone();
    let character = Castle::SimpleMonk;
    let animations_set = AnimationsSet {
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

fn spawn_ennemies(
    mut commands: Commands,
    _player_query: Query<&GlobalTransform, With<Player>>,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
    game_assets: Res<GameAssets>,
) {
    let mut rng = rand::thread_rng();

    for x in -5..=5 {
        for y in -5..=5 {
            if x == 0 && y == 0 {
                continue;
            }

            let (texture_atlas, animations_set) = if rng.gen() {
                let elemental = Elemental::from_rng(&mut rng);
                let animation = AnimationsSet {
                    idle: animations.add(elemental.idle_animation()),
                    walk: animations.add(elemental.walk_animation()),
                    attack: animations.add(elemental.attack_animation()),
                };
                (game_assets.magic_elementals.clone(), animation)
            } else if rng.gen() {
                let inferno = Inferno::from_rng(&mut rng);
                let animation = AnimationsSet {
                    idle: animations.add(inferno.idle_animation()),
                    walk: animations.add(inferno.walk_animation()),
                    attack: animations.add(inferno.attack_animation()),
                };
                (game_assets.infernos.clone(), animation)
            } else {
                let necromancer = Necromancer::from_rng(&mut rng);
                let animation = AnimationsSet {
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
                .insert(MaxSpeed(0.2))
                .insert(RigidBody::Dynamic)
                .insert(Damping::from_linear(1.))
                .insert(CollisionShape::Cuboid {
                    half_extends: Vec3::new(0.32, 0.25, 0.),
                    border_radius: None,
                })
                .insert(RotationConstraints::lock())
                .insert(
                    CollisionLayers::none()
                        .with_group(GameLayer::Enemy)
                        .with_masks(&[GameLayer::Player, GameLayer::Enemy]),
                )
                .insert(Enemy)
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
        &AnimationsSet,
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
    mut parent_query: Query<(&Children, Entity), With<Player>>,
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
pub struct AnimationsSet {
    idle: Handle<SpriteSheetAnimation>,
    walk: Handle<SpriteSheetAnimation>,
    attack: Handle<SpriteSheetAnimation>,
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

#[derive(Default, Component)]
pub struct Player;

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
