use std::process;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
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
    pub fn erase_points(&mut self, points: &[(i32, i32)]) {
        let sdl_points: Vec<Point> = points.iter().map(|p| Point::from(*p)).collect();
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        match self.canvas.draw_points(&sdl_points[..]) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error drawing points: {e}");
                process::exit(1);
            }
        }
        self.canvas.present();
    }
}
