use bevy::{
    // `Skybox`: the component on the camera that holds the cubemap.
    light::Skybox,
    prelude::*,
};

// `super` is the crate root (main.rs), where the `GameState` enum lives.
use super::GameState;

const SKYBOX_ROTATION_SPEED: f32 = 0.01;

pub fn galaxy_plugin(app: &mut App) {
    app.add_systems(Update, rotate_skybox.run_if(in_state(GameState::Game)));
}

fn rotate_skybox(mut skyboxes: Query<&mut Skybox>, time: Res<Time>) {
    for mut skybox in &mut skyboxes {
        skybox.rotation = Quat::from_rotation_y(time.elapsed_secs() * SKYBOX_ROTATION_SPEED);
    }
}
