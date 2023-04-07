use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub type ScreenCanvas = Canvas<Window>;

pub struct Screen {}

impl Screen {
    pub fn init() -> ScreenCanvas {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("chip8", 64 * 9, 32 * 9)
            .position_centered()
            .build()
            .unwrap();

        window.into_canvas().build().unwrap()

        // let mut event_pump = sdl_context.event_pump().unwrap();
        // loop {
        //     canvas.clear();
        //     canvas.present();
        //     for event in event_pump.poll_iter() {
        //         match event {
        //             Event::Quit { .. }
        //             | Event::KeyDown {
        //                 keycode: Some(Keycode::Escape),
        //                 ..
        //             } => break,
        //             _ => {}
        //         }
        //     }
        // }
    }
}
