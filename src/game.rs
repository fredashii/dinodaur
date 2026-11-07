use bevy::{
    color::palettes::basic::{BLUE, LIME},
    image::{
        CompressedImageFormats, ImageAddressMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    light::Skybox,
    math::Affine2,
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
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
        .add_systems(Update, asset_loaded.run_if(in_state(GameState::Game)))
        .add_systems(Update, keyboard_input.run_if(in_state(GameState::Game)));
}

// Tag component used to tag entities added on the game screen
#[derive(Component)]
struct OnGameScreen;

#[derive(Component, Clone, Default)]
struct BoxMarker;

#[derive(Component, Clone, Default)]
struct Speed(f32);

const CUBEMAPS: &[(&str, CompressedImageFormats)] =
    &[("textures/cubemap.png", CompressedImageFormats::NONE)];

const ROAD_WIDTH: f32 = 6.0;
const ROAD_LENGTH: f32 = 10000.0;

fn move_box(mut query: Query<(&mut Transform, &Speed), With<BoxMarker>>, time: Res<Time>) {
    for (mut transform, speed) in &mut query {
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
    mut query: Query<(&mut Transform, &Speed), With<BoxMarker>>,
    time: Res<Time>,
) {
    let (mut box_transform, speed) = query.single_mut().expect("There should only be 1 box!!!!");

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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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
        uv_transform: Affine2::from_scale(Vec2::new(1.0, ROAD_LENGTH/ROAD_WIDTH)),
        ..default()
    });

    let mesh_handle = meshes.add(Rectangle::new(ROAD_WIDTH, ROAD_LENGTH));

    commands.spawn((
        DespawnOnExit(GameState::Game),
        Mesh3d(mesh_handle),
        MeshMaterial3d(material_handle),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
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

    commands.spawn_scene_list(scene(asset_server));
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
fn scene(asset_server: Res<AssetServer>) -> impl SceneList {
    bsn_list! {
            // #Track
            // DespawnOnExit::<GameState>(GameState::Game)
            // Mesh3d(asset_value(Rectangle::from_corners(Vec2::new(-3.0, 1.0), Vec2::new(3.0, 10000.0))))
            // MeshMaterial3d::<StandardMaterial>(asset_value(Color::WHITE))
            // Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            // --
            #Cube
            BoxMarker
            DespawnOnExit::<GameState>(GameState::Game)
            Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
            Transform::from_xyz(0.0, 0.5, 0.0)
            Speed(5.0)

            --
            // PointLight {
            //     shadow_maps_enabled: true,
            //     intensity: 1000000.0,
            // }
            // DespawnOnExit::<GameState>(GameState::Game)
            // Transform::from_xyz(4.0, 8.0, 4.0)
    // directional 'sun' light

            DirectionalLight {
                illuminance: light_consts::lux::OVERCAST_DAY,
                shadow_maps_enabled: true,

            }
            Transform::from_xyz(2.0, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y)
            --
        }
}

fn asset_loaded(
    mut images: ResMut<Assets<Image>>,
    skyboxes: Query<&mut Skybox>,
) {
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
