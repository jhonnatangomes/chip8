use std::sync::mpsc;
use std::{process, thread};

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::EventPump;

pub struct Screen {
    canvas: Canvas<Window>,
    event_pump: EventPump,
}

pub enum MainLoopAction {
    Interrupt,
    Continue,
}

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const SCALING_FACTOR: usize = 9;

impl Screen {
    pub fn new() -> Screen {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window(
                "chip8",
                (SCREEN_WIDTH * SCALING_FACTOR) as u32,
                (SCREEN_HEIGHT * SCALING_FACTOR) as u32,
            )
            .position_centered()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();
        // canvas.set_scale(SCALING_FACTOR as f32, SCALING_FACTOR as f32);
        Screen {
            canvas,
            event_pump: sdl_context.event_pump().unwrap(),
        }
    }
    pub fn draw(&mut self) -> MainLoopAction {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return MainLoopAction::Interrupt,
                _ => {}
            }
        }
        MainLoopAction::Continue
    }
    pub fn clear(&mut self) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        self.canvas.present();
    }
    pub fn draw_points(&mut self, points: &[(i32, i32)]) {
        let sdl_rects: Vec<Rect> = points
            .iter()
            .map(|p| {
                Rect::from((
                    p.0 * SCALING_FACTOR as i32,
                    p.1 * SCALING_FACTOR as i32,
                    SCALING_FACTOR as u32,
                    SCALING_FACTOR as u32,
                ))
            })
            .collect();
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        match self.canvas.fill_rects(&sdl_rects[..]) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error drawing points: {e}");
                process::exit(1);
            }
        }
        self.canvas.present();
    }
    pub fn is_key_pressed(&mut self, key: u8) -> bool {
        let scancode = match key {
            1 => Scancode::Num1,
            2 => Scancode::Num2,
            3 => Scancode::Num3,
            0xC => Scancode::Num4,
            4 => Scancode::Q,
            5 => Scancode::W,
            6 => Scancode::E,
            0xD => Scancode::R,
            7 => Scancode::A,
            8 => Scancode::S,
            9 => Scancode::D,
            0xE => Scancode::F,
            0xA => Scancode::Z,
            0 => Scancode::X,
            0xB => Scancode::C,
            0xF => Scancode::V,
            _ => unreachable!(),
        };
        self.event_pump
            .keyboard_state()
            .is_scancode_pressed(scancode)
    }
    pub fn wait_for_keypress(&mut self) -> Option<u8> {
        loop {
            match self.event_pump.wait_event() {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return None,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(key) = keycode_to_u8(keycode) {
                        return Some(key);
                    }
                }
                _ => {}
            }
        }
    }
}

fn keycode_to_u8(keycode: Keycode) -> Option<u8> {
    match keycode {
        Keycode::Num1 => Some(1),
        Keycode::Num2 => Some(2),
        Keycode::Num3 => Some(3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(4),
        Keycode::W => Some(5),
        Keycode::E => Some(6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(7),
        Keycode::S => Some(8),
        Keycode::D => Some(9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
