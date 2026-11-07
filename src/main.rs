use avian3d::{PhysicsPlugins, debug_render::PhysicsDebugPlugin};
use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

mod game;
mod menu;
mod splash;

const TEXT_COLOR: Color = Color::srgb_u8(47, 56, 42);

// Enum that will be used as a global state for the game
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Splash,
    Menu,
    Game,
}

// One of the two settings that can be set through the menu. It will be a resource in the app
#[derive(Resource, Debug, PartialEq, Eq, Clone, Copy)]
enum DisplayQuality {
    Low,
    Medium,
    High,
}

#[derive(Component)]
struct Setting<T>(T);

// One of the two settings that can be set through the menu. It will be a resource in the app
#[derive(Resource, Debug, PartialEq, Eq, Clone, Copy)]
struct Volume(u32);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((PhysicsPlugins::default(), PhysicsDebugPlugin))
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        // Insert as resource the initial value for the settings resources
        .insert_resource(DisplayQuality::Medium)
        .insert_resource(Volume(7))
        // Declare the game state, whose starting value is determined by the `Default` trait
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        // Adds the plugins for each state
        .add_plugins((splash::splash_plugin, menu::menu_plugin, game::game_plugin))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
