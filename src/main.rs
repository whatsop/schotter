use nannou::prelude::*;
use nannou_egui::{self, egui, Egui};

const ROWS: u32 = 22;
const COLS: u32 = 12;
const SIZE: u32 = 30;
const MARGIN: u32 = 30;
const WIDTH: u32 = COLS * SIZE + 2 * MARGIN;
const HEIGHT: u32 = ROWS * SIZE + 2 * MARGIN;
const LINE_WIDTH: f32 = 0.08;

fn main() {
    nannou::app(model)
        .update(update)
        .loop_mode(LoopMode::refresh_sync())
        .run()
}

struct Model {
    motion: f32,
    disp_adj: f32,
    rot_adj: f32,
    gravel: Vec<Stone>,
    main_window: WindowId,
    ui: Egui,
}

fn model(app: &App) -> Model {
    let window = app
        .new_window()
        .title(app.exe_name().unwrap())
        .size(WIDTH, HEIGHT)
        .view(view)
        .key_pressed(key_pressed)
        .build()
        .unwrap();
    let ui_window = app
        .new_window()
        .title(app.exe_name().unwrap() + " controls")
        .size(280, 130)
        .view(ui_view)
        .raw_event(raw_ui_event)
        .key_pressed(key_pressed)
        .build()
        .unwrap();
    let ui_window_ref = app.window(ui_window).unwrap();
    let ui = Egui::from_window(&ui_window_ref);

    let motion = 0.0;
    let disp_adj = 1.0;
    let rot_adj = 1.0;

    let mut gravel = Vec::new();
    for y in 0..ROWS {
        for x in 0..COLS {
            let color = Rgb::default();
            let mut stone = Stone::new(x as f32, y as f32, color);

            // initialize a random color for each stone that will be used later
            let temp_color = Rgb::new(
                random_range(0.25, 1.0),
                random_range(0.25, 1.0),
                random_range(0.25, 1.0),
            );
            stone.temp_color = temp_color;

            gravel.push(stone);
        }
    }

    Model {
        motion,
        disp_adj,
        rot_adj,
        gravel,
        main_window: window,
        ui,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    update_ui(app, model);

    // update stones
    for stone in &mut model.gravel {
        let factor = stone.y / ROWS as f32;

        if stone.cycles == 0 {
            if random_f32() > model.motion {
                stone.x_velocity = 0.0;
                stone.y_velocity = 0.0;
                stone.rotation_velocity = 0.0;
                stone.cycles = random_range(50, 100);
            } else {
                let new_cycles = random_range(50, 100);

                let disp_factor = factor * model.disp_adj;
                let rot_factor = factor * model.rot_adj;
                let new_x = random_range(-0.5, 0.5) * disp_factor;
                let new_y = random_range(-0.5, 0.5) * disp_factor;
                let new_rot = random_range(-PI / 4.0, PI / 4.0) * rot_factor;
                stone.x_velocity = (new_x - stone.x_offset) / new_cycles as f32;
                stone.y_velocity = (new_y - stone.y_offset) / new_cycles as f32;
                stone.rotation_velocity = (new_rot - stone.rotation) / new_cycles as f32;

                stone.cycles = new_cycles;
            }
        } else {
            stone.x_offset += stone.x_velocity;
            stone.y_offset += stone.y_velocity;
            stone.rotation += stone.rotation_velocity;

            stone.cycles -= 1;
        }

        // set color only if stone is moving
        if abs(stone.x_offset) + abs(stone.y_offset) + abs(stone.rotation) > 0.000001 {
            stone.color = Rgb::new(
                stone.temp_color.red * factor,
                stone.temp_color.green * factor,
                stone.temp_color.blue * factor,
            );
        } else {
            stone.color = Rgb::new(0.0, 0.0, 0.0);
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    // create second coordinate system for drawing
    // (0,0) is in the middle of the first left top corner cell
    let gdraw = draw
        .scale(SIZE as f32)
        .scale_y(-1.0)
        .x_y(COLS as f32 / -2.0 + 0.5, ROWS as f32 / -2.0 + 0.5);

    // drawing cells
    for stone in &model.gravel {
        let cdraw = gdraw.x_y(stone.x, stone.y);
        cdraw
            .rect()
            .color(SNOW)
            .stroke(stone.color)
            .stroke_weight(LINE_WIDTH)
            .w_h(1.0, 1.0)
            .x_y(stone.x_offset, stone.y_offset)
            .rotate(stone.rotation);
    }

    gdraw.background().color(BLACK);
    gdraw.to_frame(app, &frame).unwrap();
}

struct Stone {
    x: f32,
    y: f32,
    x_offset: f32,
    y_offset: f32,
    rotation: f32,
    color: Rgb,
    temp_color: Rgb,
    x_velocity: f32,
    y_velocity: f32,
    rotation_velocity: f32,
    cycles: u32,
}

impl Stone {
    fn new(x: f32, y: f32, color: Rgb) -> Self {
        let x_offset = 0.0;
        let y_offset = 0.0;
        let rotation = 0.0;
        let temp_color = Rgb::default();
        let x_velocity = 0.0;
        let y_velocity = 0.0;
        let rotation_velocity = 0.0;
        let cycles = 0;
        Self {
            x,
            y,
            x_offset,
            y_offset,
            rotation,
            color,
            temp_color,
            x_velocity,
            y_velocity,
            rotation_velocity,
            cycles,
        }
    }
}

fn key_pressed(app: &App, model: &mut Model, key: Key) {
    match key {
        // save as a png image
        Key::S => {
            if let Some(window) = app.window(model.main_window) {
                window.capture_frame(app.exe_name().unwrap() + ".png");
            }
        }

        _other_key => {}
    }
}
fn ui_view(_app: &App, model: &Model, frame: Frame) {
    model.ui.draw_to_frame(&frame).unwrap();
}

fn raw_ui_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.ui.handle_raw_event(event);
}

fn update_ui(app: &App, model: &mut Model) {
    let ctx = model.ui.begin_frame();

    // create new UI
    egui::Window::new("Schotter Control Panel")
        .collapsible(false)
        .show(&ctx, |ui| {
            // displacement widget
            let disp_widget =
                egui::Slider::new(&mut model.disp_adj, 0.0..=5.0).text("Displacement");

            // rotation widget
            let rot_widget = egui::Slider::new(&mut model.rot_adj, 0.0..=5.0).text("Rotation");

            // motion widget
            let motion_widget = egui::Slider::new(&mut model.motion, 0.0..=1.0).text("Motion");

            // screenshot widget
            let screenshot_button = egui::Button::new("Screenshot");

            // add widgets to UI
            ui.add(disp_widget);
            ui.add(rot_widget);
            ui.add(motion_widget);

            ui.horizontal(|ui| {
                if ui.add(screenshot_button).clicked() {
                    if let Some(window) = app.window(model.main_window) {
                        window.capture_frame(app.exe_name().unwrap() + ".png");
                    }
                };
                ui.label(egui::RichText::new(format!("fps: {}", app.fps())));
            })
        });
}
