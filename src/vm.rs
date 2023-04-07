use crate::screen::{Screen, ScreenCanvas};

pub struct Vm {
    memory: [u8; 4096],
    registers: [u8; 16],
    i_reg: u16,
    delay_reg: u8,
    timer_reg: u8,
    pc: u16,
    sp: u8,
    stack: [u16; 16],
    screen_canvas: ScreenCanvas,
}

impl Vm {
    const HEX_SPRITES: [u8; 80] = [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    ];
    pub fn new(code: [u8; 3584]) -> Vm {
        Vm {
            memory: Vm::HEX_SPRITES
                .into_iter()
                .chain([0; 432].into_iter())
                .chain(code.into_iter())
                .collect::<Vec<u8>>()
                .try_into()
                .unwrap(),
            registers: [0; 16],
            i_reg: 0,
            delay_reg: 0,
            timer_reg: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            screen_canvas: Screen::init(),
        }
    }
    pub fn start(mut self) {}
}