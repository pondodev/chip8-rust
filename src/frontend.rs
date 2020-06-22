#[macro_use] extern crate gfx;

extern crate gfx_window_glutin;
extern crate glutin;

use gfx::traits::FactoryExt;
use gfx::Device;
use gfx_window_glutin as gfx_glutin;

pub type ColourFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

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

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        colour: [f32; 2] = "a_Colour"
    }

    pipeline pip {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out: gfx::RenderTarget<ColourFormat> = "Target0"
    }
}

pub struct FrontEnd {
    buffer: Vector<u8>,
    palette: [Colour; 2],
    window_width: usize,
    window_height: usize
}

impl FrontEnd {
    pub fn new(palette: [Colour; 2], window_width: usize, window_height: usize) -> Self {
        FrontEnd {
            buffer: vec![0; window_width * window_height],
            palette,
            window_height,
            window_width
        }
    }
}