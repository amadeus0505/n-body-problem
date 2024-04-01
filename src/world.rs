use bevy::{input::mouse::{MouseScrollUnit, MouseWheel}, prelude::*, window::{PresentMode, PrimaryWindow}};
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use iyes_perf_ui::{diagnostics::{PerfUiEntryFPS, PerfUiEntryFPSWorst}, PerfUiRoot};



pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, (load_background, setup))
        .add_systems(Update, (camera_zoom, update_fixed_timestep))
        // register Type Planet and Game so they are editable in the WorldInspector
        .register_type::<Game>()
        // load the ResourceInspectorPlugin in order to edit resources in the WorldInspector
        // .add_plugins(ResourceInspectorPlugin::<Game>::default())
        // initialize resource Game in order to be able to edit it in the WorldInspector
        .init_resource::<Game>()
        // create resource Game with speed set to 1.0
        .insert_resource(Game{speed: 1.0, stepsize: 1.0/64.0, planet_count: 0})
        ;
    }
}

#[derive(Component)]
pub struct MyCamera;

#[derive(Component)]
struct Background;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    Paused,
    Running,
}

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct Game {
    pub speed: f32,
    pub stepsize: f32,
    pub planet_count: u32,
}


fn update_fixed_timestep(
    mut time_fixed: ResMut<Time<Fixed>>,
    mut game: ResMut<Game>
) {
    if game.is_changed() {
        time_fixed.set_timestep_hz(game.speed as f64);
        // sets the stepsize fixed, so the planets movespeed is dependent on the update rate (FixedUpdate)
        // can be changed: 0.1 is a good value for a smooth curve; if you lower it, the curve will be smoother but the displayed path will be shorter
        game.stepsize = 0.1;
    }
}


fn setup(
    // get commands to spawn entities
    mut commands: Commands,
    // get the main window for positioning [Note: mutable is nessecessary to change the window present mode]
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    // getting the asset server to loading textures
) {
    // get the Main window; unwrap is safe here because the window always exists
    let mut window = window_query.get_single_mut().unwrap();

    // disable vsync
    window.present_mode = PresentMode::AutoNoVsync;

    // spawn Camera
    commands.spawn(
        (
            Camera2dBundle {
            // spawns a camera at the center of the window
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..default()
            },
            MyCamera {}
        )
    );

    // spawn FPS Display []
    commands.spawn((
        PerfUiRoot {
            // no label means text only
            display_labels: false,
            // data is displayed horizontally
            layout_horizontal: true,
            ..default()
        },
        // add entry for the worst FPS
        PerfUiEntryFPSWorst::default(),
        // add entry for the average FPS
        PerfUiEntryFPS::default(),
    ));

}

fn camera_zoom(
    mut query_camera: Query<&mut OrthographicProjection, With<Camera>>,
    mut scroll_evr: EventReader<MouseWheel>,
) {
    let mut projection = query_camera.single_mut();

    for ev in scroll_evr.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                if ev.y > 0.0 {
                    // zoom in
                    projection.scale /= 1.25;
                } else {
                    // zoom out
                    projection.scale *= 1.25;
                }
                ev.y;
            }
            MouseScrollUnit::Pixel => {
                panic!("MouseScrollUnit::Pixel not supported");
            }
        }
    }
}

fn load_background(
    // load the background image
    asset_server: Res<AssetServer>, 
    // commands in order to spawn entities
    mut commands: Commands,
    // get the main window for positioning
    window_q: Query<&Window, With<PrimaryWindow>>
) {
    // get the main window; unwrap is safe here because the window always exists
    let window = window_q.get_single().unwrap();
    /*
    // this was a try to make an infinite background, but it didn't work
    let sampler_desc = ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        ..default()
    };

    let settings = move |s: &mut ImageLoaderSettings| {
        s.sampler = ImageSampler::Descriptor(sampler_desc.clone());
    };

    // Later, inside the SpriteBundle:
    texture: asset_server.load_with_settings("background.png", settings),
    */
    // spawn the Background
    commands.spawn(
        (
            SpriteBundle {
            texture: asset_server.load("background.png"),
            transform: Transform {
                // set Z to -1 to ensure the background is displayed behind everything else
                translation: Vec3::new(0., 0., -1.0),
                scale: Vec3::new(window.width()/1024., window.height()/1024., 1.0),
                ..default()
            },
            ..default()
            },
            Name::new("Background"),
            Background {}
        )
    );
}
