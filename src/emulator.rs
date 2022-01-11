use crate::error::EmulatorErr;
use crate::op::Opcode;
use crate::port::Port;
use crate::register::Register;
use crate::rom::Rom;
use num_traits::FromPrimitive;
use std::cell::RefCell;

pub struct CpuEmulator {
    register: RefCell<Register>,
    rom: RefCell<Rom>,
    port: RefCell<Port>,
}

impl CpuEmulator {
    // register, rom, portの指定なしにオブジェクトを生成することはないのでnew関数を削除

    pub fn with(register: Register, port: Port, rom: Rom) -> Self {
        assert!(
            rom.size() <= 16,
            "Maximum memory size is 16. This program can't work."
            );
        Self {
            register: RefCell::new(register),
            port: RefCell::new(port),
            rom: RefCell::new(rom),
        }
    }

    // fetch, decode関数はexecからしか呼ばないのでpub -> privateに変更
    fn fetch(&self) -> u8 {
        let pc = self.register.borrow().pc();
        if self.rom.borrow().size() <= pc {
            return 0;
        }

        let code = self.rom.borrow().read(pc);

        code
    }

    fn decode(&self, data: u8) -> Result<(Opcode, u8), EmulatorErr> {
        let op = data >> 4;
        let im = data & 0x0f;

        if let Some(opcode) = FromPrimitive::from_u8(op) {
            match opcode {
                Opcode::AddA
                | Opcode::AddB
                | Opcode::MovA
                | Opcode::MovB
                | Opcode::MovA2B
                | Opcode::MovB2A
                | Opcode::Jmp
                | Opcode::Jnc
                | Opcode::OutIm => Ok((opcode, im)),
                Opcode::InA | Opcode::InB | Opcode::OutB => Ok((opcode, 0)), // imidiate data is always 0
            }
        } else {
            // never come
            Err(EmulatorErr::new("No match for opcode"))
        }
    }

    pub fn exec(&self) -> Result<(), EmulatorErr> {
        loop {
            let data = self.fetch();
            let (opcode, im) = self.decode(data)?;

            match opcode {
                Opcode::MovA => self.mov_a(im),
                Opcode::MovB => self.mov_b(im),
                Opcode::AddA => self.add_a(im),
                Opcode::AddB => self.add_b(im),
                Opcode::MovA2B => self.mov_a2b(),
                Opcode::MovB2A => self.mov_b2a(),
                Opcode::Jmp => self.jmp(im),
                Opcode::Jnc => self.jnc(im),
                Opcode::InA => self.in_a(),
                Opcode::InB => self.in_b(),
                Opcode::OutB => self.out_b(),
                Opcode::OutIm => self.out_im(im),
            };

            // To prevent infinite loop
            if opcode != Opcode::Jmp && opcode != Opcode::Jnc {
                self.register.borrow_mut().incr_pc();
            }
            if self.does_halt() {
                return Ok(());
            }
        }
    }

    // fetchで判定するより前に判定
    fn does_halt(&self) -> bool {
        self.register.borrow().pc() >= self.rom.borrow().size() - 1
    }

    fn mov_a(&self, im: u8) {
        // registerの値を変更するので可変参照
        // opcodeに対する処理の内容は本p.230を参照
        self.register.borrow_mut().set_register_a(im);
        self.register.borrow_mut().set_carry_flag(0);
    }

    fn mov_b(&self, im: u8) {
        self.register.borrow_mut().set_register_b(im);
        self.register.borrow_mut().set_carry_flag(0);
    }

    fn mov_a2b(&self) {
        let register_b = self.register.borrow().register_b();
        self.register.borrow_mut().set_register_a(register_b);
        self.register.borrow_mut().set_carry_flag(0);
    }

    fn mov_b2a(&self) {
        let register_a = self.register.borrow().register_a();
        self.register.borrow_mut().set_register_b(register_a);
        self.register.borrow_mut().set_carry_flag(0);
    }

    fn add_a(&self, im: u8) {
        let existence = self.register.borrow().register_a();
        let new_value = existence + im;

        if new_value > 0x0f {
            self.register.borrow_mut().set_carry_flag(1);
        }

        self.register.borrow_mut().set_register_a(new_value & 0x0f);
    }

    fn add_b(&self, im: u8) {
        let existence = self.register.borrow().register_b();
        let new_value = existence + im;

        if new_value > 0x0f {
            self.register.borrow_mut().set_carry_flag(1);
        }

        self.register.borrow_mut().set_register_b(new_value & 0x0f);
    }

    fn in_a(&self) {
        let input_port = self.port.borrow().input();
        self.register.borrow_mut().set_register_a(input_port);
        self.register.borrow_mut().set_carry_flag(0);
    }

    fn in_b(&self) {
        let input_port = self.port.borrow().input();
        self.register.borrow_mut().set_register_b(input_port);
        self.register.borrow_mut().set_carry_flag(0);
    }

    fn out_im(&self, im: u8) {
        self.port.borrow_mut().set_output(im);
        self.register.borrow_mut().set_carry_flag(0);
    }

    fn out_b(&self) {
        let register_b = self.register.borrow().register_b();
        self.port.borrow_mut().set_output(register_b);
        self.register.borrow_mut().set_carry_flag(0);
    }

    fn jmp(&self, im: u8) {
        self.register.borrow_mut().set_pc(im);
        self.register.borrow_mut().set_carry_flag(0);
    }

    fn jnc(&self, im: u8) {
        if self.register.borrow().carry_flag() == 0 {
            self.register.borrow_mut().set_pc(im);
        }
        self.register.borrow_mut().set_carry_flag(0);
    }
}

#[cfg(test)]
mod cpu_tests {
    use crate::emulator::CpuEmulator;
    use crate::port::Port;
    use crate::register::Register;
    use crate::rom::Rom;

    #[test]
    fn test_mov_a() {
        let rom = Rom::new(vec![0b00110001]);
        let register = Register::new();
        let port = Port::new(0b0000, 0b0000);
        let emu = CpuEmulator::with(register, port, rom);
        let proceeded = emu.exec();

        assert!(proceeded.is_ok());
        assert_eq!(emu.register.borrow().register_a(), 1);
        assert_eq!(emu.register.borrow().register_b(), 0);
        assert_eq!(emu.register.borrow().pc(), 1);
        assert_eq!(emu.register.borrow().carry_flag(), 0);
    }

    #[test]
    fn test_mov_b() {
        let rom = Rom::new(vec![0b01110001]);
        let register = Register::new();
        let port = Port::new(0b0000, 0b0000);
        let emu = CpuEmulator::with(register, port, rom);
        let proceeded = emu.exec();

        assert!(proceeded.is_ok());
        assert_eq!(emu.register.borrow().register_a(), 0);
        assert_eq!(emu.register.borrow().register_b(), 1);
        assert_eq!(emu.register.borrow().pc(), 1);
        assert_eq!(emu.register.borrow().carry_flag(), 0);
    }

    #[test]
    fn test_mov_a2b() {
        let rom = Rom::new(vec![0b00010000]);
        let mut register = Register::new();
        register.set_register_b(2);
        let port = Port::new(0b0000, 0b0000);
        let emu = CpuEmulator::with(register, port, rom);

        assert_eq!(emu.register.borrow().register_a(), 0);

        let proceeded = emu.exec();

        assert!(proceeded.is_ok());
        assert_eq!(emu.register.borrow().register_a(), 2);
        assert_eq!(emu.register.borrow().register_b(), 2);
        assert_eq!(emu.register.borrow().pc(), 1);
        assert_eq!(emu.register.borrow().carry_flag(), 0);
    }

    #[test]
    fn test_mov_b2a() {
        let rom = Rom::new(vec![0b01000000]);
        let mut register = Register::new();
        register.set_register_a(2);
        let port = Port::new(0b0000, 0b0000);
        let emu = CpuEmulator::with(register, port, rom);

        assert_eq!(emu.register.borrow().register_b(), 0);

        let proceeded = emu.exec();

        assert!(proceeded.is_ok());
        assert_eq!(emu.register.borrow().register_a(), 2);
        assert_eq!(emu.register.borrow().register_b(), 2);
        assert_eq!(emu.register.borrow().pc(), 1);
        assert_eq!(emu.register.borrow().carry_flag(), 0);
    }

    #[test]
    fn test_add_a_without_carrying() {
        let rom = Rom::new(vec![0b00000001]);
        let mut register = Register::new();
        register.set_register_a(1);
        let port = Port::new(0b0000, 0b0000);
        let emu = CpuEmulator::with(register, port, rom);
        let proceeded = emu.exec();

        assert!(proceeded.is_ok());
        assert_eq!(emu.register.borrow().register_a(), 2);
        assert_eq!(emu.register.borrow().register_b(), 0);
        assert_eq!(emu.register.borrow().pc(), 1);
        assert_eq!(emu.register.borrow().carry_flag(), 0);
    }

    #[test]
    fn test_add_b_without_carrying() {
        let rom = Rom::new(vec![0b01010001]);
        let mut register = Register::new();
        register.set_register_b(1);
        let port = Port::new(0b0000, 0b0000);
        let emu = CpuEmulator::with(register, port, rom);
        let proceeded = emu.exec();

        assert!(proceeded.is_ok());
        assert_eq!(emu.register.borrow().register_a(), 0);
        assert_eq!(emu.register.borrow().register_b(), 2);
        assert_eq!(emu.register.borrow().pc(), 1);
        assert_eq!(emu.register.borrow().carry_flag(), 0);
    }

    #[test]
    fn test_jmp() {
        let rom = Rom::new(vec![0b11110000]);
        let register = Register::new();
        let port = Port::new(0b0000, 0b0000);
        let emu = CpuEmulator::with(register, port, rom);
        let proceeded= emu.exec();

        assert!(proceeded.is_ok());
        assert_eq!(emu.register.borrow().pc(), 0);
    }

    #[test]
    fn test_port_in_a() {
        let rom = Rom::new(vec![0b00100000]);
        let register = Register::new();
        let port = Port::new(0b0001, 0b0000);
        let emu = CpuEmulator::with(register, port, rom);
        let proceeded = emu.exec();

        assert!(proceeded.is_ok());
        assert_eq!(emu.register.borrow().register_a(), 1);
        assert_eq!(emu.register.borrow().carry_flag(), 0);
    }

    #[test]
    fn test_port_in_b() {
        let rom = Rom::new(vec![0b01100000]);
        let register = Register::new();
        let port = Port::new(0b0011, 0b0000);
        let emu = CpuEmulator::with(register, port, rom);
        let proceeded = emu.exec();

        assert!(proceeded.is_ok());
        assert_eq!(emu.register.borrow().register_b(), 3);
        assert_eq!(emu.register.borrow().carry_flag(), 0);
    }

    #[test]
    fn test_port_out_b() {
        let rom = Rom::new(vec![0b10010000]);
        let mut register = Register::new();
        register.set_register_b(0b0011);
        let port = Port::new(0b0000, 0b0000);
        let emu = CpuEmulator::with(register, port, rom);
        let proceeded = emu.exec();

        assert!(proceeded.is_ok());
        assert_eq!(emu.port.borrow().output(), 0b0011);
        assert_eq!(emu.register.borrow().carry_flag(), 0);
    }

    #[test]
    fn test_port_out_im() {
        let rom = Rom::new(vec![0b10110011]);
        let register = Register::new();
        let port = Port::new(0b0000, 0b0000);
        let emu = CpuEmulator::with(register, port, rom);
        let proceeded = emu.exec();

        assert!(proceeded.is_ok());
        assert_eq!(emu.port.borrow().output(), 0b0011);
        assert_eq!(emu.register.borrow().carry_flag(), 0);
    }
}
