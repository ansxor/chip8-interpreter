extern crate rand;
use rand::Rng;

// use std::fs::File;
// use std::io::BufReader;
// use std::io::prelude::*;

pub const DISPLAY_X: usize = 64;
pub const DISPLAY_Y: usize = 32;
pub const VRAM_SIZE: usize = DISPLAY_Y;
pub const RAM_SIZE: usize = (0xFFF - 0x200) / 2;
pub const PIXEL_SIZE: usize = 16;
pub const PIXEL_PAD_SIZE: usize = PIXEL_SIZE / 2;
pub const WINDOW_X_SIZE: usize = DISPLAY_X * PIXEL_SIZE + PIXEL_PAD_SIZE * 2;
pub const WINDOW_Y_SIZE: usize = DISPLAY_Y * PIXEL_SIZE + PIXEL_PAD_SIZE * 2;

pub struct Program {
    vram: [u64; VRAM_SIZE],
    ram: [u8; RAM_SIZE],
    // sram: [u8; SRAM_SIZE],
    pub stack: [usize; 0x10],
    pub stack_position: usize,
    vars: [u8; 0x10],
    i: u16,
    keyboard: [bool; 0x10],
}

impl Program {
    fn get_ins(&mut self, pos: usize) -> u16 {
        return ((self.ram[pos] as u16) << 8) + ((self.ram[pos + 1]) as u16);
    }

    fn get_cur_ins(&mut self) -> u16 {
        return self.get_ins(self.stack[self.stack_position]);
    }

    fn get_nibbles(&mut self, pos: usize, shift: u16, nibbles: usize) -> u16 {
        let ins: u16 = self.get_ins(pos);
        return (ins >> (4 * shift)) & ((0x10 as u16).pow(nibbles as u32) - 1) as u16;
    }

    fn get_cur_nibbles(&mut self, shift: u16, nibbles: usize) -> u16 {
        return self.get_nibbles(self.stack[self.stack_position], shift, nibbles);
    }

    fn get_nibble(&self, pos: usize, nibble: usize) -> u8 {
        return (self.ram[pos + nibble / 2] as u8) >> (4 * ((nibble + 1) % 2)) & 0xF;
    }

    fn get_cur_nibble(&self, nibble: usize) -> u8 {
        return self.get_nibble(self.stack[self.stack_position], nibble);
    }

    pub fn set_ins(&mut self, pos: usize, val: u16) {
        self.ram[pos] = ((val >> 8) & 0xFF) as u8;
        self.ram[pos + 1] = ((val) & 0xFF) as u8;
    }

    pub fn run_cycle(&mut self) {
        if self.stack[self.stack_position] >= (RAM_SIZE - 1) {
            return;
        }
        let ins: u16 = self.get_cur_ins();
        let ins_mask: u8 = self.get_cur_nibble(0);

        match ins_mask {
            0x0 => {
                match ins {
                    // Clear VRAM
                    0xE0 => {
                        self.vram = [0; 32];
                    }
                    // Return from subroutine
                    0xEE => {
                        self.stack_position = self.stack_position - 1;
                        // curpos = &mut self.stack[self.stackPosition];
                    }
                    _ => {}
                }
            }
            // jump to position 0x1nnn
            0x1 => self.stack[self.stack_position] = (self.get_cur_nibbles(0, 3) * 2 - 2) as usize,
            // call subroutine 0x2nnn
            0x2 => {
                self.stack_position = self.stack_position + 1;
                self.stack[self.stack_position] = (self.get_cur_nibbles(0, 3) * 2 - 2) as usize;
            }
            // skip next instruction if Vx == kk 3xkk
            0x3 => {
                if self.vars[self.get_cur_nibble(1) as usize] == self.get_cur_nibbles(0, 2) as u8 {
                    self.stack[self.stack_position] = self.stack[self.stack_position] + 2;
                }
            }
            // skip next instruction if Vx ~= kk 4xkk
            0x4 => {
                if self.vars[self.get_cur_nibble(1) as usize] != self.get_cur_nibbles(0, 2) as u8 {
                    self.stack[self.stack_position] = self.stack[self.stack_position] + 2;
                }
            }
            // skip next instruction if Vx == Vy 5xy0
            0x5 => {
                if self.vars[self.get_cur_nibble(1) as usize]
                    == self.vars[self.get_cur_nibble(2) as usize]
                {
                    self.stack[self.stack_position] = self.stack[self.stack_position] + 2;
                }
            }
            // put kk into Vx 6xkk
            0x6 => self.vars[self.get_cur_nibble(1) as usize] = self.get_cur_nibbles(0, 2) as u8,
            // adds k to Vx 7xkk
            0x7 => {
                self.vars[self.get_cur_nibble(1) as usize] += self.get_cur_nibbles(0, 2) as u8;
            }
            0x8 => {
                let var = self.get_cur_nibble(1) as usize;
                let varval = self.vars[var];
                let value = self.vars[self.get_cur_nibble(2) as usize];
                match self.get_cur_nibble(3) {
                    1 => self.vars[var] |= value,
                    2 => self.vars[var] &= value,
                    3 => self.vars[var] ^= value,
                    4 => {
                        self.vars[var] += value;
                        self.vars[0xF] = (varval > self.vars[var]) as u8;
                    }
                    5 => {
                        self.vars[var] -= value;
                        self.vars[0xF] = (varval < self.vars[var]) as u8;
                    }
                    6 => {
                        self.vars[0xF] = value & 1;
                        self.vars[var] = value >> 1;
                    }
                    0xE => {
                        self.vars[0xF] = (value >> 7) & 1;
                        self.vars[var] = value << 1;
                    }
                    _ => {}
                }
            }
            // skip next instruction if Vx != Vy 9xy0
            0x9 => {
                if self.vars[self.get_cur_nibble(1) as usize]
                    != self.vars[self.get_cur_nibble(2) as usize]
                {
                    self.stack[self.stack_position] = self.stack[self.stack_position] + 2;
                }
            }
            // sets I to nnn Annn
            0xA => self.i = self.get_cur_nibbles(0, 3),
            // jump to nnn + V0 Bnnn
            0xB => {
                self.stack[self.stack_position] =
                    (self.vars[0] as usize) + (self.get_cur_nibbles(0, 3) as usize)
            }
            // sets Vx to random byte AND kk Cxkk
            0xC => {
                self.vars[self.get_cur_nibble(1) as usize] =
                    (rand::thread_rng().gen_range(0, 256) & self.get_cur_nibbles(0, 2)) as u8
            }
            // blits sprite onto screen at pos Vx Vy with size of n Dxyn
            0xD => {
                let x: u8 = self.vars[self.get_cur_nibble(1) as usize] % DISPLAY_X as u8;
                let y: u8 = self.vars[self.get_cur_nibble(2) as usize] % DISPLAY_Y as u8;
                let mut h: u8 = self.get_cur_nibble(3);

                if (y + h) > DISPLAY_Y as u8 {
                    h = DISPLAY_Y as u8 - y;
                }

                for i in 0..h {
                    let z: u64 = (self.ram[(self.i as u8 + i) as usize] as u64) << (56 - x);
                    self.vram[(i + y) as usize] ^= z;
                }
            }
            0xE => {
                let subcommand = self.get_cur_nibbles(0, 2);
                let key = self.keyboard[self.get_cur_nibble(1) as usize];

                match subcommand {
                    // skip if Kx is pressed
                    0x9E => {
                        if key {
                            self.stack[self.stack_position] = self.stack[self.stack_position] + 2;
                        }
                    }
                    // skip if Kx isn't pressed
                    0xA1 => {
                        if !key {
                            self.stack[self.stack_position] = self.stack[self.stack_position] + 2;
                        }
                    }
                    _ => {}
                }
            }
            0xF => {
                let subcommand = self.get_cur_nibbles(0, 2);
                let var = self.get_cur_nibble(1) as usize;

                match subcommand {
                    // adds Vx to I Fx1E
                    0x1E => self.i = self.i + self.vars[var] as u16,
                    // sets I to Vx Fx29
                    0x29 => self.i = self.vars[var] as u16,
                    0x33 => {
                        self.ram[self.i as usize] = self.vars[var] / 100;
                        self.ram[(self.i + 1) as usize] = self.vars[var] / 10;
                        self.ram[(self.i + 2) as usize] = self.vars[var] / 1;
                    }
                    // store V0 to VX starting at I in memory
                    0x55 => {
                        for x in 0..var {
                            self.ram[(self.i + x as u16) as usize] = self.vars[x as usize];
                        }
                    }
                    // read V0 to Vx starting at I in memory Fx65
                    0x65 => {
                        for x in 0..var {
                            self.vars[x as usize] = self.ram[(self.i + x as u16) as usize];
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        self.stack[self.stack_position] = self.stack[self.stack_position] + 2;
    }

    pub fn draw_output(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
        for i in 0..DISPLAY_Y {
            for j in 0..DISPLAY_X {
                let rect = sdl2::rect::Rect::new(
                    (j * PIXEL_SIZE + PIXEL_PAD_SIZE) as i32,
                    (i as usize * PIXEL_SIZE + PIXEL_PAD_SIZE) as i32,
                    PIXEL_SIZE as u32,
                    PIXEL_SIZE as u32,
                );
                if self.vram[i] & (1 << (DISPLAY_X - j - 1)) != 0 {
                    let _drawrect = canvas.fill_rect(rect).unwrap();
                }
            }
        }
    }

    // pub fn load_file(&mut self, filename: &str) {
    //     let file = File::open(filename).unwrap();
    //     let mut buf_reader = BufReader::new(file);
    // }
}

impl Default for Program {
    fn default() -> Self {
        Program {
            i: 0,
            keyboard: [false; 0x10],
            ram: [0; RAM_SIZE],
            vram: [0; VRAM_SIZE],
            stack: [0; 0x10],
            stack_position: 0,
            vars: [0; 0x10],
        }
    }
}
