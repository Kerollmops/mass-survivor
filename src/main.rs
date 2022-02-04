use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy_ggrs::*;
use ggrs::PlayerType;
use matchbox_socket::WebRtcNonBlockingSocket;

use self::components::Player;
use self::input::{direction, input, INPUT_SIZE};

mod components;
mod input;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.53, 0.53, 0.53)))
        .add_plugins(DefaultPlugins)
        .add_plugin(GGRSPlugin)
        .add_startup_system(setup)
        .add_startup_system(spawn_player)
        .add_startup_system(start_matchbox_socket)
        .add_system(wait_for_players)
        .add_system(camera_follow)
        .with_input_system(input)
        .with_rollback_schedule(
            Schedule::default().with_stage(
                "ROLLBACK_STAGE",
                SystemStage::single_threaded().with_system(move_players),
            ),
        )
        .register_rollback_type::<Transform>()
        .run();
}

fn setup(mut commands: Commands) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.scale = 1. / 50.;
    commands.spawn_bundle(camera_bundle);
}

fn start_matchbox_socket(mut commands: Commands, task_pool: Res<IoTaskPool>) {
    let room_url = "ws://127.0.0.1:3536/next_2";
    info!("connecting to matchbox server: {:?}", room_url);
    let (socket, message_loop) = WebRtcNonBlockingSocket::new(room_url);

    // The message loop needs to be awaited, or nothing will happen.
    // We do this here using bevy's task system.
    task_pool.spawn(message_loop).detach();

    commands.insert_resource(Some(socket));
}

struct LocalPlayerHandle(usize);

fn wait_for_players(mut commands: Commands, mut socket: ResMut<Option<WebRtcNonBlockingSocket>>) {
    let socket = socket.as_mut();

    // If there is no socket we've already started the game
    if socket.is_none() {
        return;
    }

    // Check for new connections
    socket.as_mut().unwrap().accept_new_connections();
    let players = socket.as_ref().unwrap().players();

    let num_players = 2;
    if players.len() < num_players {
        return; // wait for more players
    }

    info!("All peers have joined, going in-game");

    // consume the socket (currently required because GGRS takes ownership of its socket)
    let socket = socket.take().unwrap();

    let max_prediction = 12;

    // create a GGRS P2P session
    let mut p2p_session =
        ggrs::P2PSession::new_with_socket(num_players as u32, INPUT_SIZE, max_prediction, socket);

    for (i, player) in players.into_iter().enumerate() {
        p2p_session.add_player(player, i).expect("failed to add player");

        if player == PlayerType::Local {
            // set input delay for the local player
            p2p_session.set_frame_delay(2, i).unwrap();
            commands.insert_resource(LocalPlayerHandle(i));
        }
    }

    // start the GGRS session
    commands.start_p2p_session(p2p_session);
}

fn spawn_player(mut commands: Commands, mut rip: ResMut<RollbackIdProvider>) {
    // Player 1
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(-2., 0., 0.)),
            sprite: Sprite {
                color: Color::rgb(0., 0.47, 1.),
                custom_size: Some(Vec2::new(1., 1.)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Player { handle: 0 })
        .insert(Rollback::new(rip.next_id()));

    // Player 2
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(2., 0., 0.)),
            sprite: Sprite {
                color: Color::rgb(0., 0.4, 0.),
                custom_size: Some(Vec2::new(1., 1.)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Player { handle: 1 })
        .insert(Rollback::new(rip.next_id()));
}

fn move_players(
    inputs: Res<Vec<ggrs::GameInput>>,
    mut player_query: Query<(&mut Transform, &Player)>,
) {
    for (mut transform, player) in player_query.iter_mut() {
        let direction = direction(&inputs[player.handle]);

        if direction == Vec2::ZERO {
            continue;
        }

        let move_speed = 0.13;
        let move_delta = (direction * move_speed).extend(0.);

        transform.translation += move_delta;
    }
}

fn camera_follow(
    player_handle: Option<Res<LocalPlayerHandle>>,
    player_query: Query<(&Player, &Transform)>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    let player_handle = match player_handle {
        Some(handle) => handle.0,
        None => return, // Session hasn't started yet
    };

    for (player, player_transform) in player_query.iter() {
        if player.handle != player_handle {
            continue;
        }

        let pos = player_transform.translation;

        for mut transform in camera_query.iter_mut() {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        }
    }
}
