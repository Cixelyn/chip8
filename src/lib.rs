extern crate rand;

use rand::Rng;
use std::fmt;
use std::io::prelude::*;

pub const ROWS: usize = 32;
pub const COLS: usize = 64;

static FONT: [[u8; 5]; 16] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0], // 0
    [0x20, 0x60, 0x20, 0x20, 0x70], // 1
    [0xF0, 0x10, 0xF0, 0x80, 0xF0], // 2
    [0xF0, 0x10, 0xF0, 0x10, 0xF0], // 3
    [0x90, 0x90, 0xF0, 0x10, 0x10], // 4
    [0xF0, 0x80, 0xF0, 0x10, 0xF0], // 5
    [0xF0, 0x80, 0xF0, 0x90, 0xF0], // 6
    [0xF0, 0x10, 0x20, 0x40, 0x40], // 7
    [0xF0, 0x90, 0xF0, 0x90, 0xF0], // 8
    [0xF0, 0x90, 0xF0, 0x10, 0xF0], // 9
    [0xF0, 0x90, 0xF0, 0x90, 0x90], // A
    [0xE0, 0x90, 0xE0, 0x90, 0xE0], // B
    [0xF0, 0x80, 0x80, 0x80, 0xF0], // C
    [0xE0, 0x90, 0x90, 0x90, 0xE0], // D
    [0xF0, 0x80, 0xF0, 0x80, 0xF0], // E
    [0xF0, 0x80, 0xF0, 0x80, 0x80], // F
];

pub struct Cpu {
    pub pc: u16,
    pub reg: Box<[u8]>,
    pub vi: u16,
    pub dt: u8,
    pub st: u8,
    pub stack: Vec<u16>,
    pub mem: Box<[u8]>,
    pub vram: Box<[[bool; COLS]; ROWS]>,
}

impl fmt::Display for Cpu {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("CPU")
            .field("pc", &format_args!("0x{:04X}", &self.pc))
            .field("vi", &format_args!("0x{:04X}", &self.vi))
            .field("reg", &format_args!("{:X?}", &self.reg))
            .field("stack", &format_args!("{:X?}", &self.stack))
            .finish()
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut mem = Box::new([0; 4096]);

        for (idx, letter) in FONT.iter().enumerate() {
            let offset = 0x10 * idx;
            mem[offset..offset + 5].copy_from_slice(letter);
        }

        Cpu {
            pc: 0x200,
            vi: 0x0000,
            dt: 0,
            st: 0,
            reg: Box::new([0; 16]),
            mem: mem,
            vram: Box::new([[false; COLS]; ROWS]),
            stack: Vec::new(),
        }
    }

    pub fn load(&mut self, rom: impl Read) {
        for (idx, byte) in rom.bytes().enumerate() {
            self.mem[0x200 + idx] = byte.unwrap();
        }
    }

    pub fn draw(&self, buffer: &mut String) {
        for row in self.vram.iter() {
            for byte in row.iter() {
                buffer.push(if *byte { '▓' } else { ' ' });
            }
            buffer.push('\n');
        }
    }

    pub fn step(&mut self) {
        let op: u16 =
            ((self.mem[self.pc as usize] as u16) << 8) | (self.mem[(self.pc + 1) as usize] as u16);
        self.exec(op);
    }

    pub fn exec(&mut self, op: u16) {
        print!("{:04x} {:04x}  ", self.pc, op);

        // if op == 0x130E {
        //     panic!("dead");
        // }

        let byte = (op & 0x00FF) as u8;
        let addr = op & 0x0FFF;

        let op1 = ((op & 0xF000) >> 12) as usize;
        let op2 = ((op & 0x0F00) >> 8) as usize;
        let op3 = ((op & 0x00F0) >> 4) as usize;
        let op4 = ((op & 0x000F) >> 0) as usize;

        self.pc += 2;
        match (op1, op2, op3, op4) {
            (0x0, 0x0, 0xE, 0x0) => {
                println!("CLS");
                self.vram = Box::new([[false; COLS]; ROWS]);
            }
            (0x0, 0x0, 0xE, 0xE) => {
                println!("RET");
                self.pc = self.stack.pop().unwrap();
            }
            (0x1, _, _, _) => {
                println!("JP {:04x}", addr);
                self.pc = addr;
            }
            (0x2, _, _, _) => {
                println!("CALL {:04x}", addr);
                self.stack.push(self.pc);
                self.pc = addr;
            }
            (0x3, vx, _, _) => {
                println!("SE v{:X}, {}", vx, byte);
                if self.reg[vx] == byte {
                    self.pc += 2;
                }
            }
            (0x4, vx, _, _) => {
                println!("SNE v{:X}, {}", vx, byte);
                if self.reg[vx] != byte {
                    self.pc += 2
                }
            }
            (0x5, vx, vy, _) => {
                println!("SE v{}, v{}", vx, vy);
                if self.reg[vx] == self.reg[vy] {
                    self.pc += 2;
                }
            }
            (0x6, vx, _, _) => {
                println!("LD v{:X}, {}", vx, byte);
                self.reg[vx] = byte;
            }
            (0x7, vx, _, _) => {
                println!("ADD v{:X}, {}", vx, byte);
                self.reg[vx] = self.reg[vx].wrapping_add(byte);
            }
            (0x8, vx, vy, 0x0) => {
                println!("LD v{:X}, v{:X}", vx, vy);
                self.reg[vx] = self.reg[vy];
            }
            (0x8, vx, vy, 0x1) => {
                println!("OR v{:X}, v{:X}", vx, vy);
                self.reg[vx] |= self.reg[vy];
            }
            (0x8, vx, vy, 0x2) => {
                println!("AND v{:X}, v{:X}", vx, vy);
                self.reg[vx] &= self.reg[vy];
            }
            (0x8, vx, vy, 0x3) => {
                println!("XOR v{:X}, v{:X}", vx, vy);
                self.reg[vx] ^= self.reg[vy];
            }
            (0x8, vx, vy, 0x4) => {
                println!("ADD v{:X}, v{:X}", vx, vy);
                let (val, flag) = self.reg[vx].overflowing_add(self.reg[vy]);
                self.reg[vx] = val;
                self.reg[0xF] = flag as u8;
            }
            (0x8, vx, vy, 0x5) => {
                println!("SUB v{:X}, v{:X}", vx, vy);
                let (val, flag) = self.reg[vx].overflowing_sub(self.reg[vy]);
                self.reg[vx] = val;
                self.reg[0xF] = !flag as u8;
            }
            (0x8, vx, vy, 0x6) => {
                println!("SHR v{:X}, v{:X}", vx, vy);
                self.reg[0xF] = self.reg[vy] & 0x1;
                self.reg[vx] = self.reg[vy] >> 1;
            }
            (0x8, vx, vy, 0x7) => {
                println!("SUBN v{:X}, v{:X}", vx, vy);
                let (val, flag) = self.reg[vy].overflowing_sub(self.reg[vx]);
                self.reg[vx] = val;
                self.reg[0xF] = !flag as u8;
            }
            (0x8, vx, vy, 0xE) => {
                println!("SHL v{:X}, v{:X}", vx, vy);
                self.reg[0xF] = self.reg[vy] >> 7;
                self.reg[vx] = self.reg[vy] << 1;
            }
            (0xA, _, _, _) => {
                println!("LD i, {:X}", addr);
                self.vi = addr;
            }
            (0xC, vx, _, _) => {
                let rnd: u8 = rand::thread_rng().gen();
                self.reg[vx] = rnd & byte;
            }
            (0xD, vx, vy, n) => {
                println!("DRW v{}, v{}, {}", vx, vy, n);

                let i = self.vi as usize;
                let x = self.reg[vx] as usize;
                let y = self.reg[vy] as usize;

                let mut unset = false;

                for (dy, byte) in self.mem[i..i + n].iter().enumerate() {
                    let r = (y + dy) % ROWS;

                    for dx in 0..8 {
                        let c = (x + dx) % COLS;

                        let src = self.vram[r][c];
                        let wr = (byte & (0b1000_0000 >> dx)) > 0;

                        unset &= src && wr;
                        self.vram[r][c] = wr;
                    }
                }

                self.reg[0xF] = unset as u8;

                // let mut buf = String::new();
                // self.draw(&mut buf);
                // println!("\n{}", buf);
            }
            (0xF, vx, 0x1, 0xE) => {
                println!("ADD vi, v{}", vx);
                self.vi += self.reg[vx] as u16;
            }
            (0xF, vx, 0x3, 0x3) => {
                println!("BCD v{}", vx);
                let i = self.vi as usize;
                let x = self.reg[vx];
                self.mem[i] = x / 100;
                self.mem[i + 1] = (x % 100) / 10;
                self.mem[i + 2] = x % 10;
            }
            (0xF, vx, 0x0, 0x7) => {
                println!("LD v{}, dt", vx);
                self.reg[vx] = self.dt;
            }
            (0xF, vx, 0x1, 0x5) => {
                println!("LD dt, v{}", vx);
                self.dt = self.reg[vx];
            }
            (0xF, vx, 0x1, 0x8) => {
                println!("LD st, v{}", vx);
                self.st = self.reg[vx];
            }
            (0xF, x, 0x5, 0x5) => {
                println!("PUSHA {}", x);
                let i = self.vi as usize;
                self.mem[i..=i + x].copy_from_slice(&self.reg[..=x]);
                // self.vi += 1 + x as u16;
            }
            (0xF, x, 0x6, 0x5) => {
                println!("POPA {}", x);
                let i = self.vi as usize;
                self.reg[..=x].copy_from_slice(&self.mem[i..=i + x]);
                // self.vi += 1 + x as u16;
            }
            _ => {
                println!("NOOP");
                println!("{}", &self);
                panic!("Unknown Opcode");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
