use std::io::prelude::*;
use std::io::BufReader;
use std::fmt;
use std::fs::File;

#[allow(dead_code)]
struct Cpu {
    pc: usize,
    reg: Box<[u8]>,
    vi: u16,
    stack: Vec<u8>,
    mem: Box<[u8]>,
    vram: Box<[[bool; 64]; 32]>,
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
            vram: Box::new([[false; 64]; 32]),
            stack: Vec::new(),
        }
    }

    pub fn load(&mut self, rom: impl Read) {
        for (idx, byte) in rom.bytes().enumerate() {
            self.mem[0x200 + idx] = byte.unwrap();
        }
    }


    pub fn tick(&mut self) {
        let op: u16 = ((self.mem[self.pc] as u16) << 8) |
            (self.mem[self.pc + 1] as u16);

        self.pc += 2;
        self.exec(op);
    }

    pub fn exec(&mut self, op: u16) {
        println!("Running: {:04X}", op);

        match op {
            0x00E0 => println!("clearing the screen"),
            0x6000 ..= 0x6FFF => { // Store n in r
                let n = (op & 0x00FF) as u8;
                let r = (op & 0x0F00) >> 8;
                self.reg[r as usize] = n;
                println!("  v{:X} = {}", r, n);
            },
            _ => println!("  unknown"),
        }
    }

}


fn main() {
    let mut cpu = Cpu::new();
    let f = File::open("./roms/bc_test.ch8").unwrap();
    let reader = BufReader::new(f);
    cpu.load(reader);
    cpu.tick();
    cpu.tick();
    cpu.tick();
    cpu.tick();
    cpu.tick();
    cpu.tick();
    cpu.tick();
    cpu.tick();
    cpu.tick();
    cpu.tick();
    println!("{}", cpu);
}
