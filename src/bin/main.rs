use std::time::{Duration, Instant};
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use winit_input_helper::WinitInputHelper;
use life::{BaseLifeBoard, Cell, ParallelLifeBoard, LifeBoard};

const SCALE: u32 = 4;  // How many logical pixels correspond to one `LifeCell`
const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const N_THREADS: u8 = 5;
const MS_TIME_STEP: u64 = 250;

fn main() {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let mut auto_step: bool = false;
    let window = initialize_window(&event_loop);
    let mut pixels = initialize_pixel_buffer(&window);
    let mut game = initialize_life_board();

    let mut last_frame_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            redraw_world(&mut pixels, &mut game);
        } else if let Event::MainEventsCleared = event {
            let now = Instant::now();
            let elapsed = now - last_frame_time;
            if elapsed >= Duration::from_millis(MS_TIME_STEP) && auto_step {
                last_frame_time = now;
                game.simulate();
                window.request_redraw();
            }
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::ExitWithCode(0);
                return;
            } else if input.key_pressed(VirtualKeyCode::Space) {
                game.simulate();
                window.request_redraw();
                return;
            } else if input.key_pressed(VirtualKeyCode::P) {
                auto_step = if auto_step { false } else { true };
                return;
            }
        }
    });
}

fn redraw_world(pixels: &mut Pixels, game: &mut ParallelLifeBoard<Cell>) {
    let frame = pixels.frame_mut();
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let x = i % (WIDTH / SCALE) as usize;
        let y = i / (WIDTH / SCALE) as usize;
        if let Some(bool) = game.is_cell_alive(x, y) {
            if bool {
                pixel.copy_from_slice(&[0xff, 0, 0, 0xff]);
            } else {
                pixel.copy_from_slice(&[0xff, 0xff, 0xff, 0xff]);
            }
        }
    }
    pixels.render().expect("Unable to render pixel buffer.");
}

fn initialize_window(event_loop: &EventLoop<()>) -> Window {
    let size = LogicalSize::new(WIDTH, HEIGHT);
    WindowBuilder::new()
        .with_title("Conway's Game of Life")
        .with_min_inner_size(size)
        .with_inner_size(size)
        .build(&event_loop)
        .unwrap()
}

fn initialize_life_board() -> ParallelLifeBoard<Cell> {
    ParallelLifeBoard::<Cell>::from_board(
        BaseLifeBoard::gen(
            (WIDTH / SCALE) as usize,
            (HEIGHT / SCALE) as usize,
            Cell::gen
        ),
        N_THREADS
    )
}

fn initialize_pixel_buffer(window: &Window) -> Pixels {
    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    Pixels::new(WIDTH / SCALE, HEIGHT / SCALE, surface_texture).expect("Unable to create pixel buffer")
}