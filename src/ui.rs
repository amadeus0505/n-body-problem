use bevy::{prelude::*, window::PrimaryWindow};

use bevy_egui::egui::{DragValue, RichText};
use bevy_inspector_egui::{bevy_egui::{egui, EguiContext, EguiPlugin}, bevy_inspector::{self, hierarchy::SelectedEntities}, DefaultInspectorConfigPlugin};

use crate::{planets::Planet, world::{Game, GameState}};

pub struct UiPlugin;

const ACCENT_COLOR: egui::Color32 = egui::Color32::from_rgb(0, 255, 0);

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(EguiPlugin)
            .add_plugins(DefaultInspectorConfigPlugin)
            .add_systems(Update, ui_first_window_system)
            ;
    }
}


fn _inspector_ui(world: &mut World, mut selected_entities: Local<SelectedEntities>) {

    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();
    egui::SidePanel::left("hierarchy")
        .default_width(200.0)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.heading("Hierarchy");

                
                bevy_inspector::hierarchy::hierarchy_ui(
                    world,
                    ui,
                    &mut selected_entities,
                );

                ui.label("Press escape to toggle UI");
                ui.allocate_space(ui.available_size());
            });
        });

    egui::SidePanel::right("inspector")
        .default_width(250.0)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.heading("Inspector");

                match selected_entities.as_slice() {
                    &[entity] => {
                        bevy_inspector_egui::bevy_inspector::ui_for_entity(world, entity, ui);
                    }
                    entities => {
                        bevy_inspector_egui::bevy_inspector::ui_for_entities_shared_components(
                            world, entities, ui,
                        );
                    }
                }
                ui.allocate_space(ui.available_size());
            });
        });
}

fn ui_first_window_system(
    mut planet_q: Query<(&mut Planet, &Name)>,
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryWindow>>,
    mut game: ResMut<Game>,
    game_state: Res<State<GameState>>,
) {
    egui::SidePanel::left("Settings")
        .default_width(250.0)
        .show(egui_ctx.single_mut().get_mut(), |ui| {
            ui.vertical(|ui| {
                ui.separator();
                for (mut planet, name) in planet_q.iter_mut() {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(name.to_string()).color(ACCENT_COLOR)
                        );
                        ui.horizontal(|ui| {
                            ui.label("Add Velocity: ");
                            ui.add_enabled(
                                if game_state.get() == &GameState::Running {false} else {true},
                                DragValue::new(&mut planet.initial_velocity.x)
                                .speed(0.05)
                                .prefix("x:")
                            );
                            ui.add_enabled(
                                if game_state.get() == &GameState::Running {false} else {true},
                                DragValue::new(&mut planet.initial_velocity.y)
                                .speed(0.05)
                                .prefix("y:")
                            );
                        });
                        ui.horizontal( |ui| {
                            ui.label("Gravity: ");
                            ui.add(
                                DragValue::new(&mut planet.gravity)
                                .clamp_range(0.0..=100.0)
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Radius: ");
                            ui.add(
                                DragValue::new(&mut planet.radius)
                                .clamp_range(0.0..=1000.0)
                            );
                        });
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.label("Current Velocity:");
                            ui.add_enabled(false,
                                DragValue::new(&mut planet.velocity.x)
                                .prefix("x:")
                            );
                            ui.add_enabled(false,
                                DragValue::new(&mut planet.velocity.y)
                                .prefix("y:")
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Current Acceleration: ");
                            ui.add_enabled(false, 
                                DragValue::new(&mut planet.acceleration.x)
                                .suffix("x:")
                            );
                            ui.add_enabled(false, 
                                DragValue::new(&mut planet.acceleration.y)
                                .suffix("y:")
                            );
                        });
                        ui.separator();
                    });
                }    
            });
                
            ui.horizontal(|ui| {
                ui.label("Game Speed: ");
                ui.add_enabled(
                    if game_state.get() == &GameState::Running {true} else {true},
                    DragValue::new(&mut game.speed)
                    .clamp_range(30.0..=1000.0)
                );
            });
        });
}