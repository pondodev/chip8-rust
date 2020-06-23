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
