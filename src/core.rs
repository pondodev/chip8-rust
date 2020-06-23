use std::fs;
use std::io;
use std::io::Read;
use rand::Rng;

// program consts
const PROGRAM_START_ADDRESS: usize = 0x200;
pub const MACHINE_SCREEN_WIDTH: usize = 64;
pub const MACHINE_SCREEN_HEIGHT: usize = 32;
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

pub struct Chip8 {
    registers: [u8; 16],
    memory: [u8; 4096],
    index: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    pub keypad: [u8; 16],
    pub video: [u32; (MACHINE_SCREEN_WIDTH * MACHINE_SCREEN_HEIGHT) + 1],
    opcode: u16
}

impl Chip8 {
    pub fn new() -> Chip8 {
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
            video: [0; (MACHINE_SCREEN_WIDTH * MACHINE_SCREEN_HEIGHT) + 1],
            opcode: 0
        };

        // load fonts into memory
        for i in 0..FONT_SET_SIZE {
            machine.memory[FONT_SET_START_ADDRESS + i] = FONT_SET[i];
        }

        return machine;
    }

    // general utilities
    pub fn load_rom(&mut self, file_name: &str) -> io::Result<()> {
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

    fn get_addr(&self) -> u16 {
        return self.opcode & 0xFFF;
    }

    fn get_x_kk(&self) -> (u8, u8) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;
        let kk = (self.opcode & 0x00FF) as u8;

        return (x, kk)
    }

    fn get_x(&self) -> u8 {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;

        return x
    }

    fn get_x_y(&self) -> (u8, u8) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;
        let y = ((self.opcode & 0x00F0) >> 4) as u8;

        return (x, y)
    }

    fn get_x_y_n(&self) -> (u8, u8, u8) {
        let x = ((self.opcode & 0x0F00) >> 8) as u8;
        let y = ((self.opcode & 0x00F0) >> 4) as u8;
        let n = (self.opcode & 0x000F) as u8;

        return (x, y, n)
    }

    fn get_nibbles(&self) -> (u8, u8, u8, u8) {
        let n1 = ((self.opcode & 0xF000) >> 12) as u8;
        let n2 = ((self.opcode & 0x0F00) >> 8) as u8;
        let n3 = ((self.opcode & 0x00F0) >> 4) as u8;
        let n4 = (self.opcode & 0x000F) as u8;

        return (n1, n2, n3, n4)
    }

    pub fn cycle(&mut self) {
        for _ in 0..50 {
            // load next instruction from memory
            let first_byte = (self.memory[self.pc as usize] as u16) << 8;
            let second_byte = self.memory[(self.pc + 1) as usize] as u16;
            self.opcode = first_byte | second_byte;

            // instruction execution time
            self.pc += 2;
            self.execute_instruction();
        }

        // decrement the timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn execute_instruction(&mut self) {
        let (n1, n2, n3, n4) = self.get_nibbles();
        match (n1, n2, n3, n4) {
            (0x0, 0x0, 0xE, 0x0) => self.op_00e0(),
            (0x0, 0x0, 0xE, 0xE) => self.op_00ee(),
            (0x1,   _,   _,   _) => self.op_1nnn(),
            (0x2,   _,   _,   _) => self.op_2nnn(),
            (0x3,   _,   _,   _) => self.op_3xkk(),
            (0x4,   _,   _,   _) => self.op_4xkk(),
            (0x5,   _,   _, 0x0) => self.op_5xy0(),
            (0x6,   _,   _,   _) => self.op_6xkk(),
            (0x7,   _,   _,   _) => self.op_7xkk(),
            (0x8,   _,   _, 0x0) => self.op_8xy0(),
            (0x8,   _,   _, 0x1) => self.op_8xy1(),
            (0x8,   _,   _, 0x2) => self.op_8xy2(),
            (0x8,   _,   _, 0x3) => self.op_8xy3(),
            (0x8,   _,   _, 0x4) => self.op_8xy4(),
            (0x8,   _,   _, 0x5) => self.op_8xy5(),
            (0x8,   _,   _, 0x6) => self.op_8xy6(),
            (0x8,   _,   _, 0x7) => self.op_8xy7(),
            (0x8,   _,   _, 0xE) => self.op_8xye(),
            (0x9,   _,   _, 0x0) => self.op_9xy0(),
            (0xA,   _,   _,   _) => self.op_annn(),
            (0xB,   _,   _,   _) => self.op_bnnn(),
            (0xC,   _,   _,   _) => self.op_cxkk(),
            (0xD,   _,   _,   _) => self.op_dxyn(),
            (0xE,   _, 0x9, 0xE) => self.op_ex9e(),
            (0xE,   _, 0xA, 0x1) => self.op_exa1(),
            (0xF,   _, 0x0, 0x7) => self.op_fx07(),
            (0xF,   _, 0x0, 0xA) => self.op_fx0a(),
            (0xF,   _, 0x1, 0x5) => self.op_fx15(),
            (0xF,   _, 0x1, 0x8) => self.op_fx18(),
            (0xF,   _, 0x1, 0xE) => self.op_fx1e(),
            (0xF,   _, 0x2, 0x9) => self.op_fx29(),
            (0xF,   _, 0x3, 0x3) => self.op_fx33(),
            (0xF,   _, 0x5, 0x5) => self.op_fx55(),
            (0xF,   _, 0x6, 0x5) => self.op_fx65(),
            (  _,   _,   _,   _) => println!("instruction not recognised: {}", self.opcode),
        }
    }
}

// opcode implementation
impl Chip8 {
    // CLS
    // clear screen
    fn op_00e0(&mut self) {
        self.video = [0; (MACHINE_SCREEN_WIDTH * MACHINE_SCREEN_HEIGHT) + 1];
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
        self.pc = self.get_addr();
    }

    // CALL addr
    // store next pc on stack and jump to addr
    fn op_2nnn(&mut self) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = self.get_addr();
    }

    // SE Vx, kk
    // skip next instruction if Vx == kk
    fn op_3xkk(&mut self) {
        let (vx, kk) = self.get_x_kk();
        if self.registers[vx as usize] == kk {
            self.pc += 2;
        }
    }

    // SNE Vx, kk
    // skip next instruction if Vx != kk
    fn op_4xkk(&mut self) {
        let (vx, kk) = self.get_x_kk();
        if self.registers[vx as usize] != kk {
            self.pc += 2;
        }
    }

    // SE Vx, Vy
    // skip next instruction if Vx == Vy
    fn op_5xy0(&mut self) {
        let (vx, vy) = self.get_x_y();
        if self.registers[vx as usize] == self.registers[vy as usize] {
            self.pc += 2;
        }
    }

    // LD Vx, kk
    // load kk into Vx
    fn op_6xkk(&mut self) {
        let (vx, kk) = self.get_x_kk();
        self.registers[vx as usize] = kk;
    }

    // ADD Vx, kk
    // add kk to Vx
    fn op_7xkk(&mut self) {
        let (vx, kk) = self.get_x_kk();
        self.registers[vx as usize] = (self.registers[vx as usize] as u16 + kk as u16) as u8;
    }

    // LD Vx, Vy
    // load Vy into Vx
    fn op_8xy0(&mut self) {
        let (vx, vy) = self.get_x_y();
        self.registers[vx as usize] = self.registers[vy as usize];
    }

    // OR Vx, Vy
    // Vx = Vx OR Vy
    fn op_8xy1(&mut self) {
        let (vx, vy) = self.get_x_y();
        self.registers[vx as usize] |= self.registers[vy as usize];
    }

    // AND Vx, Vy
    // Vx = Vx AND Vy
    fn op_8xy2(&mut self) {
        let (vx, vy) = self.get_x_y();
        self.registers[vx as usize] &= self.registers[vy as usize];
    }

    // XOR Vx, Vy
    // Vx = Vx XOR Vy
    fn op_8xy3(&mut self) {
        let (vx, vy) = self.get_x_y();
        self.registers[vx as usize] ^= self.registers[vy as usize];
    }

    // ADD Vx, Vy
    // add Vy to Vx, set VF to carry
    fn op_8xy4(&mut self) {
        let (vx, vy) = self.get_x_y();
        let result = self.registers[vx as usize] as u16 + self.registers[vy as usize] as u16;

        if result > 0xFF { // if there is overflow...
            self.registers[0xF] = 1; // ...set carry bit to 1...
        } else {
            self.registers[0xF] = 0; // ...else set to 0
        }

        self.registers[vx as usize] = (result & 0xFF) as u8;
    }

    // SUB Vx, Vy
    // subtract Vy from Vx, set VF to NOT borrow
    fn op_8xy5(&mut self) {
        let (vx, vy) = self.get_x_y();

        if self.registers[vx as usize] > self.registers[vy as usize] { // if there is no borrowing...
            self.registers[0xF] = 1; // ...set VF to 1...
        } else {
            self.registers[0xF] = 0; // ...else set to 0
        }

        // TODO: clean
        //self.registers[vx as usize] -= self.registers[vy as usize];
        self.registers[vx as usize] = self.registers[vx as usize].wrapping_sub(self.registers[vy as usize]);
    }

    // SHR Vx
    // shift Vx right one bit. store overflow in VF
    fn op_8xy6(&mut self) {
        let vx = self.get_x();
        self.registers[0xF] = self.registers[vx as usize] & 0b00000001;
        self.registers[vx as usize] >>= 1;
    }

    // SUBN Vx, Vy
    // subtract Vy from Vx, set VF to NOT borrow
    fn op_8xy7(&mut self) {
        let (vx, vy) = self.get_x_y();

        if self.registers[vy as usize] > self.registers[vx as usize] { // if there is no borrowing...
            self.registers[0xF] = 1; // ...set VF to 1...
        } else {
            self.registers[0xF] = 0; // ...else set to 0
        }

        // TODO: clean
        //self.registers[vx as usize] = self.registers[vy as usize] - self.registers[vx as usize];
        self.registers[vx as usize] = self.registers[vy as usize].wrapping_sub(self.registers[vx as usize]);
    }

    // SHL Vx {, Vy}
    // shift Vx left one bit. store overflow in VF
    fn op_8xye(&mut self) {
        let vx = self.get_x();
        self.registers[0xF] = (self.registers[vx as usize] & 0b10000000) >> 7;
        self.registers[vx as usize] <<= 1;
    }

    // SNE Vx, Vy
    // skip next instruction if Vx != Vy
    fn op_9xy0(&mut self) {
        let (vx, vy) = self.get_x_y();
        if self.registers[vx as usize] != self.registers[vy as usize] {
            self.pc += 2;
        }
    }

    // LD I, addr
    // load addr into I
    fn op_annn(&mut self) {
        self.index = self.get_addr();
    }

    // JP V0, addr
    // jump to addr + V0
    fn op_bnnn(&mut self) {
        self.pc = self.get_addr() + self.registers[0x0] as u16;
    }

    // RND Vx, kk
    // set Vx to random byte AND kk
    fn op_cxkk(&mut self) {
        let (vx, kk) = self.get_x_kk();
        self.registers[vx as usize] = Chip8::get_random_number() & kk;
    }

    // DRW Vx, Vy, n
    // get n bytes from memory starting at address I and display as sprite at Vx, Vy. sprite is
    // XOR'd onto screen, and if it causes any pixels to be set to 0 then VF is set to 1, otherwise
    // VF is set to 0
    fn op_dxyn(&mut self) {
        let (vx, vy, n) = self.get_x_y_n();

        // wrap around screen
        let x = self.registers[vx as usize] % MACHINE_SCREEN_WIDTH as u8;
        let y = self.registers[vy as usize] % MACHINE_SCREEN_HEIGHT as u8;

        // set VF to 0
        self.registers[0xF] = 0;

        // actually draw the sprite
        for row in 0..n {
            let sprite_byte = self.memory[(self.index + row as u16) as usize];
            for col in 0..8 {
                let sprite_pixel = sprite_byte & (0b10000000 >> col);
                let screen_pixel = &mut self.video[(y + row) as usize * MACHINE_SCREEN_WIDTH + (x + col) as usize];
                if sprite_pixel > 0 {
                    // collision detection
                    if *screen_pixel == 0xFFFFFFFF {
                        self.registers[0xF] = 1;
                    }

                    // XOR the pixels
                    *screen_pixel ^= 0xFFFFFFFF;
                }
            }
        }
    }

    // SKP Vx
    // skip next instruction if key code stored in Vx is pressed
    fn op_ex9e(&mut self) {
        let vx = self.get_x();
        let key = self.registers[vx as usize] as usize;
        if self.keypad[key] == 1 {
            self.pc += 2;
        }
    }

    // SKNP Vx
    // skip next instruction if key code stored in Vx is not pressed
    fn op_exa1(&mut self) {
        let vx = self.get_x();
        let key = self.registers[vx as usize] as usize;
        if self.keypad[key] != 1 {
            self.pc += 2;
        }
    }

    // LD Vx, DT
    // set Vx to delay timer value
    fn op_fx07(&mut self) {
        let vx = self.get_x();
        self.registers[vx as usize] = self.delay_timer;
    }

    // LD Vx, K
    // wait for key press, store key code in Vx
    fn op_fx0a(&mut self) {
        let vx = self.get_x();
        let val = &mut self.registers[vx as usize];

        // store key press value in vx
        if self.keypad[0x0] == 1 {
            *val = 0x0;
        } else if self.keypad[0x1] == 1 {
            *val = 0x1;
        } else if self.keypad[0x2] == 1 {
            *val = 0x2;
        } else if self.keypad[0x3] == 1 {
            *val = 0x3;
        } else if self.keypad[0x4] == 1 {
            *val = 0x4;
        } else if self.keypad[0x5] == 1 {
            *val = 0x5;
        } else if self.keypad[0x6] == 1 {
            *val = 0x6;
        } else if self.keypad[0x7] == 1 {
            *val = 0x7;
        } else if self.keypad[0x8] == 1 {
            *val = 0x8;
        } else if self.keypad[0x9] == 1 {
            *val = 0x9;
        } else if self.keypad[0xA] == 1 {
            *val = 0xA;
        } else if self.keypad[0xB] == 1 {
            *val = 0xB;
        } else if self.keypad[0xC] == 1 {
            *val = 0xC;
        } else if self.keypad[0xD] == 1 {
            *val = 0xD;
        } else if self.keypad[0xE] == 1 {
            *val = 0xE;
        } else if self.keypad[0xF] == 1 {
            *val = 0xF;
        } else {
            self.pc -= 2; // loop back to same instruction if no key was pressed
        }
    }

    // LD DT, Vx
    // set delay timer = Vx
    fn op_fx15(&mut self) {
        let vx = self.get_x();
        self.delay_timer = self.registers[vx as usize];
    }

    // LD ST, Vx
    // set sound timer = Vx
    fn op_fx18(&mut self) {
        let vx = self.get_x();
        self.sound_timer = self.registers[vx as usize];
    }

    // LD ADD I, Vx
    // add Vx to I
    fn op_fx1e(&mut self) {
        let vx = self.get_x();
        self.index += self.registers[vx as usize] as u16;
    }

    // LD F, Vx
    // set I = location of sprite for digit Vx
    fn op_fx29(&mut self) {
        let vx = self.get_x();
        let digit = self.registers[vx as usize];

        self.index = FONT_SET_START_ADDRESS as u16 + (5 * digit) as u16;
    }

    // LD B, Vx
    // store BCD representation of Vx in I, I + 1, and I + 2
    fn op_fx33(&mut self) {
        let vx = self.get_x();
        let mut value = self.registers[vx as usize];

        // ones place
        self.memory[(self.index + 2) as usize] = value % 10;
        value /= 10;

        // tens place
        self.memory[(self.index + 1) as usize] = value % 10;
        value /= 10;

        // hundreds place
        self.memory[self.index as usize] = value % 10;
    }

    // LD [I], Vx
    // store registers V0 -> Vx in memory starting at index
    fn op_fx55(&mut self) {
        let vx = self.get_x();
        for i in 0..vx + 1 {
            self.memory[(self.index + i as u16) as usize] = self.registers[i as usize];
        }
    }

    // LD Vx, [I]
    // read in values to V0 -> Vx starting a index in memory
    fn op_fx65(&mut self) {
        let vx = self.get_x();
        for i in 0..vx + 1 {
            self.registers[i as usize] = self.memory[(self.index + i as u16) as usize];
        }
    }
}

