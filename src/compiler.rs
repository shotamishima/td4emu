use crate::error::EmulatorErr;
use crate::token::{Register, Token};

pub struct Compiler;

impl Compiler {
    pub fn new() -> Self {
        Compiler
    }

    pub fn compile(&self, tokens: Vec<Token>) -> Result<Vec<u8>, EmulatorErr> {
        let mut result = Vec::new();

        for token in tokens {
            let program = match token {
                Token::Mov(Register::A, im) => self.gen_bin_code(0b0011, im)?,
                Token::Mov(Register::B, im) => self.gen_bin_code(0b0111, im)?,
                Token::MovAB => self.gen_bin_code_with_zero_padding(0b0001),
                Token::MovBA => self.gen_bin_code_with_zero_padding(0b0100),
                Token::Add(Register::A, im) => self.gen_bin_code(0b0000, im)?,
                Token::Add(Register::B, im) => self.gen_bin_code(0b0101, im)?,
                Token::Jmp(im) => self.gen_bin_code(0b1111, im)?,
                Token::Jnc(im) => self.gen_bin_code(0b1110, im)?,
                Token::In(Register::A, im) => self.gen_bin_code(0b0010, im)?,
                Token::In(Register::B, im) => self.gen_bin_code(0b0110, im)?,
                Token::OutB => self.gen_bin_code_with_zero_padding(0b1001),
                Token::OutIm(im) => self.gen_bin_code(0b1011, im)?,
            };
            result.push(program);
        }

        Ok(result)
    }

    fn gen_bin_code(&self, op: u8, im: String) -> Result<u8, EmulatorErr> {
        let shift_op = op << 4;
        let shift_data = im
            .parse::<u8>()
            .map_err(|_| EmulatorErr::new("Failed to parse im: {}"))?
            & 0x0f;
        Ok(shift_op | shift_data)
    }

    fn gen_bin_code_with_zero_padding(&self, op: u8) -> u8 {
        let shift_op = op << 4;
        let zero_padding = 0b0000 & 0x0f;
        shift_op | zero_padding
    }
}

#[cfg(test)]
mod compiler_tests {
    use crate::compiler::Compiler;
    use crate::token::Register;
    use crate::token::Token::{Add, In, Jmp, Jnc, Mov, MovAB, MovBA, OutB, OutIm};

    #[test]
    fn test_compiler_mov_a() {
        let compiler = Compiler::new();
        let program = compiler.compile(vec![Mov(Register::A, "1".to_string())]);
        assert_eq!(program.unwrap(), vec![0b00110001]);
    }

    #[test]
    fn test_compiler_mov_b() {
        let compiler = Compiler::new();
        let program = compiler.compile(vec![Mov(Register::B, "1".to_string())]);
        assert_eq!(program.unwrap(), vec![0b01110001]);
    }

    #[test]
    fn test_compiler_mov_ab() {
        let compiler = Compiler::new();
        let program = compiler.compile(vec![MovAB]);
        assert_eq!(program.unwrap(), vec![0b00010000]);
    }

    #[test]
    fn test_conpiler_mov_ba() {
        let compiler = Compiler::new();
        let program = compiler.compile(vec![MovBA]);
        assert_eq!(program.unwrap(), vec![0b01000000]);
    }

    #[test]
    fn test_compiler_add_a() {
        let compiler = Compiler::new();
        let program = compiler.compile(vec![Add(Register::A, "1".to_string())]);
        assert_eq!(program.unwrap(), vec![0b00000001]);
    }

    #[test]
    fn test_compiler_add_b() {
        let compiler = Compiler::new();
        let program = compiler.compile(vec![Add(Register::B, "1".to_string())]);
        assert_eq!(program.unwrap(), vec![0b01010001]);
    }

    #[test]
    fn test_compiler_jmp() {
        let compiler = Compiler::new();
        let program = compiler.compile(vec![Jmp("1".to_string())]);
        assert_eq!(program.unwrap(), vec![0b11110001]);
    }

    #[test]
    fn test_compiler_jnc() {
        let compiler = Compiler::new();
        let program = compiler.compile(vec![Jnc("1".to_string())]);
        assert_eq!(program.unwrap(), vec![0b11100001]);
    }
    #[test]
    fn test_compiler_in_a() {
        let compiler = Compiler::new();
        let program = compiler.compile(vec![In(Register::A, "1".to_string())]);
        assert_eq!(program.unwrap(), vec![0b00100001]);
    }
    #[test]
    fn test_compiler_in_b() {
        let compiler = Compiler::new();
        let program = compiler.compile(vec![In(Register::B, "1".to_string())]);
        assert_eq!(program.unwrap(), vec![0b01100001]);
    }

    #[test]
    fn test_compiler_out_im() {
        let compiler = Compiler::new();
        let program = compiler.compile(vec![OutIm("1".to_string())]);
        assert_eq!(program.unwrap(), vec![0b10110001]);
    }

    #[test]
    fn test_compiler_out_b() {
        let compiler = Compiler::new();
        let program = compiler.compile(vec![OutB]);
        assert_eq!(program.unwrap(), vec![0b10010000]);
    }
}
