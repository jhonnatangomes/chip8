use std::{
    io::stdin,
    process,
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
            0x1000..=0x1FFF => {
                self.pc = instruction & 0x0FFF;
            }
            0x2000..=0x2FFF => {
                let address = instruction & 0x0FFF;
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = address;
            }
            0x3000..=0x3FFF => {
                let register_index = (instruction >> 8 & 0x000F) as usize;
                let value = (instruction & 0x00FF) as u8;
                if self.registers[register_index] == value {
                    self.pc += 2;
                }
            }
            0x4000..=0x4FFF => {
                let register_index = (instruction >> 8 & 0x000F) as usize;
                let value = (instruction & 0x00FF) as u8;
                if self.registers[register_index] != value {
                    self.pc += 2;
                }
            }
            instruction
                if upper_first_byte(instruction) == 5 && lower_second_byte(instruction) == 0 =>
            {
                let register_index_x = (instruction >> 8 & 0x000F) as usize;
                let register_index_y = (instruction >> 4 & 0x000F) as usize;
                if self.registers[register_index_x] == self.registers[register_index_y] {
                    self.pc += 2;
                }
            }
            0x6000..=0x6FFF => {
                let register_index = (instruction >> 8 & 0x000F) as usize;
                let value = instruction & 0x00FF;
                self.registers[register_index] = value as u8;
            }
            0x7000..=0x7FFF => {
                let register_index = instruction >> 8 & 0x000F;
                let value = instruction & 0x00FF;
                self.registers[register_index as usize] += value as u8;
            }
            instruction if upper_first_byte(instruction) == 8 => {
                match lower_second_byte(instruction) {
                    0 => {
                        let register_index_x = (instruction >> 8 & 0x000F) as usize;
                        let register_index_y = (instruction >> 4 & 0x000F) as usize;
                        self.registers[register_index_x] = self.registers[register_index_y];
                    }
                    1 => {
                        let register_index_x = (instruction >> 8 & 0x000F) as usize;
                        let register_index_y = (instruction >> 4 & 0x000F) as usize;
                        self.registers[register_index_x] |= self.registers[register_index_y];
                    }
                    2 => {
                        let register_index_x = (instruction >> 8 & 0x000F) as usize;
                        let register_index_y = (instruction >> 4 & 0x000F) as usize;
                        self.registers[register_index_x] &= self.registers[register_index_y];
                    }
                    3 => {
                        let register_index_x = (instruction >> 8 & 0x000F) as usize;
                        let register_index_y = (instruction >> 4 & 0x000F) as usize;
                        self.registers[register_index_x] ^= self.registers[register_index_y];
                    }
                    4 => {
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
                    5 => {
                        let register_index_x = (instruction >> 8 & 0x000F) as usize;
                        let register_index_y = (instruction >> 4 & 0x000F) as usize;
                        if self.registers[register_index_x] > self.registers[register_index_y] {
                            self.registers[0xF] = 1;
                        } else {
                            self.registers[0xF] = 0;
                        }
                        self.registers[register_index_x] -= self.registers[register_index_y];
                    }
                    6 => {
                        let register_index_x = (instruction >> 8 & 0x000F) as usize;
                        self.registers[0xF] = self.registers[register_index_x] & 1;
                        self.registers[register_index_x] >>= 1;
                    }
                    7 => {
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
                    0xE => {
                        let register_index_x = (instruction >> 8 & 0x000F) as usize;
                        self.registers[0xF] = self.registers[register_index_x] >> 7 & 1;
                        self.registers[register_index_x] <<= 1;
                    }
                    _ => panic!("Unknown instruction: {instruction}"),
                }
            }
            instruction
                if upper_first_byte(instruction) == 9 && lower_second_byte(instruction) == 0 =>
            {
                let register_index_x = (instruction >> 8 & 0x000F) as usize;
                let register_index_y = (instruction >> 4 & 0x000F) as usize;
                if self.registers[register_index_x] != self.registers[register_index_y] {
                    self.pc += 2;
                }
            }
            0xA000..=0xAFFF => {
                self.i_reg = instruction & 0x0FFF;
            }
            0xB000..=0xBFFF => {
                let address = instruction & 0x0FFF;
                self.pc = self.registers[0] as u16 + address;
            }
            0xC000..=0xCFFF => {
                let register_index = (instruction >> 8 & 0x000F) as usize;
                let value = instruction & 0x00FF;
                self.registers[register_index] = random::<u8>() & value as u8;
            }
            0xD000..=0xDFFF => self.draw_generic_sprite(instruction),
            instruction
                if upper_first_byte(instruction) == 0xE
                    && upper_second_byte(instruction) == 9
                    && lower_second_byte(instruction) == 0xE =>
            {
                let register_index = (instruction >> 8 & 0x000F) as usize;
                let register_value = self.registers[register_index];
                if self.screen.is_key_pressed(register_value) {
                    self.pc += 2;
                }
            }
            instruction
                if upper_first_byte(instruction) == 0xE
                    && upper_second_byte(instruction) == 0xA
                    && lower_second_byte(instruction) == 1 =>
            {
                let register_index = (instruction >> 8 & 0x000F) as usize;
                let register_value = self.registers[register_index];
                if !self.screen.is_key_pressed(register_value) {
                    self.pc += 2;
                }
            }
            instruction if upper_first_byte(instruction) == 0xF => {
                if upper_second_byte(instruction) == 0 && lower_second_byte(instruction) == 7 {
                    let register_index = (instruction >> 8 & 0x000F) as usize;
                    self.registers[register_index] = *self.delay_reg.lock().unwrap();
                }
                if upper_second_byte(instruction) == 0 && lower_second_byte(instruction) == 0xA {
                    let register_index = (instruction >> 8 & 0x000F) as usize;
                    match self.screen.wait_for_keypress() {
                        Some(key) => {
                            self.registers[register_index] = key;
                        }
                        None => return MainLoopAction::Interrupt,
                    }
                }
                if upper_second_byte(instruction) == 1 && lower_second_byte(instruction) == 5 {
                    let register_index = (instruction >> 8 & 0x000F) as usize;
                    self.delay_reg = Arc::new(Mutex::new(self.registers[register_index]));
                }
                if upper_second_byte(instruction) == 1 && lower_second_byte(instruction) == 8 {
                    let register_index = (instruction >> 8 & 0x000F) as usize;
                    self.sound_reg = Arc::new(Mutex::new(self.registers[register_index]));
                }
                if upper_second_byte(instruction) == 1 && lower_second_byte(instruction) == 0xE {
                    let register_index = (instruction >> 8 & 0x000F) as usize;
                    self.i_reg += self.registers[register_index] as u16;
                }
                if upper_second_byte(instruction) == 2 && lower_second_byte(instruction) == 9 {
                    let register_index = (instruction >> 8 & 0x000F) as usize;
                    let value = self.registers[register_index];
                    self.i_reg = value as u16 * 5;
                }
                if upper_second_byte(instruction) == 3 && lower_second_byte(instruction) == 3 {
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
                if upper_second_byte(instruction) == 5 && lower_second_byte(instruction) == 5 {
                    let register_index = (instruction >> 8 & 0x000F) as usize;
                    for reg in 0..=register_index {
                        self.memory[self.i_reg as usize + reg] = self.registers[reg];
                    }
                }
                if upper_second_byte(instruction) == 6 && lower_second_byte(instruction) == 5 {
                    let register_index = (instruction >> 8 & 0x000F) as usize;
                    for reg in 0..=register_index {
                        self.registers[reg] = self.memory[self.i_reg as usize + reg];
                    }
                }
            }
            instruction => panic!("Unknown instruction: {instruction}"),
        }
        // println!(" == Vm State == ");
        // println!("instruction: {}", format!("{:#06x}", instruction));
        // println!(
        //     "bytes: {:?}",
        //     &self.memory[self.i_reg as usize..(self.i_reg + 16) as usize]
        //         .iter()
        //         .map(|x| format!("{:08b}", x))
        //         .collect::<Vec<_>>()
        // );
        // println!(
        //     "registers: {:?}",
        //     self.registers
        //         .iter()
        //         .map(|x| format!("{:04x}", x))
        //         .collect::<Vec<_>>()
        // );
        // println!("I register: {:?}", format!("{:04x}", self.i_reg));
        // println!("pc: {:?}", format!("{:04x}", self.pc));
        // println!("virtual screen: {:?}", self.virtual_screen);
        MainLoopAction::Continue
    }
}
fn upper_first_byte(instruction: u16) -> u8 {
    (instruction >> 12 & 0x000F) as u8
}
fn upper_second_byte(instruction: u16) -> u8 {
    (instruction >> 4 & 0x000F) as u8
}
fn lower_second_byte(instruction: u16) -> u8 {
    (instruction & 0x000F) as u8
}
