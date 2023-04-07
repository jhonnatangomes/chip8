use std::{env, fs, process};

use vm::Vm;

mod screen;
mod vm;

fn main() {
    let file = env::args().nth(1).unwrap();
    let rom = fs::read(file).unwrap();
    if rom.len() > 3584 {
        eprintln!("Rom is too large");
        process::exit(1);
    }
    println!("{}", rom.len());
    let vm = Vm::new(rom);
    vm.start();
}
