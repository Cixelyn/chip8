extern crate chip8;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

const BLOCK_SIZE: usize = 10;

static FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000_u64 / 60);
static STEP_TIME: Duration = Duration::from_nanos(1_000_000_000_u64 / 500);

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 1 {
        panic!("Missing path to ROM");
    }

    let path = &args[1];
    println!("Loading {}", path);
    let f = File::open(path).unwrap();
    let reader = BufReader::new(f);

    let mut cpu = chip8::Cpu::new();
    cpu.load(reader);

    let cpu = Arc::new(RwLock::new(cpu));

    let mcpu = cpu.clone();
    thread::spawn(move || loop {
        let start = Instant::now();
        {
            mcpu.write().unwrap().step();
        }

        if start.elapsed() < STEP_TIME {
            std::thread::sleep(STEP_TIME - start.elapsed())
        }
    });

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "rust-sdl2 demo",
            (chip8::COLS * BLOCK_SIZE) as u32,
            (chip8::ROWS * BLOCK_SIZE) as u32,
        ).position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        let start = Instant::now();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        {
            canvas.set_draw_color(Color::RGB(255, 255, 255));
            for (y, row) in cpu.read().unwrap().vram.iter().enumerate() {
                for (x, byte) in row.iter().enumerate() {
                    if !byte {
                        continue;
                    }
                    canvas
                        .fill_rect(Rect::new(
                            (x * BLOCK_SIZE) as i32,
                            (y * BLOCK_SIZE) as i32,
                            BLOCK_SIZE as u32,
                            BLOCK_SIZE as u32,
                        )).unwrap();
                }
            }
        }

        // The rest of the game loop goes here...
        canvas.present();

        if start.elapsed() < FRAME_TIME {
            std::thread::sleep(FRAME_TIME - start.elapsed());
        }
    }
}
