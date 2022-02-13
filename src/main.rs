use bevy::prelude::*;
use bevy_asset_loader::AssetLoader;
use bevy_tweening::*;
use heron::prelude::*;
use rand::Rng;

use self::animations::*;
use self::assets::*;

mod animations;
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
        .with_collection::<EnemiesAssets>()
        .build(&mut app);

    app.add_state(MyStates::AssetLoading)
        .insert_resource(ClearColor(Color::rgb(0.53, 0.53, 0.53)))
        .add_plugins(DefaultPlugins)
        .add_plugin(TweeningPlugin)
        .add_plugin(PhysicsPlugin::default())
        .insert_resource(Gravity::from(Vec3::ZERO))
        .add_startup_system(setup)
        .add_startup_system(spawn_player)
        .add_system_to_stage(CoreStage::PostUpdate, camera_follow)
        .add_system_set(SystemSet::on_enter(MyStates::Next).with_system(spawn_ennemies_fleet))
        .add_system_set(
            SystemSet::on_update(MyStates::Next)
                .with_system(free_enemies_from_fleets)
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

fn spawn_ennemies_fleet(mut commands: Commands, enemies_assets: Res<EnemiesAssets>) {
    let small_demons_count = 12 * 3;
    let mut rng = rand::thread_rng();

    let start = Vec3::new(-5., 5., 90.);
    let tween = space_invader_animation(start, 5, 10., 0.8);

    commands
        .spawn_bundle(FleetBundle {
            transform: Transform::from_translation(start),
            ..Default::default()
        })
        .insert(Timer::new(tween.duration(), false))
        .insert(Animator::new(tween))
        .with_children(|parent| {
            for column in -5..=5 {
                for row in -2..=1 {
                    let position = Vec3::new(column as f32, row as f32, 0.) * 1.2;
                    let index = rng.gen_range(0..small_demons_count);

                    parent
                        .spawn_bundle(SpriteSheetBundle {
                            transform: Transform::from_translation(position),
                            sprite: TextureAtlasSprite {
                                index,
                                custom_size: Some(Vec2::new(1., 1.)),
                                ..Default::default()
                            },
                            texture_atlas: enemies_assets.small_demons.clone(),
                            ..Default::default()
                        })
                        .insert(Velocity::default())
                        .insert(RigidBody::KinematicPositionBased)
                        .insert(CollisionShape::Sphere { radius: 0.6 })
                        .insert(RotationConstraints::lock())
                        .insert(
                            CollisionLayers::none()
                                .with_group(GameLayer::Enemy)
                                .with_masks(&[GameLayer::Player, GameLayer::Enemy]),
                        )
                        .insert(InFleet)
                        .insert(Enemy);
                }
            }
        });
}

fn free_enemies_from_fleets(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<&GlobalTransform, With<Player>>,
    mut query_fleet: Query<(&mut Timer, Entity, &Children), With<Fleet>>,
    mut query_enemy: Query<
        (&mut Velocity, &mut RotationConstraints, &mut RigidBody, &mut Transform, &GlobalTransform),
        (With<Enemy>, With<InFleet>),
    >,
) {
    let player_transform = player_query.single();

    for (mut timer, parent, children) in query_fleet.iter_mut() {
        if timer.tick(time.delta()).finished() {
            commands.entity(parent).remove_children(children);

            for child in children.iter().copied() {
                // TODO do not change the velocity here, declare a FollowNearestPlayer marker
                let (
                    mut velocity,
                    mut rotation_constraints,
                    mut rigid_body,
                    mut transform,
                    global_transform,
                ) = query_enemy.get_mut(child).unwrap();

                *transform.translation = *global_transform.translation;
                velocity.angular = AxisAngle::new(Vec3::Z * 0.01, 6.);
                velocity.linear = (player_transform.translation - transform.translation)
                    .normalize_or_zero()
                    * 2.0;
                *rotation_constraints = RotationConstraints::allow();
                *rigid_body = RigidBody::Dynamic;
                commands.entity(child).remove::<InFleet>();
            }

            commands.entity(parent).despawn_recursive();
        }
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

#[derive(Component)]
pub enum Stuff {
    FishingRod,
}

#[derive(Component)]
pub struct AxeHead;

#[derive(Component)]
pub struct RotationRadian(f32);

#[derive(Default, Component)]
pub struct MoveToPlayer(bool);

#[derive(Component)]
pub struct Health(pub usize);

#[derive(PhysicsLayer)]
pub enum GameLayer {
    Player,
    Weapon,
    Enemy,
    Gem,
    Stuff,
}
