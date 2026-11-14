use std::f32::consts::PI;

use bevy::{
    image::{
        CompressedImageFormats, ImageAddressMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    input::mouse::AccumulatedMouseMotion,
    light::Skybox,
    math::Affine2,
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
    ui_widgets::{ActivateOnPress, Button},
    window::CursorOptions,
};

use crate::menu;
use avian3d::prelude::*;

use super::{GamePhase, GameState, TEXT_COLOR};

// This plugin will contain the game. In this case, it's just be a screen that will
// display the current settings for 5 seconds before returning to the menu
pub fn game_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Game), game_setup)
        .add_systems(
            Update,
            mouse_click_system.run_if(in_state(GamePhase::PreGame)),
        )
        .add_systems(Update, pause_game.run_if(in_state(GamePhase::InGame)))
        .add_systems(Update, resume_game.run_if(in_state(GamePhase::PauseGame)))
        .add_systems(Update, update_player.run_if(in_state(GamePhase::InGame)))
        .add_systems(
            Update,
            camera_follows_player.run_if(in_state(GamePhase::InGame)),
        )
        .add_systems(Update, asset_loaded.run_if(in_state(GameState::Game)))
        .add_systems(Update, get_collisions.run_if(in_state(GamePhase::InGame)))
        .add_systems(
            Update,
            start_player_run_animation.run_if(in_state(GamePhase::InGame)),
        )
        .add_systems(Update, update_animation.run_if(in_state(GamePhase::InGame)))
        .add_systems(
            Update,
            handle_mouse_motion.run_if(in_state(GamePhase::InGame)),
        );
}

// Tag component used to tag entities added on the game screen
#[derive(Component)]
struct OnGameScreen;

#[derive(Component, Clone, Default)]
struct Player {
    time: f32,
    health: f32,
    food_counter: i32,
}

#[derive(Component, Clone, Default)]
struct RampMarker;

#[derive(Component, Clone, Default)]
struct FoodMarker;


#[derive(Component, Clone, Default)]
struct EnemyMarker;

const CUBEMAPS: &[(&str, CompressedImageFormats)] =
    &[("textures/cubemap.png", CompressedImageFormats::NONE)];

const ROAD_WIDTH: f32 = 10.0;
const ROAD_LENGTH: f32 = 10000.0;

const ACCELERATION: f32 = 5.0;
const DEACCELERATION: f32 = 1000.0;

const ANIMATION_RUN: usize = 0;
const ANIMATION_BITE: usize = 1;
const ANIMATION_ROAR: usize = 2;
const ANIMATION_ATTACK: usize = 3;
const ANIMATION_IDLE: usize = 4;

fn pause_game(
    input: Res<ButtonInput<KeyCode>>,
    mut game_phase: ResMut<NextState<GamePhase>>,
    mut animation_players: Query<&mut AnimationPlayer>,
    mut time: ResMut<Time<Physics>>,
    mut cursor_options: Single<&mut CursorOptions>,
    state: Res<State<GamePhase>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        game_phase.set(GamePhase::PauseGame);
        time.pause(); // Stop avian physics. 
        for mut animation_player in &mut animation_players {
            for (_, animation) in animation_player.playing_animations_mut() {
                animation.pause();
            }
        }
        cursor_options.visible = true;
        cursor_options.grab_mode = bevy::window::CursorGrabMode::None;
    }
}

fn resume_game(
    input: Res<ButtonInput<KeyCode>>,
    mut game_phase: ResMut<NextState<GamePhase>>,
    mut animation_players: Query<&mut AnimationPlayer>,
    mut time: ResMut<Time<Physics>>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    if input.just_pressed(KeyCode::Escape) {
        game_phase.set(GamePhase::InGame);
        time.unpause(); // Resume avian physics.
        for mut animation_player in &mut animation_players {
            for (_, animation) in animation_player.playing_animations_mut() {
                animation.resume();
            }
        }
        cursor_options.visible = false;
        cursor_options.grab_mode = bevy::window::CursorGrabMode::Locked;
    }
}

fn mouse_click_system(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut game_phase: ResMut<NextState<GamePhase>>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        info!("Left mouse button was just pressed!");
        game_phase.set(GamePhase::InGame)
    }
}

fn start_player_run_animation(
    mut commands: Commands,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    for (enity, mut player) in &mut players {
        let (graph, animation_to_play) = AnimationGraph::from_clip(
            asset_server
                .load(GltfAssetLabel::Animation(ANIMATION_RUN).from_asset("models/dinosaur.glb")),
        );

        commands
            .entity(enity)
            .insert(AnimationGraphHandle(graphs.add(graph)));

        player.play(animation_to_play).repeat().set_speed(1.0);
    }
}

fn get_collisions(
    mut query: Query<(&CollidingEntities, &mut Player), With<Player>>,
    ramps: Query<Entity, With<RampMarker>>,
    enemies: Query<Entity, With<EnemyMarker>>,
    food: Query<Entity, With<FoodMarker>>,
    mut commands: Commands,
) {
    for (colliding_entities, mut player) in &mut query {
        for entity in colliding_entities.iter() {
            if ramps.get(*entity).is_ok() {
                player.time = 5.0;
            } else if enemies.get(*entity).is_ok() {
                player.health = 0.0;
            } else if food.get(*entity).is_ok(){
                player.food_counter += 1;
                println!("player food {}", player.food_counter);
                commands.entity(*entity).despawn();
            }
        }
    }
}

fn update_player(
    mut query: Query<(&mut LinearVelocity, &mut Player), With<Player>>,
    time: Res<Time>,
) {
    for (mut velocity, mut player) in &mut query {
        if player.health <= 0.0 {
            velocity.z = 0.0;
        } else {
            if velocity.y < -5.0 {
                velocity.y = -5.0;
            }
            if player.time == 0.0 {
                if velocity.z >= -20.0 {
                    velocity.z -= ACCELERATION * time.delta_secs();
                    if velocity.z < -20.0 {
                        velocity.z = -20.0
                    }
                } else {
                    velocity.z += DEACCELERATION * time.delta_secs();
                    if velocity.z > -20.0 {
                        velocity.z = -20.0
                    }
                }
            } else if player.time > 0.0 {
                velocity.z -= 200.0 * time.delta_secs();
                player.time -= 1.0 * time.delta_secs();
            } else if player.time < 0.0 {
                player.time = 0.0;
            }
        }
    }
}

fn update_animation(
    mut query: Query<(&mut LinearVelocity, &mut Player), With<Player>>,
    mut animation_players: Query<&mut AnimationPlayer>,
) {
    for (velocity, player) in &mut query {
        let mut animation_speed = velocity.z.abs() * 0.1;
        if animation_speed > 4.0 {
            animation_speed = 4.0;
        }
        if animation_speed < 1.0 {
            animation_speed = 1.0;
        }

        for mut animation_player in &mut animation_players {
            for (_, animation) in animation_player.playing_animations_mut() {
                animation.set_speed(animation_speed);
                if player.health <= 0.0 {
                    animation.set_speed(0.0);
                }
            }
        }
    }
}

fn camera_follows_player(
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    box_query: Query<&Transform, (With<Player>, Without<Camera3d>)>,
) {
    let mut camera_transform = camera_query
        .single_mut()
        .expect("There should only be 1 camera!!!!");
    let box_transform = box_query.single().expect("There should only be 1 box!!!!");

    let offset = box_transform.translation;
    camera_transform.translation = Vec3::new(0.0, 4.5, 9.0) + offset;

    let (yaw, pitch, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);

    let roll = box_transform.translation.x * -0.05;

    camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
}

fn handle_mouse_motion(
    accumulated_motion: Res<AccumulatedMouseMotion>,
    mut query: Query<&mut LinearVelocity, With<Player>>,
    time: Res<Time>,
) {
    let mut velocity = query.single_mut().expect("There should only be 1 box!!!!");

    let delta = accumulated_motion.delta;
    if delta != Vec2::ZERO {
        velocity.x += delta.x * 0.5;
        velocity.x = velocity.x.clamp(-10.0, 10.0)
    } else {
        if velocity.x.abs() > 0.0 {
            let sign = velocity.x.signum();
            velocity.x -= 30.0 * time.delta_secs() * sign;
            if velocity.x.signum() != sign {
                velocity.x = 0.0;
            }
        }
    }
}

fn game_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cursor_options: Single<&mut CursorOptions>,
    mut mouse_input: ResMut<ButtonInput<MouseButton>>,
    mut game_phase: ResMut<NextState<GamePhase>>,
    mut time: ResMut<Time<Physics>>,
) {
    time.unpause();

    mouse_input.clear_just_pressed(MouseButton::Left);

    commands.spawn((
        DespawnOnExit(GamePhase::PreGame),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Relative,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Text::new("Click to start!"),
            TextFont {
                font_size: FontSize::Px(40.0),
                ..default()
            }
        )],
    ));

    cursor_options.visible = false;
    cursor_options.grab_mode = bevy::window::CursorGrabMode::Locked;

    let texture_handle = asset_server
        .load_builder()
        .with_settings(|settings: &mut _| {
            *settings = ImageLoaderSettings {
                sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::MirrorRepeat,
                    address_mode_v: ImageAddressMode::MirrorRepeat,
                    ..default()
                }),
                ..default()
            };
        })
        .load("textures/road.png");

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle),
        // The texture is square, so one tile covers ROAD_WIDTH x ROAD_WIDTH world units.
        uv_transform: Affine2::from_scale(Vec2::new(1.0, ROAD_LENGTH / ROAD_WIDTH)),
        ..default()
    });

    let mesh_handle: Handle<Mesh> = meshes.add(Rectangle::new(ROAD_WIDTH, ROAD_LENGTH));

    commands.spawn((
        RigidBody::Static,
        DespawnOnExit(GameState::Game),
        // Collider::Qua(ROAD_WIDTH, 0.1, ROAD_LENGTH),
        Collider::cuboid(ROAD_WIDTH, 0.05, ROAD_LENGTH),
        Transform::from_xyz(0.0, -0.05, 0.0),
        children![(
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
            Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        )],
    ));

    commands.spawn((
        RigidBody::Static,
        DespawnOnExit(GameState::Game),
        Transform::from_xyz(0.0, 0.0, -20.0)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        Mesh3d(meshes.add(Extrusion::new(
            Triangle2d::new(
                Vec2::new(0.0, 0.0),
                Vec2::new(6.0, 0.0),
                Vec2::new(3.0, 2.0),
            ),
            2.0,
        ))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        ColliderConstructor::ConvexHullFromMesh,
        CollidingEntities::default(),
        RampMarker,
    ));

    let button_node = Node {
        width: px(300),
        height: px(65),
        margin: UiRect::all(px(20)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_font = TextFont {
        font_size: FontSize::Px(33.0),
        ..default()
    };
    let button_icon_node = Node {
        width: px(30),
        // This takes the icons out of the flexbox flow, to be positioned exactly
        position_type: PositionType::Absolute,
        // The icon will be close to the left border of the button
        left: px(10),
        ..default()
    };
    let right_icon = asset_server.load("textures/Game Icons/fairy.png");

    let cubemap_handle: Handle<Image> = asset_server.load(CUBEMAPS[0].0);

    let skybox = Skybox {
        image: Some(cubemap_handle),
        brightness: 1000.0,
        rotation: Quat::IDENTITY,
    };

    commands.spawn_scene_list(scene());
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::FRAC_PI_3, // Sets FOV to 60 degrees (in radians)
            ..default()
        }),
        DespawnOnExit::<GameState>(GameState::Game),
        Transform::from_xyz(0.0, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        skybox,
    ));

    commands.spawn((
        RigidBody::Static,
        AngularVelocity(Vec3::new(2.5, 3.5, 1.5)),
        Collider::cuboid(1.0, 1.0, 1.0),
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d::<StandardMaterial>(materials.add(Color::srgb_u8(255, 144, 255))),
        Transform::from_xyz(-3.0, 0.5, -30.0),
        EnemyMarker,
    ));

    // A potato standing upright on the road, facing the camera.
    // The image is 900x637, so keep that aspect ratio to avoid stretching it.
    let potato_width =1.5;
    let potato_height = 3.;
    commands.spawn((
        RigidBody::Static,
        Sensor, 
        Collider::cuboid(potato_width, potato_height, 0.2),
        Mesh3d(meshes.add(Rectangle::new(potato_width, potato_height))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("textures/potato.png")),
            alpha_mode: AlphaMode::Mask(0.5),
            ..default()
        })),
        Transform::from_xyz(3.0, 1.5, -30.0),
        DespawnOnExit(GameState::Game),
        FoodMarker,
    ));

    commands.spawn((
        DespawnOnExit(GameState::Game),
        Node {
            width: percent(100),
            height: percent(100),
            // center children
            // align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnGameScreen,
        children![(
            DespawnOnExit(GameState::Game),
            Button,
            ActivateOnPress,
            button_node.clone(),
            menu::MenuButtonAction::BackToMainMenu,
            children![
                (ImageNode::new(right_icon), button_icon_node.clone()),
                (
                    Text::new("Return to Main Menu"),
                    button_text_font.clone(),
                    TextColor(TEXT_COLOR),
                ),
            ],
        )],
    ));
}

/// set up a simple 3D scene
fn scene() -> impl SceneList {
    bsn_list! {
        #Cube
        RigidBody::Dynamic
        Friction{static_coefficient: 0.0, dynamic_coefficient: 0.0, combine_rule: CoefficientCombine::Min}
        Player{
            time: 0.0,
            health: 100.0,
            food_counter: 0,
        }
        DespawnOnExit::<GameState>(GameState::Game)
        Children [
            WorldAssetRoot("models/dinosaur.glb#Scene0")
            Transform::from_xyz(0.0, -0.5, 0.0)
            .with_scale(Vec3::splat(1.5))
            .with_rotation(Quat::from_rotation_y(std::f32::consts::PI) * Quat::from_rotation_x(std::f32::consts::PI/-6.0))
        ]
        Transform::from_xyz(0.0, 0.5, 0.0)
        Collider::sphere(0.5)
        LockedAxes::new().lock_rotation_x().lock_rotation_y().lock_rotation_z()
        LinearVelocity(Vec3::new(0.0, 0.0, 0.0))
        CollidingEntities::default()
        --
        #Blocker
        RigidBody::Static
        AngularVelocity(Vec3::new(2.5, 3.5, 1.5))
        Collider::cuboid(1.0, 1.0, 1.0)
        Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
        Transform::from_xyz(0.0, 0.5, -10.0)
        --
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadow_maps_enabled: true,
        }
        Transform::from_xyz(2.0, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y)
        --
    }
}

fn asset_loaded(mut images: ResMut<Assets<Image>>, skyboxes: Query<&mut Skybox>) {
    let skybox = skyboxes.single().expect("We should have a skybox");
    let handle = skybox.image.clone().unwrap();
    // Only take a mutable borrow when there's work to do: `get_mut` marks the
    // image as modified, which would re-upload it to the GPU every frame.
    // `get` returns `None` until the image has loaded.
    if images
        .get(&handle)
        .is_some_and(|image| image.texture_descriptor.array_layer_count() == 1)
    {
        let mut image = images.get_mut(&handle).unwrap();
        let layers = image.height() / image.width();
        image
            .reinterpret_stacked_2d_as_array(layers)
            .expect("asset should be 2d texture and height will always be evenly divisible with the given layers");
        image.texture_view_descriptor = Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..default()
        });
    }
}
