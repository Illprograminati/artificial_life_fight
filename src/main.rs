use serde::{Deserialize, Serialize};

const HEIGHT: i32 = 1080;
const WIDTH: i32 = 1920;

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
        match rand::gen_range(1, 5) {
            1 => entity.x -= 1.0,
            2 => entity.x += 1.0,
            3 => entity.y -= 1.0,
            _ => entity.y += 1.0,
        }
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
use macroquad::{prelude::*, rand};

pub fn draw_grid_2d(slices: u32, spacing: f32, axes_color: Color, other_color: Color) {
    let half_slices = (slices as i32) / 2;
    for i in -half_slices..half_slices + 1 {
        let color = if i == 0 { axes_color } else { other_color };

        draw_line(
            i as f32 * spacing,
            -half_slices as f32 * spacing,
            i as f32 * spacing,
            half_slices as f32 * spacing,
            0.1,
            color,
        );
        draw_line(
            -half_slices as f32 * spacing,
            i as f32 * spacing,
            half_slices as f32 * spacing,
            i as f32 * spacing,
            0.1,
            color,
        );
    }
}

#[macroquad::main("Artificial Life Simulation")]
async fn main() {
    let mut state = SimulationState {
        entities: Vec::new(),
        time: 0.0,
    };

    // Initialize entities
    state.entities = vec![Entity::new(50.0, 50.0)];

    let mut paused = false;
    let mut speed = 1.0;
    let rewind_interval = 0.5; // Store the state every 0.5 seconds
    let mut elapsed_time = 0.0;
    let mut rewind_buffer = Vec::<SimulationState>::new();

    let mut slider_value = 0;
    let mut max_buffer_index = 0;
    let mut current_index = 0;

    let min_zoom = vec2(0.008, 0.008);
    let max_zoom = vec2(0.2, 0.2);

    let mut camera = Camera2D {
        zoom: vec2(0.01, 0.01),
        target: vec2(50.0, 50.0),
        ..Default::default()
    };

    let move_speed = 0.001;
    let zoom_speed = 0.001;

    let circle_size = 1.0;
    let half_circle_size = circle_size / 2.0;

    // push initial state
    rewind_buffer.push(state.clone());

    let mut old_mouse_position = mouse_position();

    loop {
        // Process user input and update the simulation
        let dt = get_frame_time() * speed;

        let current_mouse_position = mouse_position();
        if is_mouse_button_down(MouseButton::Left) {
            let mouse_delta = vec2(
                current_mouse_position.0 - old_mouse_position.0,
                old_mouse_position.1 - current_mouse_position.1,
            );
            camera.target += mouse_delta * (-1.0 / camera.zoom) * move_speed;
        }
        old_mouse_position = current_mouse_position;

        // move the camera with arrow keys
        if is_key_down(KeyCode::Right) {
            camera.target.x += move_speed / camera.zoom.x;
        }
        if is_key_down(KeyCode::Left) {
            camera.target.x -= move_speed / camera.zoom.x;
        }
        if is_key_down(KeyCode::Up) {
            camera.target.y -= move_speed / camera.zoom.y;
        }
        if is_key_down(KeyCode::Down) {
            camera.target.y += move_speed / camera.zoom.y;
        }

        // zoom in and out with mouse wheel
        let scroll = mouse_wheel().1;
        if scroll != 0.0 {
            let mouse_world_pos = camera.screen_to_world(mouse_position().into());
            // camera.offset = mouse_position().into();
            camera.target = mouse_world_pos;
            camera.zoom += zoom_speed * scroll.signum();
            camera.zoom = camera.zoom.clamp(min_zoom, max_zoom);
        }

        if !paused {
            elapsed_time += dt;

            // Store the state in fixed intervals
            if elapsed_time >= rewind_interval {
                if current_index != max_buffer_index {
                    current_index = max_buffer_index;
                    state = rewind_buffer[slider_value].clone();
                }

                update_simulation(&mut state, dt);
                rewind_buffer.push(state.clone());

                max_buffer_index = rewind_buffer.len() - 1;

                // Add a slider to control the simulation time
                slider_value = max_buffer_index;

                current_index += 1;
                elapsed_time = 0.0;
            }
        } else {
            // Update the simulation state based on the slider value
            if slider_value < rewind_buffer.len() {
                state = rewind_buffer[slider_value].clone();
                current_index = slider_value;
            }
        }

        clear_background(WHITE);

        push_camera_state();
        set_camera(&camera);

        draw_grid_2d(20000, 1., BLACK, GRAY);

        for entity in &state.entities {
            draw_circle(
                entity.x + half_circle_size,
                entity.y + half_circle_size,
                half_circle_size,
                BLUE,
            ); // Draw each entity as a blue circle
        }

        pop_camera_state();

        // EGUI interface
        egui_macroquad::ui(|egui_ctx| {
            egui::panel::TopBottomPanel::top("menu").show(egui_ctx, |ui| {
                if ui.button("Pause/Resume").clicked() {
                    paused = !paused;
                }

                // Add a speed slider
                ui.add(
                    egui::Slider::new(&mut speed, 0.01..=10.0)
                        .text("Speed")
                        .step_by(rewind_interval as f64),
                );

                ui.add(egui::Slider::new(&mut slider_value, 0..=max_buffer_index).text("Time"));

                if cfg!(not(wasm)) {
                    if ui.button("Save").clicked() {
                        save_simulation_state(&state, "simulation_state.json");
                    }
                    if ui.button("Load").clicked() {
                        state = load_simulation_state("simulation_state.json");
                    }
                }
                if ui.button("focus").clicked() {
                    camera.target = vec2(50.0, 50.0);
                }
            });
            egui::panel::TopBottomPanel::bottom("debug").show(egui_ctx, |ui| {
                ui.add(egui::Label::new(format!("Zoom  : {}", camera.zoom.x)));
                ui.add(egui::Label::new(format!("Target: {}", camera.target)));
                ui.add(egui::Label::new(format!("Offset: {}", camera.offset)));
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
