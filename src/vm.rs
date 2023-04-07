use crate::screen::{MainLoopAction, Screen, SCREEN_HEIGHT, SCREEN_WIDTH};

pub struct Vm {
    memory: [u8; 4096],
    registers: [u8; 16],
    i_reg: u16,
    delay_reg: u8,
    timer_reg: u8,
    pc: u16,
    sp: u8,
    stack: [u16; 16],
    virtual_screen: [[bool; SCREEN_HEIGHT]; SCREEN_WIDTH],
    screen: Screen,
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
    pub fn new(code: Vec<u8>) -> Vm {
        let len = code.len();
        let memory: [u8; 4096] = Vm::HEX_SPRITES
            .into_iter()
            .chain([0; 432].into_iter())
            .chain(code.into_iter())
            .chain(vec![0; 4096 - 512 - len].into_iter())
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap();
        // println!("code_start: {}", &memory[0x200]);
        Vm {
            memory,
            registers: [0; 16],
            i_reg: 0,
            delay_reg: 0,
            timer_reg: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            virtual_screen: [[false; SCREEN_HEIGHT]; SCREEN_WIDTH],
            screen: Screen::new(),
        }
    }
    pub fn start(mut self) {
        loop {
            match self.screen.draw() {
                MainLoopAction::Interrupt => break,
                MainLoopAction::Continue => {}
            }
            match self.next_instruction() {
                0x00E0 => self.screen.clear(),
                instruction if instruction >= 0x1000 && instruction <= 0x1FFF => {
                    self.pc = instruction & 0x0FFF
                }
                instruction if instruction >= 0x6000 && instruction <= 0x6FFF => {
                    let register_index = (instruction >> 8 & 0x000F) as usize;
                    let value = instruction & 0x00FF;
                    self.registers[register_index] = value as u8;
                }
                instruction if instruction >= 0x7000 && instruction <= 0x7FFF => {
                    let register_index = instruction >> 8 & 0x000F;
                    let value = instruction & 0x00FF;
                    self.registers[register_index as usize] += value as u8;
                }
                instruction if instruction >= 0xA000 && instruction <= 0xAFFF => {
                    self.i_reg = instruction & 0x0FFF
                }
                instruction if instruction >= 0xD000 && instruction <= 0xDFFF => {
                    self.draw_generic_sprite(instruction)
                }
                instruction => panic!("Unknown instruction: {instruction}"),
            }
        }
    }
    fn next_instruction(&mut self) -> u16 {
        let high_byte = self.memory[self.pc as usize];
        self.pc += 1;
        let low_byte = self.memory[self.pc as usize];
        self.pc += 1;
        ((high_byte as u16) << 8) | (low_byte as u16)
    }
    fn draw_generic_sprite(&mut self, instruction: u16) {
        let vx_register_index = instruction >> 8 & 0x000F;
        let vy_register_index = instruction >> 4 & 0x000F;
        let sprite_height = instruction & 0x000F;
        let pixels_to_draw = &self.memory[self.i_reg.into()..(self.i_reg + sprite_height).into()];
        let vx = self.registers[vx_register_index as usize];
        let vy = self.registers[vy_register_index as usize];
        let mut bit_erased = false;
        let mut points = vec![];
        for i in 0..(sprite_height as u8) {
            for j in 0..8 {
                let screen_x = (vx + i) as usize;
                let screen_y = (vy + j) as usize;
                let current_pixel_on = self.virtual_screen[screen_x][screen_y];
                let new_pixel_on = if j != 8 {
                    (pixels_to_draw[i as usize] >> (8 - j)) & 0x000F
                } else {
                    pixels_to_draw[i as usize] & 0x0F
                } != 0;
                if current_pixel_on && !new_pixel_on {
                    bit_erased = true;
                }
                self.virtual_screen[screen_x][screen_y] = new_pixel_on;
                if new_pixel_on {
                    points.push((screen_x as i32, screen_y as i32));
                }
            }
        }
        if bit_erased {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
        self.screen.draw_points(&points[..]);
    }
    // pub fn run(&mut self) {}
}
