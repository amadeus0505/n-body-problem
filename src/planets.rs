use bevy::{input::common_conditions::input_toggle_active, prelude::*, window::PrimaryWindow};

use crate::{world::{Game, MyCamera}, GameState};

use std::time::{Duration, Instant};

use std::collections::HashMap;

pub struct PlanetPlugin;

const G: f32 = 6.67430e-11;

impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, (setup, setup_gizmo_config))
        // load update systems [Note: these are only running if GameState is set to Running]
        // .add_systems(Update, planet_movement.run_if(in_state(GameState::Running)))
        // add systems that get called every frame [Note: these are always running]
        .add_systems(Update, (update_planet_size, spawn_new_planets, clicked_on_planet))
        // load future planet movement drawing system, but only if GameState is set to Paused
        .add_systems(Update, draw_future_planet_movement.run_if(input_toggle_active(true, KeyCode::Tab)))
        // add physics system to FixedUpdate in order to get a more stable simulation
        .add_systems(FixedUpdate, (update_velocity, planet_movement).chain().run_if(in_state(GameState::Running)))
        // transfer initial velocity to current velocity, but only if GameState is set from anything to Running
        .add_systems(OnEnter(GameState::Running), set_initial_velocity)

        // register type Planet to display and edit it in the WorldInspector
        .register_type::<Planet>()
        // init future planet movement gizmos for drawing lines
        .init_gizmo_group::<FutuerPlanetMovementGizmos>()
        ;
    }
}

// ############# Planet Label Component #############
#[derive(Component)]
struct PlanetLabel;

// ############# Planet Component #############
#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Planet {
    pub gravity: f32,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub radius: f32,
    pub initial_velocity: Vec3,
}

impl Planet {
    pub fn mass(&self) -> f32 {
        self.gravity * self.radius.powi(2) / G
    }
}

// Used for drawing lines to display future planet movement
#[derive(Default, Reflect, GizmoConfigGroup)]
struct FutuerPlanetMovementGizmos {}

// ############# Helper Functions #############
fn spawn_planet(mut commands: Commands, asset_server: &AssetServer, game: &mut Game, position: Vec3) {
    // we added 1 planet, so we add 1 to the global planet count
    game.planet_count += 1;

    // define scale for the planet
    // default planet size is 1024x1024, so with 0.05 scale it is 51.2x51.2
    let scale = 0.05;

    // spawn Planet
    commands.spawn(
        (
            // with a Sprite bundle (for all the visual stuff)
            SpriteBundle {
                // transform describes the position, scale, and rotation of the sprite
                transform: Transform {
                    // set the position to the center of the window
                    translation: position,
                    // set the image to a prescaled size
                    scale: Vec3::new(scale, scale, 1.0),
                    // loads all the default values (rotation set to 0)
                    ..default()
                },
                // load texture with asset server (automatically looks in the assets folder for the file name provided)
                texture: asset_server.load("planet00.png"),
                // again, load all the default values [Note: hover over SpriteBundle to see all the values you can set here]
                ..default()
            },
            // and a Planet, so it is marked as a planet 
            // (also holds some data about the planet [Q: is this the best way to do this? Or should gravity, velocity, etc. all be separate Components?])
            Planet {
                gravity: 10.,
                velocity: Vec3::new(0., 0., 0.),
                radius: 100.0,
                ..default()
            },
            // Give it a name so it is easily identifiable in the WorldInspector
            Name::new(format!("Planet {}", game.planet_count))
        )
    // spawn one children, a Text2dBundle, to display the number of the planet
    ).with_children(|builder|{
        builder.spawn(
            (
                Text2dBundle {
                    text: Text::from_section(
                        game.planet_count.to_string(), 
                        TextStyle {
                            font_size: 600.0,
                            color: Color::GREEN,
                            ..default()
                        }
                    ),
                    transform: Transform {
                        translation: Vec3::new(0., 0., 1.0),
                        ..default()
                    },
                    ..default()
                },
                PlanetLabel {}
            )
        );
    });
}

// ############# Systems #############
// ############# Startup Systems #############
fn setup (
    // get commands to spawn entities
    commands: Commands,
    // getting the asset server to loading textures
    asset_server: Res<AssetServer>,
    mut game: ResMut<Game>
) {
    // spawn the first planet at the center of the window
    spawn_planet(commands, asset_server.as_ref(), game.as_mut(), Vec3::new(0., 0., 0.));

}

fn setup_gizmo_config (
    mut config_store: ResMut<GizmoConfigStore>
) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.line_width = 2.;

}

// ############# Update Systems: GameState::Running #############
// calculate the velocity of the planets based on their current velocity, mass and distance
fn update_velocity(
    mut planet_q: Query<(&mut Transform, &mut Planet)>,
    game: Res<Game>
) {

    let mut bodies: Vec<(Mut<Transform>, Mut<Planet>)> = Vec::new();

    for (transform, mut planet) in planet_q.iter_mut() {
        for (other_transform, other_planet) in bodies.iter_mut() {
            let sqr_dst = (other_transform.translation - transform.translation).length_squared();
            let force_dir = (other_transform.translation - transform.translation).normalize();
            // force is the same for both planets
            let force = force_dir * G * (planet.mass() * other_planet.mass()) / sqr_dst;
            // acceleration differs based on mass
            planet.acceleration = force / planet.mass();
            let planet_acceleration = force / planet.mass();
            // movement is only dependent on stepsize: this is fixed, because the whole system gets updated in FixedUpdate, which frequency gets
            // set by the Game.speed value [in world.rs:55]
            planet.velocity += planet_acceleration * game.stepsize;

            // same force is applied, just in the opposite direction
            other_planet.acceleration = -force / other_planet.mass();
            let other_planet_acceleration = -force / other_planet.mass();
            other_planet.velocity += other_planet_acceleration * game.stepsize;
        }
        bodies.push((transform, planet))
    }
}

// updates the position of the planets based on their velocity
fn planet_movement (
    mut query: Query<(&mut Transform, &Planet)>,
    game: Res<Game>
) {

    for (mut transform, planet) in query.iter_mut() {
        transform.translation += planet.velocity * game.stepsize;
    }

}

// sets planet and label size based on radius
fn update_planet_size(
    mut query: Query<(&mut Transform, &Planet, &Children)>,
    mut child_q: Query<&mut Text, With<PlanetLabel>>,
    query_camera: Query<&mut OrthographicProjection, With<Camera>>
) {
    for (mut transform, planet, children) in query.iter_mut() {
        // go over all children (currently, there should only be one child)
        children.iter().for_each(|child| {
            // here it gets checked if the child is a Text (child_q only contains Texts)
            if let Ok(mut text) = child_q.get_mut(*child){
                // if so, set the text size to the radius of the planet
                // let font_size = text.sections[0].style.font_size;
                let size = 600. * query_camera.single().scale;
                if size < 1500. {
                    text.sections[0].style.font_size = size;
                }
            }
        });
        // set the scale of the planet sprite to the radius of the planet
        transform.scale = Vec3::splat(planet.radius/1024.0);
    }
}
// ############# Update Systems: GameState::Paused #############
// This can get a lot more performant
fn draw_future_planet_movement (
    mut gizmos: Gizmos,
    planet_q: Query<(&Transform, Ref<Planet>, &Name)>,
    game: Res<Game>
) {
    // TODO: Currently all the calculations are done every frame, this is not necessary when the velocity of the planets dont change

    // ############# Time Measure #############
    let mut _now = Instant::now();
    let _elapsed1: Duration;
    let mut _elapsed2: Duration = Duration::new(0, 0);
    let mut _elapsed3: Duration = Duration::new(0, 0);
    let mut _elapsed4: Duration = Duration::new(0, 0);
    // ########################################

    // transform all planets to virtual planets
    let mut virtual_planets = planet_q.iter().map(|(transform, planet, name)| {
        let virtual_transform = transform.clone();
        let virtual_planet_velocity = planet.velocity + planet.initial_velocity;
        let virtual_planet_mass = planet.mass();
        let virtual_planet_name = name.to_string();
        // virtual_planet_velocity = planet.velocity;
        (virtual_transform, virtual_planet_velocity, virtual_planet_mass, virtual_planet_name)
    }).collect::<Vec<_>>();

    // ############# Time Measure #############
    _elapsed1 = _now.elapsed();
    // ########################################

    // points to draw [TODO: make this dynamic, so more than the path of two planets can be drawn]
    let mut points = HashMap::<String, Vec<Vec2>>::new();

    // defines how long in the future we can see
    // [TODO: Maybe make a field in the WorldInspector to change this value]
    for _ in 0..5000 {

        // ############# Time Measure #############
        _now = Instant::now();
        // ########################################

        let mut bodies: Vec<(Transform, (Vec3, f32), String)> = Vec::new();
        // use clone here to satisfy borrow checker; altough virtual_planets gets overwritten later
        // it is crucial to consume the values here, otherwise we would not be able to set the velocity of the "other_planets" 
        for (virtual_transform, mut virtual_planet_velocity, virtual_planet_mass, virtual_planet_name) in virtual_planets.clone().into_iter() {
            // we dont need to consume bodies here, a mutable reference will do just fine
            for (other_virtual_transform, (other_virtual_planet_velocity, other_virtual_planet_mass ), _other_name) in bodies.iter_mut() {
                let sqr_dst = (other_virtual_transform.translation - virtual_transform.translation).length_squared();
                let force_dir = (other_virtual_transform.translation - virtual_transform.translation).normalize();
                // calculate the force between the two planets [same for both planets, but with opposite direction]
                let force = force_dir * G * (virtual_planet_mass * *other_virtual_planet_mass) / sqr_dst;
                // calculate the acceleration for one planet and apply it
                let planet_acceleration = force / virtual_planet_mass;
                virtual_planet_velocity += planet_acceleration * game.stepsize;

                // calculate the acceleration for the other planet and apply it
                let other_planet_acceleration = -force / *other_virtual_planet_mass;
                *other_virtual_planet_velocity += other_planet_acceleration * game.stepsize;
            }
            // we then move the values to the bodies vector, thats why we needed to clone the virtual_planets
            bodies.push((virtual_transform, (virtual_planet_velocity, virtual_planet_mass), virtual_planet_name))
        }

        _elapsed2 = _now.elapsed();
        _now = Instant::now();

        // move all calculated bodies back to virtual planets so they can get used in the next iteration
        virtual_planets = bodies.into_iter().map(|(transform, (planet_velocity, planet_mass), planet_name)| {
            (transform, planet_velocity, planet_mass, planet_name)
        }).collect::<Vec<_>>();
        // ############# Time Measure #############
        _elapsed3 = _now.elapsed();
        _now = Instant::now();

        // go over all virtual planets and move them, then push the position to the points vector
        for (transform, planet_velocity, _planet_mass, virtual_planet_name) in virtual_planets.iter_mut() {
            transform.translation += *planet_velocity * game.stepsize;
            // TODO: make this dynamic, so more than the path of two planets can be drawn
            
            if !points.contains_key(virtual_planet_name) {
                points.insert(virtual_planet_name.clone(), vec![]);
            }
            for (name, calculated_points) in points.iter_mut() {
                if name == virtual_planet_name {
                    calculated_points.push(Vec2::new(transform.translation.x, transform.translation.y));
                }
            }
        }

        _elapsed4 = _now.elapsed();
    }
    _now = Instant::now();
    // finally, draw the calculated points as lines
    for (_name, calculated_points) in points.iter() {
        gizmos.linestrip_2d(calculated_points.clone(), Color::WHITE);
    }
    let _elapsed5 = _now.elapsed();

    // this is just for debugging purposes [performance measurement]
    // println!("Transform Planets to Virtual Planets: {:?}, Calculating Physics: {:?}, Moving from bodies to Planets: {:?}, Update Position and add points: {:?}, Draw Lines: {:?}", _elapsed1, _elapsed2, _elapsed3, _elapsed4, _elapsed5);
}

// ############# OnEnter Systems: GameState::Running #############
// transfers the initial velocity to the current velocity
fn set_initial_velocity(
    mut query: Query<&mut Planet>
) {
    for mut planet in query.iter_mut() {
        // add initial velocity to current velocity
        let velocity = planet.initial_velocity;
        planet.velocity += velocity;
        // set initial velocity to zero, so it doesnt get added the next time we pause and play the simulation
        planet.initial_velocity = Vec3::ZERO;
    }
}

fn clicked_on_planet(
    interaction: Query<(&Interaction, ), (With<Planet>, Changed<Interaction>)>
) {
    for (interaction, ) in interaction.iter() {
        match interaction {
            Interaction::Pressed => {
                println!("Clicked on Planet");
            }
            _ => {}
        }
    }
}

fn spawn_new_planets (
    buttons: Res<ButtonInput<MouseButton>>,
    commands: Commands,
    mut game: ResMut<Game>,
    asset_server: Res<AssetServer>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MyCamera>>
) {
    if buttons.just_pressed(MouseButton::Right) {
        let window = window_q.get_single().unwrap();
        // fetch window to get mouse position
        if let Some(window_position) = window.cursor_position() {
            // fetch camera and its global transform
            let (camera, global_transform) = camera_q.single();
            // convert window position to world position
            if let Some(cursor_position) = camera.viewport_to_world_2d(global_transform, window_position) {
                spawn_planet(commands, asset_server.as_ref(), game.as_mut(), Vec3::new(cursor_position.x, cursor_position.y, 0.))
            }
        }
    }
}
