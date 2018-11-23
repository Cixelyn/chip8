extern crate chip8;
extern crate sdl2;

use sdl2::audio::{AudioCallback, AudioSpecDesired, AudioStatus};
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

static KEY_MAP: [Keycode; 16] = [
    Keycode::X,
    Keycode::Num1,
    Keycode::Num2,
    Keycode::Num3,
    Keycode::Q,
    Keycode::W,
    Keycode::E,
    Keycode::A,
    Keycode::S,
    Keycode::D,
    Keycode::Z,
    Keycode::C,
    Keycode::Num4,
    Keycode::R,
    Keycode::F,
    Keycode::V,
];

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("Missing path to ROM");
    }

    let path = &args[1];
    println!("Loading {}", path);
    let f = File::open(path).unwrap();
    let reader = BufReader::new(f);

    let mut cpu = chip8::cpu::Cpu::new();
    cpu.load(reader);

    let cpu = Arc::new(RwLock::new(cpu));

    let sdl_context = sdl2::init().unwrap();

    // video subsystem
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "cixelyn/chip8",
            (chip8::cpu::COLS * BLOCK_SIZE) as u32,
            (chip8::cpu::ROWS * BLOCK_SIZE) as u32,
        ).position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    // audio subsystem
    let audio_subsystem = sdl_context.audio().unwrap();
    let audio_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };

    let device = audio_subsystem
        .open_playback(None, &audio_spec, |spec| {
            // Show obtained AudioSpec
            println!("{:?}", spec);

            // initialize the audio callback
            chip8::sound::SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            }
        }).unwrap();

    device.pause();

    let mut event_pump = sdl_context.event_pump().unwrap();

    // cpu subsystem
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

    'running: loop {
        let start = Instant::now();

        // Deal w/ Timers
        {
            if cpu.read().unwrap().st > 0 {
                cpu.write().unwrap().st -= 1;
                if device.status() == AudioStatus::Paused {
                    device.resume();
                }
            } else {
                if device.status() == AudioStatus::Playing {
                    device.pause();
                }
            }

            if cpu.read().unwrap().dt > 0 {
                cpu.write().unwrap().dt -= 1;
            }
        }

        // Set key registers
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => if let Some(idx) = KEY_MAP.iter().position(|&idx| idx == key) {
                    cpu.write().unwrap().key[idx as usize] = true;
                },
                Event::KeyUp {
                    keycode: Some(key), ..
                } => if let Some(idx) = KEY_MAP.iter().position(|&idx| idx == key) {
                    cpu.write().unwrap().key[idx as usize] = false;
                },
                _ => {}
            }
        }

        // Draw the screen
        {
            for (y, row) in cpu.read().unwrap().vram.iter().enumerate() {
                for (x, byte) in row.iter().enumerate() {
                    if *byte {
                        canvas.set_draw_color(Color::RGB(255, 255, 255));
                    } else {
                        canvas.set_draw_color(Color::RGB(0, 0, 0));
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
        canvas.present();

        if start.elapsed() < FRAME_TIME {
            std::thread::sleep(FRAME_TIME - start.elapsed());
        }
    }
}
