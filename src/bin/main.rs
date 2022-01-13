use std::fs::File;
use std::io::{BufRead, BufReader};
use td4emu::emulator::CpuEmulator;
use td4emu::port::Port;
use td4emu::register::Register;
use td4emu::rom::Rom;
use td4emu::compiler::Compiler;
use td4emu::parser::Parser;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        panic!("Invalid args. Usage: [command] [file_path]");
    }

    let f = BufReader::new(File::open(args.get(1).unwrap()).expect("file not found"));
    let operations= f.lines().map(|line| line.unwrap()).collect::<Vec<String>>();

    let mut parser = Parser::new(operations);
    let tokens = match parser.parse() {
        Ok(tokens) => tokens,
        Err(err) => panic!("{:?}", err),
    };

    let compiler = Compiler::new();
    let program = match compiler.compile(tokens) {
        Ok(program) => program,
        Err(err) => panic!("{:?}", err),
    };

    let rom = Rom::new(program);
    let register = Register::new();
    let port = Port::new(0b0000, 0b0000);
    let emulator = CpuEmulator::with(register, port, rom);
    match emulator.exec() {
        Ok(_) => (),
        Err(err) => panic!("{:?}", err),
    }
}
