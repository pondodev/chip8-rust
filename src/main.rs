use std::env;
use std::fs;
use std::io;
use std::io::Read;
use rand::Rng;
use std::env::join_paths;

fn main() {
    let mut machine = Chip8::init();
    let args: Vec<String> = env::args().collect();

    machine.load_rom(&args[1]);
    machine.show_memory();
}

// program consts
const PROGRAM_START_ADDRESS: usize = 0x200;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const FONT_SET_SIZE: usize = 80;
const FONT_SET_START_ADDRESS: usize = 0x50;
const FONT_SET: [u8; FONT_SET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

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
    video: [u32; SCREEN_WIDTH * SCREEN_HEIGHT],
    opcode: u16
}

impl Chip8 {
    fn init() -> Chip8 {
        let mut machine = Chip8 {
            registers: [0; 16],
            memory: [0; 4096],
            index: 0,
            pc: PROGRAM_START_ADDRESS as u16,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16],
            video: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            opcode: 0
        };

        // load fonts into memory
        for i in 0..FONT_SET_SIZE {
            machine.memory[FONT_SET_START_ADDRESS + i] = FONT_SET[i];
        }

        return machine;
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
            self.memory[i + PROGRAM_START_ADDRESS] = buffer[i];
        }

        Ok(())
    }

    fn get_random_number() -> u8 {
        return rand::thread_rng().gen_range(0, 256) as u8
    }

    fn get_addr(val: u16) -> u16 {
        return val & 0xFFF;
    }

    fn get_x_kk(val: u16) -> (u8, u8) {
        let x = ((val & 0x0F00) >> 8) as u8;
        let kk = (val & 0x00FF) as u8;

        return (x, kk)
    }

    fn get_x_y(val: u16) -> (u8, u8) {
        let x = ((val & 0x0F00) >> 8) as u8;
        let y = ((val & 0x00F0) >> 4) as u8;

        return (x, y)
    }
}

// opcode implementation
impl Chip8 {
    // CLS
    // clear screen
    fn op_00e0(&mut self) {
        self.video = [0; SCREEN_WIDTH * SCREEN_HEIGHT];
    }

    // RET
    // pop address off stack and return to it
    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    // JP addr
    // jump to addr
    fn op_1nnn(&mut self) {
        let address = Chip8::get_addr(self.opcode);
        self.pc = address;
    }

    // CALL addr
    // store next pc on stack and jump to addr
    fn op_2nnn(&mut self) {
        let address = Chip8::get_addr(self.opcode);
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = address;
    }

    // SE Vx, kk
    // skip next instruction if Vx == kk
    fn op_3xkk(&mut self) {
        let (vx, kk) = Chip8::get_x_kk(self.opcode);
        if self.registers[vx as usize] == kk {
            self.pc += 2;
        }
    }

    // SNE Vx, kk
    // skip next instruction if Vx != kk
    fn op_4xkk(&mut self) {
        let (vx, kk) = Chip8::get_x_kk(self.opcode);
        if self.registers[vx as usize] != kk {
            self.pc += 2;
        }
    }

    // SE Vx, Vy
    // skip next instruction if Vx == Vy
    fn op_5xy0(&mut self) {
        let (vx, vy) = Chip8::get_x_y(self.opcode);
        if self.registers[vx as usize] == self.registers[vy as usize] {
            self.pc += 2;
        }
    }

    // LD Vx, kk
    // load kk into Vx
    fn op_6xkk(&mut self) {
        let (vx, kk) = Chip8::get_x_kk(self.opcode);
        self.registers[vx as usize] = kk;
    }

    // ADD Vx, kk
    // add kk to Vx
    fn op_7xkk(&mut self) {
        let (vx, kk) = Chip8::get_x_kk(self.opcode);
        self.registers[vx as usize] += kk;
    }

    // LD Vx, Vy
    // load Vy into Vx
    fn op_8xy0(&mut self) {
        let (vx, vy) = Chip8::get_x_y(self.opcode);
        self.registers[vx as usize] = self.registers[vy as usize];
    }

    // OR Vx, Vy
    // Vx = Vx OR Vy
    fn op_8xy1(&mut self) {
        let (vx, vy) = Chip8::get_x_y(self.opcode);
        self.registers[vx as usize] |= self.registers[vy as usize];
    }

    // AND Vx, Vy
    // Vx = Vx AND Vy
    fn op_8xy2(&mut self) {
        let (vx, vy) = Chip8::get_x_y(self.opcode);
        self.registers[vx as usize] &= self.registers[vy as usize];
    }

    // XOR Vx, Vy
    // Vx = Vx XOR Vy
    fn op_8xy3(&mut self) {
        let (vx, vy) = Chip8::get_x_y(self.opcode);
        self.registers[vx as usize] ^= self.registers[vy as usize];
    }

    // ADD Vx, Vy
    // add Vy to Vx, set VF to carry
    fn op_8xy4(&mut self) {
        let (vx, vy) = Chip8::get_x_y(self.opcode) as (u16, u16);
        let result = self.registers[vx as usize] + self.registers[vy as usize];

        if result > 0xFF { // if there is overflow...
            self.registers[0xF] = 1; // ...set carry bit to 1...
        } else {
            self.registers[0xF] = 0; // ...else set to 0
        }

        self.registers[vx as usize] = result & 0xFF;
    }
}
