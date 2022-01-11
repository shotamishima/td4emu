use td4emu::emulator::CpuEmulator;
use td4emu::port::Port;
use td4emu::register::Register;
use td4emu::rom::Rom;

fn main() {
    let rom = Rom::new(vec![0b00110001, 0b00000001, 0b01000000, 0b10010000]);
    let register = Register::new();
    let port = Port::new(0b0000, 0b0000);
    let emulator = CpuEmulator::with(register, port, rom);
    match emulator.exec() {
        Ok(_) => emulator.out(),
        Err(err) => eprintln!("{:?}", err),
    }
}
