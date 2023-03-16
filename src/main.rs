use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
struct SimulationState {
    entities: Vec<Entity>,
    time: f64,
}

#[derive(Clone, Serialize, Deserialize)]
struct Entity {
    x: f32,
    y: f32,
    // Other properties specific to your artificial life simulation
}

fn update_simulation(state: &mut SimulationState, dt: f32) {
    for entity in &mut state.entities {
        entity.x += dt * 10.0; // Move entities horizontally at a fixed rate
        entity.y += dt * 10.0; // Move entities vertically at a fixed rate
    }
}

fn save_simulation_state(state: &SimulationState, file_path: &str) {
    let json = serde_json::to_string_pretty(state).unwrap();
    std::fs::write(file_path, json).expect("Failed to save the simulation state.");
}

fn load_simulation_state(file_path: &str) -> SimulationState {
    let json = std::fs::read_to_string(file_path).expect("Failed to read the simulation state.");
    serde_json::from_str(&json).expect("Failed to parse the simulation state.")
}

use egui_macroquad::*;
use macroquad::prelude::*;

#[macroquad::main("Artificial Life Simulation")]
async fn main() {
    let mut state = SimulationState {
        entities: Vec::new(),
        time: 0.0,
    };

    // Initialize entities
    state.entities = vec![
        Entity::new(50.0, 50.0),
        Entity::new(100.0, 100.0),
        Entity::new(200.0, 200.0),
    ];

    let mut paused = false;
    let mut speed = 1.0;
    let rewind_interval = 0.5; // Store the state every 0.5 seconds
    let mut elapsed_time = 0.0;
    let mut rewind_buffer = Vec::<SimulationState>::new();

    let mut slider_value = 0;
    let mut max_buffer_index = 0;

    // push initial state
    rewind_buffer.push(state.clone());

    loop {
        // Process user input and update the simulation
        let dt = get_frame_time() * speed;

        if !paused {
            elapsed_time += dt;

            // Store the state in fixed intervals
            if elapsed_time >= rewind_interval {
                rewind_buffer.push(state.clone());
                elapsed_time = 0.0;
            }
            max_buffer_index = rewind_buffer.len() - 1;
            // Add a slider to control the simulation time
            slider_value = max_buffer_index;
            update_simulation(&mut state, dt);
        } else {
            // Update the simulation state based on the slider value
            if slider_value < rewind_buffer.len() {
                state = rewind_buffer[slider_value].clone();
            }
        }

        clear_background(WHITE);
        for entity in &state.entities {
            draw_circle(entity.x, entity.y, 10.0, BLUE); // Draw each entity as a blue circle
        }

        // EGUI interface
        egui_macroquad::ui(|egui_ctx| {
            egui::panel::TopBottomPanel::top("menu").show(egui_ctx, |ui| {
                if ui.button("Pause/Resume").clicked() {
                    paused = !paused;
                }

                // Add a speed slider
                ui.add(egui::Slider::new(&mut speed, 0.01..=10.0).text("Speed").step_by(rewind_interval as f64));

                ui.add(egui::Slider::new(&mut slider_value, 0..=max_buffer_index).text("Time"));

                if ui.button("Save").clicked() {
                    save_simulation_state(&state, "simulation_state.json");
                }
                if ui.button("Load").clicked() {
                    state = load_simulation_state("simulation_state.json");
                }
            });
        });

        egui_macroquad::draw();

        // End the frame and update the window
        next_frame().await;
    }
}

impl Entity {
    fn new(x: f32, y: f32) -> Self {
        Entity { x, y }
    }
}
