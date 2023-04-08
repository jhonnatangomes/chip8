use std::{io::stdin, process};

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
    virtual_screen: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
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
            virtual_screen: [[0; SCREEN_WIDTH]; SCREEN_HEIGHT],
            screen: Screen::new(),
        }
    }
    pub fn start(mut self) {
        // loop {
        // match self.screen.draw() {
        //     // MainLoopAction::Interrupt => break,
        //     // MainLoopAction::Continue => {}
        //     _ => (),
        // }
        loop {
            match self.screen.draw() {
                MainLoopAction::Interrupt => break,
                MainLoopAction::Continue => {}
            }
            self.run();
        }
        // }
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
        let mut points_to_draw = vec![];
        let mut points_to_erase = vec![];
        for i in 0..(sprite_height as u8) {
            for j in 0..8 {
                let screen_x = (vx + j) as usize;
                let screen_y = (vy + i) as usize;
                let current_pixel = &mut self.virtual_screen[screen_y][screen_x];
                let new_pixel = ((pixels_to_draw[i as usize] >> (7 - j)) & 1) ^ *current_pixel;
                // println!("{new_pixel}");
                if *current_pixel == 1 && new_pixel == 0 {
                    bit_erased = true;
                }
                *current_pixel = new_pixel;
                // println!("({}, {})", screen_x, screen_y);
                if new_pixel == 1 {
                    points_to_draw.push((screen_x as i32, screen_y as i32));
                } else {
                    points_to_erase.push((screen_x as i32, screen_y as i32));
                }
            }
        }
        if bit_erased {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
        self.screen.draw_points(&points_to_draw[..]);
        // self.screen.erase_points(&points_to_erase[..]);
    }
    fn run(&mut self) {
        println!("Press enter to read an instruction...");
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();
        if buffer.trim() == "q" {
            process::exit(0);
        }
        let instruction = self.next_instruction();
        match instruction {
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
        println!(" == Vm State == ");
        // println!("memory: {:?}", &self.memory[0x200..(0x200 + 132)]);
        println!(
            "bytes: {:?}",
            &self.memory[self.i_reg as usize..(self.i_reg + 16) as usize]
                .iter()
                .map(|x| format!("{:08b}", x))
                .collect::<Vec<_>>()
        );
        println!("registers: {:?}", self.registers);
        println!("I register: {:?}", self.i_reg);
        println!("pc: {:?}", self.pc);
        println!("virtual screen: {:?}", self.virtual_screen);
    }
    // pub fn run(&mut self) {}
}
