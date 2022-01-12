use std::fs::File;
use std::io::{BufRead, BufReader};
use td4emu::emulator::CpuEmulator;
use td4emu::port::Port;
use td4emu::register::Register;
use td4emu::rom::Rom;
use td4emu::compiler::Compiler;
use td4emu::parser::Parser;

fn main() {
    let f = BufReader::new(File::open("example/simple_calc.sasm").expect("file not found"));
    let ops = f.lines().map(|line| line.unwrap()).collect::<Vec<String>>();

    let mut source = Vec::new();
    for op in ops {
        let v: Vec<&str> = op.split(' ').collect();
        for value in v {
            let cloned = value.to_string();
            source.push(cloned);
        }
    }

    let mut parser = Parser::new(source);
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
        Ok(_) => emulator.out(),
        Err(err) => panic!("{:?}", err),
    }
}
