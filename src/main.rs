use std::io::prelude::*;
use std::io::BufReader;
use std::fmt;
use std::fs::File;

const ROWS: usize = 32;
const COLS: usize = 8;

#[allow(dead_code)]
struct Cpu {
    pc: u16,
    reg: Box<[u8]>,
    vi: u16,
    stack: Vec<u8>,
    mem: Box<[u8]>,
    vram: Box<[[u8; COLS]; ROWS]>,
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
        Cpu {
            pc: 0x200,
            vi: 0x0000,
            reg: Box::new([0; 16]),
            mem: Box::new([0; 4096]),
            vram: Box::new([[0; COLS]; ROWS]),
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
            for col in row {
                for i in (0..8).rev() {
                    buffer.push(if (col & (1 << i)) > 0 {'▓'} else {' '});
                }
            }
            buffer.push('\n');
        }
    }


    pub fn tick(&mut self) {
        let op: u16 = ((self.mem[self.pc as usize] as u16) << 8) |
            (self.mem[(self.pc + 1) as usize] as u16);
        self.exec(op);
    }

    pub fn exec(&mut self, op: u16) {
        print!("{:04x} {:04x}  ", self.pc, op);

        if op == 0x130E {
            panic!("dead");
        }

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
                self.vram = Box::new([[0; COLS]; ROWS]);
            }
            (0x1, _, _, _) => {
                println!("JP {:04x}", addr);
                self.pc = addr;
            },
            (0x3, vx, _, _) => {
                println!("SE v{:X}, {}", vx, byte);
                if self.reg[vx] == byte {
                    self.pc += 2;
                }
            },
            (0x4, vx, _, _) => {
                println!("SNE v{:X}, {}", vx, byte);
                if self.reg[vx] != byte {
                    self.pc += 2
                }
            },
            (0x5, vx, vy, _) => {
                println!("SE v{}, v{}", vx, vy);
                if self.reg[vx] == self.reg[vy] {
                    self.pc += 2;
                }
            },
            (0x6, vx, _, _) => {
                println!("LD v{:X}, {}", vx, byte);
                self.reg[vx] = byte;
            },
            (0x7, vx, _, _) => {
                println!("ADD v{:X}, {}", vx, byte);
                self.reg[vx] += byte;
            },
            (0x8, vx, vy, 0x1) => {
                println!("OR v{:X}, v{:X}", vx, vy);
                self.reg[vx] |= self.reg[vy];
            },
            (0x8, vx, vy, 0x2) => {
                println!("AND v{:X}, v{:X}", vx, vy);
                self.reg[vx] &= self.reg[vy];
            },
            (0x8, vx, vy, 0x3) => {
                println!("XOR v{:X}, v{:X}", vx, vy);
                self.reg[vx] ^= self.reg[vy];
            },
            (0x8, vx, vy, 0x5) => {
                println!("SUB v{:X}, v{:X}", vx, vy);
                let (val, flag) = self.reg[vx].overflowing_sub(self.reg[vy]);
                self.reg[vx] = val;
                self.reg[0xF] = !flag as u8;
            },
            (0x8, vx, vy, 0x6) => {
                println!("SHR v{:X}, v{:X}", vx, vy);
                self.reg[0xF] = self.reg[vy] & 0x1;
                self.reg[vx] = self.reg[vy] >> 1;
            },
            (0x8, vx, vy, 0x7) => {
                println!("SUBN v{:X}, v{:X}", vx, vy);
                let (val, flag) = self.reg[vy].overflowing_sub(self.reg[vx]);
                self.reg[vx] = val;
                self.reg[0xF] = !flag as u8;
            },
            (0x8, vx, vy, 0xE) => {
                println!("SHL v{:X}, v{:X}", vx, vy);
                self.reg[0xF] = self.reg[vy] >> 7;
                self.reg[vx] = self.reg[vy] << 1;
            },
            (0xA, _, _, _) => {
                println!("LD i, {:X}", addr);
                self.vi = addr;
            },
            (0xD, vx, vy, n) => {
                println!("DRW v{}, v{}, {}", vx, vy, n);

                let i = self.vi as usize;

                let x = self.reg[vx] as usize;
                let y = self.reg[vy] as usize;

                let mut unset = false;
                for (dy, byte) in self.mem[i .. i + n].iter().enumerate() {

                    let r = (y + dy) % ROWS;
                    let c = x % COLS;
                    let src = self.vram[r][c];

                    unset &= (src & byte) > 0;
                    self.vram[r][c] = src ^ byte;
                }

                self.reg[0xF] = unset as u8;

                let mut buf = String::new();
                self.draw(&mut buf);
                println!("\n{}", buf);

            },
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
            },
            (0xF, x, 0x5, 0x5) => {
                println!("PUSHA {}", x);
                let i = self.vi as usize;
                self.mem[i ..= i + x].copy_from_slice(&self.reg[..= x]);
                // self.vi += 1 + vx as u16;
            },
            (0xF, x, 0x6, 0x5) => {
                println!("POPA {}", x);
                let i = self.vi as usize;
                self.reg[..=x].copy_from_slice(&self.mem[i ..= i + x]);
                // self.vi += 1 + vx as u16;
            },
            _ => {
                println!("NOOP");
                println!("{}", &self);
                panic!("Unknown Opcode");
            },
        }
    }

}


fn main() {
    let mut cpu = Cpu::new();
    let f = File::open("./roms/bc_test.ch8").unwrap();
    let reader = BufReader::new(f);
    cpu.load(reader);

    loop{ 
        cpu.tick();
    }
}
