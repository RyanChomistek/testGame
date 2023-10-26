use bevy::{
    prelude::*,
    // sprite::collide_aabb::{collide, Collision},
    sprite::MaterialMesh2dBundle,
};

use bevy::prelude::*;
use bevy_rand::prelude::*;
use rand_core::RngCore;
use bevy_prng::ChaCha8Rng;

const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const BALL_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const PADDLE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
const SUN_COLOR: Color = Color::rgb(0.3, 0.3, 0.0);

const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, 0.0, 1.0);
const BALL_SIZE: Vec3 = Vec3::new(30.0, 30.0, 0.0);
const PADDLE_SIZE: Vec3 = Vec3::new(120.0, 20.0, 0.0);

const GRAVITY_CONSTANT: f32 = 1.0;


#[derive(Component)]
struct Player;

#[derive(Component)]
struct GameCamera;

#[derive(Component)]
struct Body {
    pub vel: Vec3,
    pub mass: f32,
}

fn setup(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>
) {
    // Camera
    commands.spawn((Camera2dBundle::default(), GameCamera));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(BALL_COLOR)),
            transform: Transform::from_translation(BALL_STARTING_POSITION).with_scale(BALL_SIZE),
            ..default()
        },
        Player,
        //Body{vel: Vec3::ZERO, mass: 1.0}
    ));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(SUN_COLOR)),
            transform:  Transform::from_translation(Vec3::ZERO).with_scale(Vec3::new(200.0, 200.0, 0.0)),
            ..default()
        },
        Body{vel: Vec3::ZERO, mass: 100000.0}
    ));

    for _ in 0..10
    {
        let mass = (rng.next_u32() as i32 % 200 + 15) as f32;

        let transform = Transform::from_translation(Vec3::new(
            (rng.next_u32() as i32 % 500).abs() as f32,
            (rng.next_u32() as i32 % 500).abs() as f32,
            1.0)).with_scale(Vec3::new(mass, mass, 0.0));

        let vel_scale = 100;
        let vel = Vec3::new(
            (rng.next_u32() as i32 % vel_scale) as f32,
            (rng.next_u32() as i32 % vel_scale) as f32,
            0.0);

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::default().into()).into(),
                material: materials.add(ColorMaterial::from(PADDLE_COLOR)),
                transform: transform,
                ..default()
            },
            Body{vel: vel, mass: mass}
        ));
    }
}

fn move_player(
    keyboard_input: Res<Input<KeyCode>>, 
    mut query: Query<&mut Transform, With<Player>>,
    time_step: Res<FixedTime>,)
{
    let mut player_transform = query.single_mut();
    let mut direction = Vec3::new(0.0, 0.0, 0.0);

    if keyboard_input.pressed(KeyCode::Left) {
        direction.x = -1.0;
    }
    else if keyboard_input.pressed(KeyCode::Right) {
        direction.x = 1.0;
    }

    if keyboard_input.pressed(KeyCode::Up) {
        direction.y = 1.0;
    }
    else if keyboard_input.pressed(KeyCode::Down) {
        direction.y = -1.0;
    }

    player_transform.translation = player_transform.translation + (direction * 100.0 * time_step.period.as_secs_f32());
    // println!("{} {} {}", player_transform.translation.x, player_transform.translation.y, player_transform.translation.z);
}

fn camera_chase(
    playerQuery: Query<&Transform, With<Player>>,
    mut cameraQuery: Query<&mut Transform, (Without<Player>, With<GameCamera>)>,
)
{
    let player_transform = playerQuery.single();
    let mut camera_transform = cameraQuery.single_mut();

    camera_transform.translation = player_transform.translation;
}



fn body_interactions(
    mut currentBodyQuery: Query<(&Transform, &mut Body)>,
    time_step: Res<FixedTime>,
)
{
    // TODO I think this loop is going over every object to every object, so some iterations are self to self causing the calc to get weird
    let mut iter = currentBodyQuery.iter_combinations_mut();
    while let Some([(transform, mut body), (transform_other, mut body_other)]) = iter.fetch_next() {
        let mut delta =  transform_other.translation - transform.translation ;
        delta.z = 0.0;
        let distance_sq = delta.length_squared();

        let f1 = (GRAVITY_CONSTANT * body_other.mass) / distance_sq;
        let a1 = delta.normalize() * f1;
        body.vel += a1 * time_step.period.as_secs_f32();

        let f2 = (GRAVITY_CONSTANT * body.mass) / distance_sq;
        let a2 = -1.0 * delta.normalize() * f2;
        body_other.vel += a2 * time_step.period.as_secs_f32();

        println!("{} {} {} {} {}", distance_sq, body.mass, body_other.mass, a1.length(), a2.length());
    }
}

fn move_bodies(
    mut currentBodyQuery: Query<(&mut Transform, &Body)>,
    time_step: Res<FixedTime>,
) {
    for (mut transform, body) in currentBodyQuery.iter_mut()
    {
        transform.translation += body.vel * time_step.period.as_secs_f32();
    }
}

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(EntropyPlugin::<ChaCha8Rng>::default())
    .insert_resource(FixedTime::new_from_secs(1.0 / 60.0))
    .insert_resource(ClearColor(BACKGROUND_COLOR))
    .add_systems(Startup, setup)
    .add_systems(Update, (move_player, camera_chase, move_bodies, body_interactions))
    .add_systems(Update, bevy::window::close_on_esc)
    .run();
}
