use bevy::{
    color::palettes::basic::{BLUE, LIME},
    prelude::*,
    ui_widgets::{Activate, ActivateOnPress, Button, RadioButton, RadioGroup, ValueChange},
};

use crate::menu;

use super::{DisplayQuality, GameState, TEXT_COLOR, Volume};

// This plugin will contain the game. In this case, it's just be a screen that will
// display the current settings for 5 seconds before returning to the menu
pub fn game_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Game), game_setup)
        .add_systems(Update, move_box.run_if(in_state(GameState::Game)))
        .add_systems(Update, move_camera.run_if(in_state(GameState::Game)))
        .add_systems(Update, keyboard_input.run_if(in_state(GameState::Game)))
        .insert_resource(Speed(5.0));
}

// Tag component used to tag entities added on the game screen
#[derive(Component)]
struct OnGameScreen;

#[derive(Component, Clone, Default)]
struct BoxMarker;

#[derive(Resource)]
struct Speed(f32);

fn move_box(mut query: Query<&mut Transform, With<BoxMarker>>, time: Res<Time>, speed: Res<Speed>) {
    for mut transform in &mut query {
        transform.translation.z -= speed.0 * time.delta_secs();
    }
}

fn move_camera(
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    box_query: Query<&Transform, (With<BoxMarker>, Without<Camera3d>)>,
) {
    let mut camera_transform = camera_query
        .single_mut()
        .expect("There should only be 1 camera!!!!");
    let box_transform = box_query.single().expect("There should only be 1 box!!!!");

    let offset = box_transform.translation;
    camera_transform.translation = Vec3::new(0.0, 4.5, 9.0) + offset;
}

fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<BoxMarker>>,
    time: Res<Time>,
    speed: Res<Speed>,
) {
    let mut box_transform = query.single_mut().expect("There should only be 1 box!!!!");

    if keys.pressed(KeyCode::KeyA) {
        box_transform.translation.x -= speed.0 * time.delta_secs();
    }
    if keys.pressed(KeyCode::KeyD) {
        box_transform.translation.x += speed.0 * time.delta_secs();
    }
}

fn game_setup(
    mut commands: Commands,
    display_quality: Res<DisplayQuality>,
    volume: Res<Volume>,
    asset_server: Res<AssetServer>,
) {
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

    commands.spawn_scene_list(scene());
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
        #Track
        DespawnOnExit::<GameState>(GameState::Game)
        Mesh3d(asset_value(Rectangle::from_corners(Vec2::new(-3.0, 1.0), Vec2::new(3.0, 10000.0))))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::WHITE))
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
        --
        #Cube
        BoxMarker
        DespawnOnExit::<GameState>(GameState::Game)
        Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
        Transform::from_xyz(0.0, 0.5, 0.0)
        --
        PointLight {
            shadow_maps_enabled: true,
            intensity: 1000000.0,
        }
        DespawnOnExit::<GameState>(GameState::Game)
        Transform::from_xyz(4.0, 8.0, 4.0)
        --
        Camera3d
        DespawnOnExit::<GameState>(GameState::Game)
        Transform::from_xyz(0.0, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y)
    }
}
