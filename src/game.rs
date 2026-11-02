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
        .add_systems(Update, game.run_if(in_state(GameState::Game)));
}

// Tag component used to tag entities added on the game screen
#[derive(Component)]
struct OnGameScreen;

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

// Tick the timer, and change state when finished
fn game(time: Res<Time>, mut game_state: ResMut<NextState<GameState>>) {}

/// set up a simple 3D scene
fn scene() -> impl SceneList {
    bsn_list! {
        #CircularBase
        DespawnOnExit::<GameState>(GameState::Game)
        Mesh3d(asset_value(Circle::new(4.0)))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::WHITE))
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
        --
        #Cube
        DespawnOnExit::<GameState>(GameState::Game)
        Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
        Transform::from_xyz(0.0, 0.5, 0.0)
        --
        PointLight {
            shadow_maps_enabled: true,
        }
        DespawnOnExit::<GameState>(GameState::Game)
        Transform::from_xyz(4.0, 8.0, 4.0)
        --
        Camera3d
        DespawnOnExit::<GameState>(GameState::Game)
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y)
    }
}
