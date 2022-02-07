use std::f32::consts::PI;

use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::transform::TransformSystem;
use bevy_asset_loader::AssetLoader;
use heron::prelude::*;
use rand::Rng;

use self::assets::*;
use self::enemies::*;
use self::game_sprites::*;
use self::helper::*;

mod assets;
mod enemies;
mod game_sprites;
mod helper;

const MAP_SIZE: u32 = 41;
const GRID_WIDTH: f32 = 0.05;
const SLOW_DOWN: f32 = 4.0;
const PLAYER_SPEED: f32 = 10.0;

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
        .add_plugin(PhysicsPlugin::default())
        .insert_resource(Gravity::from(Vec3::ZERO))
        .add_startup_system(setup)
        .add_startup_system(spawn_player)
        .add_system_to_stage(CoreStage::PostUpdate, camera_follow)
        .add_system_set(
            SystemSet::on_update(MyStates::Next)
                .with_system(move_player)
                .with_system(tracking_movement)
                .with_system(slow_walking_movement)
                .with_system(running_group_movement)
                .with_system(spawn_enemy_waves)
                // .with_system(create_loot)
                // .with_system(player_loot_stuff)
                .with_system(axe_head_kill_ennemies)
                .with_system(rotate_axe_head)
                .with_system(change_player_color)
                // .with_system(player_loot_gems)
                // .with_system(ennemies_repulsion)
                .with_system(gems_player_attraction),
        )
        .run();
}

fn setup(mut commands: Commands) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.scale = 1. / 50.;
    commands.spawn_bundle(camera_bundle);
    commands.insert_resource(LootAllGemsFor(Timer::from_seconds(0., false)));

    // Setup enemy waves
    commands.spawn_bundle(EnemyWaveBundle {
        kind: EnemyKind::BlueFish,
        timer: Timer::from_seconds(3., false),
        size: EnemyWaveSize(40),
        count: EnemyWavesCount(2),
        movement_kind: MovementKind::Tracking,
    });

    commands.spawn_bundle(EnemyWaveBundle {
        kind: EnemyKind::Pumpkin,
        timer: Timer::from_seconds(10., true),
        size: EnemyWaveSize(10),
        count: EnemyWavesCount(3),
        movement_kind: MovementKind::SlowWalking,
    });

    commands.spawn_bundle(EnemyWaveBundle {
        kind: EnemyKind::SkeletonHead,
        timer: Timer::from_seconds(15., false),
        size: EnemyWaveSize(30),
        count: EnemyWavesCount(1),
        movement_kind: MovementKind::RunningGroup,
    });

    commands.spawn_bundle(EnemyWaveBundle {
        kind: EnemyKind::BigRedFish,
        timer: Timer::from_seconds(25., false),
        size: EnemyWaveSize(30),
        count: EnemyWavesCount(2),
        movement_kind: MovementKind::Tracking,
    });

    commands.spawn_bundle(EnemyWaveBundle {
        kind: EnemyKind::Knife,
        timer: Timer::from_seconds(35., false),
        size: EnemyWaveSize(40),
        count: EnemyWavesCount(2),
        movement_kind: MovementKind::RunningGroup,
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
        .insert(RigidBody::Dynamic)
        .insert(CollisionShape::Cuboid { half_extends: Vec3::splat(0.5), border_radius: None })
        .insert(RotationConstraints::lock())
        .insert(
            CollisionLayers::none()
                .with_group(GameLayer::Player)
                .with_masks(&[GameLayer::Loot, GameLayer::Enemies]),
        )
        .insert(Player::default())
        .with_children(|commands| {
            commands
                .spawn_bundle(SpriteBundle {
                    transform: Transform::from_translation(Vec3::new(0., -3., 0.)),
                    sprite: Sprite {
                        color: AXE_HEAD_COLOR,
                        custom_size: Some(Vec2::new(0.8, 0.8)),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(RigidBody::Static)
                .insert(CollisionShape::Sphere { radius: 0.6 })
                .insert(
                    CollisionLayers::none()
                        .with_group(GameLayer::Weapon)
                        .with_mask(GameLayer::Enemies),
                )
                .insert(RotationRadian(0.))
                .insert(AxeHead);
        });
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

fn rotate_axe_head(
    time: Res<Time>,
    mut axe_head_query: Query<
        (&mut Transform, &mut RotationRadian),
        (With<AxeHead>, Without<Player>),
    >,
) {
    let (mut transform, mut rotation) = match axe_head_query.iter_mut().next() {
        Some(transform) => transform,
        None => return,
    };

    let radian = rotation.0 + AXE_HEAD_SPEED * time.delta_seconds();
    rotation.0 = if radian >= 2. * PI { 0. } else { radian };

    let x = rotation.0.cos() * 3.;
    let y = rotation.0.sin() * 3.;
    transform.translation = Vec3::new(x, y, 0.);
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

// fn change_player_color(
//     mut player_query: Query<(&mut Sprite, &CollisionShape), With<Player>>,
//     ennemies_query: Query<&CollisionShape, With<Enemy>>,
// ) {
//     let (mut player_sprite, player_shape) = match player_query.iter_mut().next() {
//         Some(value) => value,
//         None => return,
//     };

//     if ennemies_query.iter().any(|shape| shape.is_collided_with(player_shape)) {
//         player_sprite.color = HIT_PLAYER_COLOR;
//     } else {
//         player_sprite.color = HEALTHY_PLAYER_COLOR;
//     }
// }

// fn player_loot_gems(
//     mut commands: Commands,
//     mut player_query: Query<(&mut Player, &CollisionShape)>,
//     gems_query: Query<(Entity, &CollisionShape), With<Gem>>,
// ) {
//     let (mut player, player_shape) = match player_query.iter_mut().next() {
//         Some(value) => value,
//         None => return,
//     };

//     for (gem_entity, shape) in gems_query.iter() {
//         if shape.is_collided_with(player_shape) {
//             player.xp += 1;
//             commands.entity(gem_entity).despawn();
//         }
//     }
// }

// fn create_loot(
//     mut commands: Commands,
//     iconset_assets: Res<IconsetAssets>,
//     mut player_query: Query<(&mut Player, &Transform)>,
// ) {
//     let (mut player, transform) = match player_query.iter_mut().next() {
//         Some(value) => value,
//         None => return,
//     };

//     let mut rng = rand::thread_rng();
//     if !player.lvl1_stuff_generated && player.xp >= 30 {
//         player.lvl1_stuff_generated = true;
//         let pos = random_in_radius(&mut rng, transform.translation, 5.);
//         let pos = move_from_deadzone(pos, 3.).extend(95.);

//         commands
//             .spawn_bundle(SpriteSheetBundle {
//                 transform: Transform::from_translation(pos).with_scale(Vec3::splat(0.04)),
//                 sprite: TextureAtlasSprite::new(864),
//                 texture_atlas: iconset_assets.iconset_fantasy_castshadows.clone(),
//                 ..Default::default()
//             })
//             .insert(CollisionShape::new_rectangle(2., 2.))
//             .insert(Stuff::FishingRod);
//     }
// }

// fn player_loot_stuff(
//     mut commands: Commands,
//     mut loot_all_gems: ResMut<LootAllGemsFor>,
//     mut player_query: Query<(&mut Player, &CollisionShape)>,
//     stuff_query: Query<(Entity, &Stuff, &CollisionShape)>,
// ) {
//     let (_player, player_shape) = match player_query.iter_mut().next() {
//         Some(value) => value,
//         None => return,
//     };

//     for (stuff_entity, stuff, shape) in stuff_query.iter() {
//         if shape.is_collided_with(player_shape) {
//             match stuff {
//                 Stuff::FishingRod => {
//                     *loot_all_gems = LootAllGemsFor(Timer::from_seconds(2., false))
//                 }
//             }
//             commands.entity(stuff_entity).despawn();
//         }
//     }
// }

fn axe_head_kill_ennemies(
    mut commands: Commands,
    iconset_assets: Res<IconsetAssets>,
    mut events: EventReader<CollisionEvent>,
    ennemies_query: Query<&Transform>,
) {
    events
        .iter()
        .filter(|e| e.is_started())
        .filter_map(|event| {
            let (entity_1, entity_2) = event.rigid_body_entities();
            let (layers_1, layers_2) = event.collision_layers();
            if is_weapon(layers_1) && is_enemy(layers_2) {
                Some(entity_2)
            } else if is_weapon(layers_2) && is_enemy(layers_1) {
                Some(entity_1)
            } else {
                None
            }
        })
        .for_each(|enemy_entity| {
            if let Ok(transform) = ennemies_query.get_component::<Transform>(enemy_entity) {
                let pos = transform.translation.xy().extend(80.0);
                commands.entity(enemy_entity).despawn();
                commands
                    .spawn_bundle(SpriteSheetBundle {
                        transform: Transform::from_translation(pos).with_scale(Vec3::splat(0.015)),
                        sprite: TextureAtlasSprite::new(474), // blue diamond
                        texture_atlas: iconset_assets.iconset_fantasy_standalone.clone(),
                        ..Default::default()
                    })
                    .insert(MoveToPlayer::default())
                    .insert(Gem);
            }
        });
}

// Note: We check both layers each time to avoid a false-positive
// that can occur if an entity has the default (unconfigured) `CollisionLayers`
fn is_weapon(layers: CollisionLayers) -> bool {
    layers.contains_group(GameLayer::Weapon) && !layers.contains_group(GameLayer::Enemies)
}

fn is_enemy(layers: CollisionLayers) -> bool {
    !layers.contains_group(GameLayer::Player) && layers.contains_group(GameLayer::Enemies)
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
pub struct MoveToPlayer(bool);

#[derive(Component)]
pub struct Health(pub usize);

#[derive(PhysicsLayer)]
pub enum GameLayer {
    Player,
    Weapon,
    Enemies,
    Loot,
}
