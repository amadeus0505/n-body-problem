
// this for disabling the console window on windows
#![windows_subsystem = "windows"]
use bevy::{app::AppExit, prelude::*, window::WindowMode};

// used for the FPS counter
use iyes_perf_ui::prelude::*;

// import Planet logic
mod planets;
use planets::PlanetPlugin;

mod world;
use world::{WorldPlugin, GameState};

mod ui;
use ui::UiPlugin;

/* 
TODO: 
    * Hintergrund Shader
    * Shader für den Pfad der Planeten
    * Speichern von aktuellen Planetenpositionen -geschwindigkeiten und -beschleunigungen (für speichern und laden von "Spielständen")
    * genaue positionierung von planeten ermöglichen
    * eventuell "sonnensystem editor": 
        # automatische plazierung von planeten
        # berechnung der geschwindigkeiten und beschleunigungen nach eingabe der 
                                                                                * masse
                                                                                * größe
                                                                                * dichte
                                                                                * entfernung
                                                                                * flugrichtung(links,rechts)
    * Eigenes bild für Sonne
*/


fn main() {
    // create the default plugin with window settings
    let default_plugin = DefaultPlugins.set(
        WindowPlugin {
            primary_window: Some(Window {
                mode: WindowMode::BorderlessFullscreen,
                ..default()
            }),
            ..default()
        }
    );
    // create app
    App::new()
        // load default plugin from above
        .add_plugins(default_plugin)
        // load world inspector plugin (for setting specific world values [in an egui window])
        // .add_plugins(WorldInspectorPlugin::new())
        // load planet logic
        .add_plugins(PlanetPlugin)
        // load world plugin (for setting up world [camera, background, etc.])
        .add_plugins(WorldPlugin)
        // load User Interface
        .add_plugins(UiPlugin)
        // add system for gizmo config (line width, etc.)
        // load update systems where order is not important
        .add_systems(Update, (exit_listener,))

        // set the initial state to Running
        .insert_state(GameState::Paused)

        // load Plugins for FPS monitoring
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(PerfUiPlugin)



        // finally, run the app
        .run();
}



fn exit_listener(
    key_q: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
){
    if key_q.just_pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
    if key_q.just_pressed(KeyCode::Space) {
        match state.get() {
            GameState::Running => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Running),
        }
    }
}





