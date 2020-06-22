mod core;

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

use crate::core::SCREEN_WIDTH;
use crate::core::SCREEN_HEIGHT;

fn main() {
    let mut machine = core::Chip8::new();
    let args: Vec<String> = env::args().collect();

    match machine.load_rom(&args[1]) {
        Err(e) => println!("error: {}", e),
        Ok(()) => {
            let mut platform = Platform::new(
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
                [ Colour::new(0, 0, 0), Colour::new(255, 255, 255) ],
                machine
            );

            let mut last_cycle = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_micros();
            'main: loop {
                // main program loop
                platform.process_input();

                let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_micros();
                let duration = current_time - last_cycle;
                if duration > 10 {
                    last_cycle = current_time;

                    platform.machine.cycle();
                    platform.process_video();
                }

                // TODO: actually handle exiting the program
                break 'main;
            }
        },
    };
}

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        colour: [f32; 2] = "a_Colour",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out: gfx::RenderTarget<ColourFormat> = "Target0",
    }
}

struct Platform {
    screen_width: usize,
    screen_height: usize,
    palette: [Colour; 2],
    buffer: Vec<u8>,
    machine: core::Chip8
}

impl Platform {
    fn new(screen_width: usize, screen_height: usize, palette: [Colour; 2], machine: core::Chip8) -> Platform {
        let buffer = vec![0; screen_height * screen_width + 1];
        Platform {
            screen_width,
            screen_height,
            palette,
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

    fn process_video(&mut self) {
        // TODO: implement
    }
}

pub struct Colour {
    red: u8,
    green: u8,
    blue: u8
}

impl Colour {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Colour {
            red,
            green,
            blue
        }
    }

    pub fn to_array(&self) -> [f32; 3] {
        let r = self.red as f32 / 256.0;
        let g = self.green as f32 / 256.0;
        let b = self.blue as f32 / 256.0;

        return [r, g, b];
    }
}
