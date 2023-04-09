use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use rand::random;

use crate::{
    audio::Audio,
    screen::{MainLoopAction, Screen, SCREEN_HEIGHT, SCREEN_WIDTH},
};

pub struct Vm {
    memory: [u8; 4096],
    registers: [u8; 16],
    i_reg: u16,
    delay_reg: Arc<Mutex<u8>>,
    sound_reg: Arc<Mutex<u8>>,
    pc: u16,
    sp: u8,
    stack: [u16; 16],
    virtual_screen: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
    screen: Screen,
    audio: Arc<Audio>,
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
        Vm {
            memory,
            registers: [0; 16],
            i_reg: 0,
            delay_reg: Arc::new(Mutex::new(0)),
            sound_reg: Arc::new(Mutex::new(0)),
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            virtual_screen: [[0; SCREEN_WIDTH]; SCREEN_HEIGHT],
            screen: Screen::new(),
            audio: Arc::new(Audio::new()),
        }
    }
    pub fn start(mut self) {
        let sound_reg = Arc::clone(&self.sound_reg);
        let delay_reg = Arc::clone(&self.delay_reg);
        let audio = Arc::clone(&self.audio);
        thread::spawn(move || loop {
            let mut sound_reg = sound_reg.lock().unwrap();
            let mut delay_reg = delay_reg.lock().unwrap();
            if *sound_reg > 0 {
                audio.play();
                *sound_reg -= 1;
            }
            if *delay_reg > 0 {
                *delay_reg -= 1;
            }
            thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        });
        loop {
            match self.screen.draw() {
                MainLoopAction::Interrupt => break,
                MainLoopAction::Continue => {}
            }
            match self.run() {
                MainLoopAction::Interrupt => break,
                MainLoopAction::Continue => {}
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
        for i in 0..(sprite_height as u8) {
            for j in 0..8 {
                let screen_x = ((vx + j) % SCREEN_WIDTH as u8) as usize;
                let screen_y = ((vy + i) % SCREEN_HEIGHT as u8) as usize;
                let current_pixel = &mut self.virtual_screen[screen_y][screen_x];
                let new_pixel = ((pixels_to_draw[i as usize] >> (7 - j)) & 1) ^ *current_pixel;
                if *current_pixel == 1 && new_pixel == 0 {
                    bit_erased = true;
                }
                *current_pixel = new_pixel;
            }
        }
        if bit_erased {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
        self.draw_points();
    }
    fn draw_points(&mut self) {
        let mut points = vec![];
        for (i, row) in self.virtual_screen.iter().enumerate() {
            for (j, pixel) in row.iter().enumerate() {
                if *pixel == 1 {
                    points.push((j as i32, i as i32));
                }
            }
        }
        self.screen.draw_points(&points[..]);
    }
    fn run(&mut self) -> MainLoopAction {
        // println!("Press enter to read an instruction...");
        // let mut buffer = String::new();
        // stdin().read_line(&mut buffer).unwrap();
        // if buffer.trim() == "q" {
        //     process::exit(0);
        // }
        let instruction = self.next_instruction();
        match instruction {
            0x00E0 => self.screen.clear(),
            0x00EE => {
                self.sp -= 1;
                self.pc = self.stack[self.sp as usize];
            }
            instruction if instruction >= 0x1000 && instruction <= 0x1FFF => {
                self.pc = instruction & 0x0FFF;
            }
            instruction if instruction >= 0x2000 && instruction <= 0x2FFF => {
                let address = instruction & 0x0FFF;
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = address;
            }
            instruction if instruction >= 0x3000 && instruction <= 0x3FFF => {
                let register_index = (instruction >> 8 & 0x000F) as usize;
                let value = (instruction & 0x00FF) as u8;
                if self.registers[register_index] == value {
                    self.pc += 2;
                }
            }
            instruction if instruction >= 0x4000 && instruction <= 0x4FFF => {
                let register_index = (instruction >> 8 & 0x000F) as usize;
                let value = (instruction & 0x00FF) as u8;
                if self.registers[register_index] != value {
                    self.pc += 2;
                }
            }
            instruction if instruction >= 0x5000 && instruction <= 0x5FF0 => {
                let register_index_x = (instruction >> 8 & 0x000F) as usize;
                let register_index_y = (instruction >> 4 & 0x000F) as usize;
                if self.registers[register_index_x] == self.registers[register_index_y] {
                    self.pc += 2;
                }
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
            instruction if instruction >= 0x8000 && instruction <= 0x8FF0 => {
                let register_index_x = (instruction >> 8 & 0x000F) as usize;
                let register_index_y = (instruction >> 4 & 0x000F) as usize;
                self.registers[register_index_x] = self.registers[register_index_y];
            }
            instruction if instruction >= 0x8001 && instruction <= 0x8FF1 => {
                let register_index_x = (instruction >> 8 & 0x000F) as usize;
                let register_index_y = (instruction >> 4 & 0x000F) as usize;
                self.registers[register_index_x] |= self.registers[register_index_y];
            }
            instruction if instruction >= 0x8002 && instruction <= 0x8FF2 => {
                let register_index_x = (instruction >> 8 & 0x000F) as usize;
                let register_index_y = (instruction >> 4 & 0x000F) as usize;
                self.registers[register_index_x] &= self.registers[register_index_y];
            }
            instruction if instruction >= 0x8003 && instruction <= 0x8FF3 => {
                let register_index_x = (instruction >> 8 & 0x000F) as usize;
                let register_index_y = (instruction >> 4 & 0x000F) as usize;
                self.registers[register_index_x] ^= self.registers[register_index_y];
            }
            instruction if instruction >= 0x8004 && instruction <= 0x8FF4 => {
                let register_index_x = (instruction >> 8 & 0x000F) as usize;
                let register_index_y = (instruction >> 4 & 0x000F) as usize;
                let sum = self.registers[register_index_x] as u16
                    + self.registers[register_index_y] as u16;
                if sum > 255 {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
                self.registers[register_index_x] += self.registers[register_index_y];
            }
            instruction if instruction >= 0x8005 && instruction <= 0x8FF5 => {
                let register_index_x = (instruction >> 8 & 0x000F) as usize;
                let register_index_y = (instruction >> 4 & 0x000F) as usize;
                if self.registers[register_index_x] > self.registers[register_index_y] {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
                self.registers[register_index_x] -= self.registers[register_index_y];
            }
            instruction if instruction >= 0x8006 && instruction <= 0x8FF6 => {
                let register_index_x = (instruction >> 8 & 0x000F) as usize;
                self.registers[0xF] = self.registers[register_index_x] & 1;
                self.registers[register_index_x] >>= 1;
            }
            instruction if instruction >= 0x8007 && instruction <= 0x8FF7 => {
                let register_index_x = (instruction >> 8 & 0x000F) as usize;
                let register_index_y = (instruction >> 4 & 0x000F) as usize;
                if self.registers[register_index_y] > self.registers[register_index_x] {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
                self.registers[register_index_x] =
                    self.registers[register_index_y] - self.registers[register_index_x];
            }
            instruction if instruction >= 0x800E && instruction <= 0x8FFE => {
                let register_index_x = (instruction >> 8 & 0x000F) as usize;
                self.registers[0xF] = self.registers[register_index_x] >> 7 & 1;
                self.registers[register_index_x] <<= 1;
            }
            instruction if instruction >= 0x9000 && instruction <= 0x9FF0 => {
                let register_index_x = (instruction >> 8 & 0x000F) as usize;
                let register_index_y = (instruction >> 4 & 0x000F) as usize;
                if self.registers[register_index_x] != self.registers[register_index_y] {
                    self.pc += 2;
                }
            }
            instruction if instruction >= 0xA000 && instruction <= 0xAFFF => {
                self.i_reg = instruction & 0x0FFF
            }
            instruction if instruction >= 0xB000 && instruction <= 0xBFFF => {
                let address = instruction & 0x0FFF;
                self.pc = self.registers[0] as u16 + address;
            }
            instruction if instruction >= 0xC000 && instruction <= 0xCFFF => {
                let register_index = (instruction >> 8 & 0x000F) as usize;
                let value = instruction & 0x00FF;
                self.registers[register_index] = random::<u8>() & value as u8;
            }
            instruction if instruction >= 0xD000 && instruction <= 0xDFFF => {
                self.draw_generic_sprite(instruction)
            }
            instruction if instruction >= 0xE09E && instruction <= 0xEF9E => {
                let register_index = (instruction >> 8 & 0x000F) as usize;
                let register_value = self.registers[register_index];
                if self.screen.is_key_pressed(register_value) {
                    self.pc += 2;
                }
            }
            instruction if instruction >= 0xE0A1 && instruction <= 0xEFA1 => {
                let register_index = (instruction >> 8 & 0x000F) as usize;
                let register_value = self.registers[register_index];
                if !self.screen.is_key_pressed(register_value) {
                    self.pc += 2;
                }
            }
            instruction if instruction >= 0xF007 && instruction <= 0xFF07 => {
                let register_index = (instruction >> 8 & 0x000F) as usize;
                self.registers[register_index] = *self.delay_reg.lock().unwrap();
            }
            instruction if instruction >= 0xF00A && instruction <= 0xFF0A => {
                let register_index = (instruction >> 8 & 0x000F) as usize;
                match self.screen.wait_for_keypress() {
                    Some(key) => {
                        self.registers[register_index] = key;
                    }
                    None => return MainLoopAction::Interrupt,
                }
            }
            instruction if instruction >= 0xF015 && instruction <= 0xFF15 => {
                let register_index = (instruction >> 8 & 0x000F) as usize;
                self.delay_reg = Arc::new(Mutex::new(self.registers[register_index]));
            }
            instruction if instruction >= 0xF018 && instruction <= 0xFF18 => {
                let register_index = (instruction >> 8 & 0x000F) as usize;
                self.sound_reg = Arc::new(Mutex::new(self.registers[register_index]));
            }
            instruction if instruction >= 0xF01E && instruction <= 0xFF1E => {
                let register_index = (instruction >> 8 & 0x000F) as usize;
                self.i_reg += self.registers[register_index] as u16;
            }
            instruction if instruction >= 0xF029 && instruction <= 0xFF29 => {
                let register_index = (instruction >> 8 & 0x000F) as usize;
                let value = self.registers[register_index];
                self.i_reg = value as u16 * 5;
            }
            instruction if instruction >= 0xF033 && instruction <= 0xFF33 => {
                let register_index = (instruction >> 8 & 0x000F) as usize;
                let value = self.registers[register_index];
                let hundreds = (value / 100) * 100;
                let tens = ((value - hundreds) / 10) * 10;
                let ones = value - hundreds - tens;
                let i_reg = self.i_reg as usize;
                self.memory[i_reg] = hundreds;
                self.memory[i_reg + 1] = tens;
                self.memory[i_reg + 2] = ones;
            }
            instruction if instruction >= 0xF055 && instruction <= 0xFF55 => {
                let register_index = (instruction >> 8 & 0x000F) as usize;
                for reg in 0..=register_index {
                    self.memory[self.i_reg as usize + reg] = self.registers[reg];
                }
            }
            instruction if instruction >= 0xF065 && instruction <= 0xFF65 => {
                let register_index = (instruction >> 8 & 0x000F) as usize;
                for reg in 0..=register_index {
                    self.registers[reg] = self.memory[self.i_reg as usize + reg];
                }
            }
            instruction => panic!("Unknown instruction: {instruction}"),
        }
        MainLoopAction::Continue
        // println!(" == Vm State == ");
        // // println!("memory: {:?}", &self.memory[0x200..(0x200 + 132)]);
        // println!(
        //     "bytes: {:?}",
        //     &self.memory[self.i_reg as usize..(self.i_reg + 16) as usize]
        //         .iter()
        //         .map(|x| format!("{:08b}", x))
        //         .collect::<Vec<_>>()
        // );
        // println!("registers: {:?}", self.registers);
        // println!("I register: {:?}", self.i_reg);
        // println!("pc: {:?}", self.pc);
        // println!("virtual screen: {:?}", self.virtual_screen);
    }
}
