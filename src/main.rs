mod core;
mod colour;

#[macro_use] extern crate gfx;

extern crate gfx_window_glutin;
extern crate glutin;

use std::env;
use std::time::SystemTime;
use gfx::traits::FactoryExt;
use gfx::Device;
use gfx_window_glutin as gfx_glutin;

pub type ColourFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

use crate::core::MACHINE_SCREEN_WIDTH;
use crate::core::MACHINE_SCREEN_HEIGHT;

const SCREEN_MULTIPLIER: u32 = 10;
const SCREEN_WIDTH: u32 = MACHINE_SCREEN_WIDTH as u32 * SCREEN_MULTIPLIER;
const SCREEN_HEIGHT: u32 = MACHINE_SCREEN_HEIGHT as u32 * SCREEN_MULTIPLIER;
const SCREEN_CLEAR_COLOUR: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

fn main() {
    let mut machine = core::Chip8::new();
    let args: Vec<String> = env::args().collect();

    match machine.load_rom(&args[1]) {
        Err(e) => println!("error: {}", e),
        Ok(()) => {
            let mut platform = Platform::new(
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
                machine
            );

            let white = colour::Colour::new(255, 255, 255);

            // set up windowing/graphics stuffs
            let events_loop = glutin::EventsLoop::new();
            let builder = glutin::WindowBuilder::new()
                .with_title("Chip8 Interpreter")
                .with_dimensions(SCREEN_WIDTH, SCREEN_HEIGHT)
                .with_vsync();

            let (window, mut device, mut factory, mut main_colour, mut main_depth) =
                gfx_glutin::init::<ColourFormat, DepthFormat>(builder, &events_loop);

            let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
            // read in the shaders
            let pso = factory.create_pipeline_simple(
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/vertex.glsl")),
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/fragment.glsl")),
                pipe::new()
            ).unwrap();

            let mut vertices: Vec<Vertex> = vec![];
            let mut indices: Vec<u16> = vec![];
            let (vertex_buffer, mut slice) = factory.create_vertex_buffer_with_slice(&vertices, &*indices);
            let mut data = pipe::Data {
                vbuf: vertex_buffer,
                out: main_colour
            };


            let mut last_cycle = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_micros();
            let mut running = true;

            while running {
                // main program loop
                platform.process_input();

                let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_micros();
                let duration = current_time - last_cycle;
                if duration > 10 {
                    last_cycle = current_time;

                    // handle window events
                    events_loop.poll_events(|glutin::Event::WindowEvent { window_id: _, event }| {
                        use glutin::WindowEvent::*;
                        match event {
                            Closed => running = false,
                            _ => (),
                        }
                    });

                    platform.machine.cycle();

                    // process video stuffs
                    encoder.clear(&data.out, SCREEN_CLEAR_COLOUR);
                    encoder.draw(&slice, &pso, &data);
                    encoder.flush(&mut device);
                    window.swap_buffers().unwrap();
                    device.cleanup();
                }
            }
        },
    };
}

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        colour: [f32; 3] = "a_Colour",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out: gfx::RenderTarget<ColourFormat> = "Target0",
    }
}

struct Platform {
    screen_width: u32,
    screen_height: u32,
    buffer: Vec<u8>,
    machine: core::Chip8
}

impl Platform {
    fn new(screen_width: u32, screen_height: u32, machine: core::Chip8) -> Platform {
        let buffer = vec![0; (screen_height * screen_width) as usize + 1];
        Platform {
            screen_width,
            screen_height,
            buffer,
            machine
        }
    }

    // we will map the key inputs as such:
    // 1 2 3 4
    // Q W E R
    // A S D F
    // Z X C V
    // this should hopefully fit enough people's needs, or at least just work for me
    fn process_input(&mut self) {
        // TODO: implement
    }
}
