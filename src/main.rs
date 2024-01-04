mod life;

use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use crate::life::LifeBoard;

const SCALE: u32 = 16;
const WIDTH: u32 = 320;
const HEIGHT: u32 = 256;

fn main() {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH, HEIGHT);
        WindowBuilder::new()
            .with_title("Conway's Game of Life")
            .with_min_inner_size(size)
            .with_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH / SCALE, HEIGHT / SCALE, surface_texture).expect("Unable to create pixel buffer")
    };

    let mut game = LifeBoard::gen(WIDTH as usize, HEIGHT as usize);

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            let mut frame = pixels.frame_mut();
            for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                let x = (i % (WIDTH as usize)) as i64;
                let y = (i / (WIDTH as usize)) as i64;
                if let Some(bool) = dbg!(game.is_cell_alive(x, y)) {
                    if bool {
                        pixel.copy_from_slice(&[0xff, 0, 0, 0xff]);
                    } else {
                        pixel.copy_from_slice(&[0xff, 0xff, 0xff, 0xff]);
                    }
                }
            }
            pixels.render().expect("Unable to render pixel buffer.");
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::ExitWithCode(0);
                return;
            } else if input.key_pressed(VirtualKeyCode::Space) {
                game.simulate();
                window.request_redraw();
                return;
            }
        }
    });
}