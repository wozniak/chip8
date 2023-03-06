use chip8::Chip8;
use minifb::{Key, Window, WindowOptions};
use std::{env, fs, time::{Instant, Duration}};

const KEYS: [Key; 16] = [
    Key::X,
    Key::Key1,
    Key::Key2,
    Key::Key3,
    Key::Q,
    Key::W,
    Key::E,
    Key::A,
    Key::S,
    Key::D,
    Key::Z,
    Key::C,
    Key::Key4,
    Key::R,
    Key::F,
    Key::V,
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    let rom = fs::read(env::args().nth(1).unwrap()).unwrap();

    let mut chip8 = Chip8::default();
    chip8.load_program(&rom)?;

    let mut buffer = [0u32; 64 * 32];
    let winopts = {
        let mut w = WindowOptions::default();
        w.scale = minifb::Scale::X8;
        w
    };
    let mut window = Window::new("CHIP-8", 64, 32, winopts).unwrap();
    window.limit_update_rate(None);
    let mut last_frame = Instant::now();
    let mut next = Instant::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        next += Duration::from_secs_f32(1./500.);
        chip8.cycle();
        // ~60Hz
        if last_frame.elapsed().as_micros() > 16600 {
            if chip8.dt > 0 {
                chip8.dt -= 1;
            }
            if chip8.st > 0 {
                chip8.st -= 1;
            }
            last_frame = Instant::now();
            // gen real window buffer
            for x in 0..64 {
                for y in 0..32 {
                    buffer[y * 64 + x] = if chip8.get_px(x as u16, y as u16) {
                        0xffffff
                    } else {
                        // fade out
                        if buffer[y * 64 + x] > 0 {
                            buffer[y * 64 + x] - 0x333333
                        } else {
                            0x000000
                        }
                    };
                }
            }

            for (i, key) in KEYS.iter().enumerate() {
                chip8.keys[i] = window.is_key_down(*key);
            }

            window.get_keys_pressed(minifb::KeyRepeat::Yes)
                .iter()
                .for_each(|key| match key {
                    _ => (),
                });
            window.update_with_buffer(&buffer, 64, 32)?;
        }
        std::thread::sleep(next - Instant::now());
    }
    Ok(())
}
