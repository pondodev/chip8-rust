use std::env;
use std::fs;
use std::io;
use std::io::Read;

fn main() {
    let mut machine = Chip8::init();
    let args: Vec<String> = env::args().collect();

    machine.load_rom(&args[1]);
    machine.show_memory();
}

struct Chip8 {
    registers: [u8; 16],
    memory: [u8; 4096],
    index: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [u8; 16],
    video: [u32; 64 * 32],
    opcode: u16
}

impl Chip8 {
    fn init() -> Chip8 {
        Chip8 {
            registers: [0; 16],
            memory: [0; 4096],
            index: 0,
            pc: 0,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16],
            video: [0; 64 * 32],
            opcode: 0
        }
    }

    // debug utilities
    fn show_memory(&self) {
        let step = 64;
        for i in (0..self.memory.len()).step_by(step) {
            for j in 0..step {
                let mut to_print = format!("{:#X} ", self.memory[j + i]);
                let to_print = &to_print[2..to_print.len()];
                if to_print.chars().count() == 2 {
                    print!("0{:width$} ", to_print, width = 2);
                } else {
                    print!("{:width$} ", to_print, width = 2);
                }
            }
            println!();
        }
    }

    // general utilities
    fn load_rom(&mut self, file_name: &str) -> io::Result<()> {
        // read in file
        let mut file = fs::File::open(file_name)?;
        let file_length = file.metadata()?.len();
        let mut buffer = vec![0u8; file_length as usize];
        file.read(&mut buffer)?;

        // store rom in memory
        for i in 0..buffer.len() {
            self.memory[i + 0x200] = buffer[i]; // rom storage starts at 0x200 in memory
        }

        Ok(())
    }
}
