use std::mem;

use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};
use rand::Rng;

const MAP_SIZE: u32 = 41;
const GRID_WIDTH: f32 = 0.05;
const SLOW_DOWN: f32 = 0.95;
const PLAYER_SPEED: f32 = 0.5;
const ENNEMIES_SPEED: f32 = 0.1;

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
        .add_system_set(SystemSet::on_update(MyStates::Next).with_system(camera_follow))
        .add_system_set(SystemSet::on_update(MyStates::Next).with_system(spawn_ennemies))
        .add_system_set(SystemSet::on_update(MyStates::Next).with_system(move_ennemies))
        .add_system_set(SystemSet::on_update(MyStates::Next).with_system(ennemies_repulsion))
        .add_system_set(SystemSet::on_update(MyStates::Next).with_system(move_player))
        .add_system_set(SystemSet::on_update(MyStates::Next).with_system(apply_velocity))
        .run();
}

fn setup(mut commands: Commands) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.scale = 1. / 50.;
    commands.spawn_bundle(camera_bundle);
    commands.insert_resource(EnnemyWaveSpawned(false));

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
            transform: Transform::from_translation(Vec3::new(-2., 0., 100.)),
            sprite: Sprite {
                color: Color::rgb(0., 0.47, 1.),
                custom_size: Some(Vec2::new(1., 1.)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Velocity::default())
        .insert(Player);
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
        velocity.0 *= SLOW_DOWN;
    }
}

fn gen_in_radius<R: Rng>(rng: &mut R, center: Vec3, radius: f32, deadzone: f32) -> Vec2 {
    use std::f32::consts::PI;
    let [x0, y0, _] = center.to_array();
    let t = 2.0 * PI * rng.gen_range(0.0..=1.0);
    let r = radius * rng.gen_range(0.0..=1.0f32).sqrt() + deadzone;
    let x = x0 + r * t.cos();
    let y = y0 + r * t.sin();
    Vec2::new(x, y)
}

fn spawn_ennemies(
    time: Res<Time>,
    iconset_assets: Res<IconsetAssets>,
    mut ennemy_wave_spawned: ResMut<EnnemyWaveSpawned>,
    player_query: Query<&Transform, With<Player>>,
    mut commands: Commands,
) {
    match time.time_since_startup().as_secs() {
        2 if !ennemy_wave_spawned.spawned() => {
            let mut rng = rand::thread_rng();
            for player_transform in player_query.iter() {
                for i in 0..40 {
                    let pos = gen_in_radius(&mut rng, player_transform.translation, 10.0, 2.0)
                        .extend(90.0);

                    commands
                        .spawn_bundle(SpriteSheetBundle {
                            transform: Transform::from_translation(pos)
                                .with_scale(Vec3::splat(0.02)),
                            sprite: TextureAtlasSprite::new(162),
                            texture_atlas: iconset_assets.iconset_standalone.clone(),
                            ..Default::default()
                        })
                        .insert(Velocity::default())
                        .insert(Ennemy);
                }
            }
        }
        _ => (),
    }
}

fn move_ennemies(
    player_query: Query<&Transform, With<Player>>,
    mut ennemies_query: Query<(&mut Velocity, &Transform), (With<Ennemy>, Without<Player>)>,
) {
    let player_transform = match player_query.iter().next() {
        Some(transform) => transform,
        None => return,
    };

    for (mut velocity, transform) in ennemies_query.iter_mut() {
        let player = player_transform.translation.xy();
        let ennemy = transform.translation.xy();
        let direction = player - ennemy;
        let strenght = player.distance(ennemy) / 10.0;
        velocity.0 += direction * strenght * ENNEMIES_SPEED;
    }
}

fn ennemies_repulsion(mut ennemies_query: Query<(&mut Velocity, &Transform, &Ennemy)>) {
    let mut combinations = ennemies_query.iter_combinations_mut();
    while let Some([(mut avel, atransf, _), (bvel, btransf, _)]) = combinations.fetch_next() {
        let dist = atransf.translation.distance(btransf.translation);
        if dist <= 2.0 {
            let strenght = 2.0 - dist;
            let dir = (btransf.translation - atransf.translation).normalize_or_zero().xy();
            avel.0 -= dir * (strenght / 50.0);
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

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Ennemy;

#[derive(Default, Component)]
pub struct Velocity(Vec2);

pub struct EnnemyWaveSpawned(bool);

impl EnnemyWaveSpawned {
    /// Set it to `true` and returns the previous value.
    pub fn spawned(&mut self) -> bool {
        let EnnemyWaveSpawned(old) = self;
        mem::replace(old, true)
    }
}

// fish 162-165
#[derive(AssetCollection)]
struct IconsetAssets {
    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 32., columns = 18, rows = 31))]
    #[asset(path = "images/iconset_standalone.png")]
    iconset_standalone: Handle<TextureAtlas>,
}
