use crate::writer::Writer;

#[derive(Default)]
pub struct X86Asm {
    writer: Writer,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum Reg64 {
    Rax = 0,
    Rcx = 1,
    Rdx = 2,
    Rbx = 3,
    Rsp = 4,
    Rbp = 5,
    Rsi = 6,
    Rdi = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    R13 = 13,
    R14 = 14,
    R15 = 15,
}

#[derive(Debug, Clone, Copy)]
pub enum Operand {
    Reg(Reg64),
    Imm64(i64),
    Imm32(i32),
    Imm16(i16),
    Imm8(i8),
    MemDisp(Reg64, i32),
    MemAbs(Reg64),
}

#[allow(dead_code)]
enum ModRM {
    Mem = 0b00,
    Reg = 0b11,
    MemDisp8 = 0b01,
    MemDisp32 = 0b10,
}

impl X86Asm {
    pub fn new() -> Self {
        Self {
            writer: Writer::new(),
        }
    }

    fn emit_rex_oi(&mut self, arg: Operand, w: u8) {
        self.emit_rex_slash(arg, w);
    }

    #[allow(clippy::identity_op)]
    fn emit_rex_slash(&mut self, arg: Operand, w: u8) {
        match arg {
            Operand::Reg(reg) | Operand::MemDisp(reg, _) => {
                if w == 0 && (reg as u8) < 8 {
                    return;
                }
                let is_b = ((reg as u8) >= 8) as u8;
                let rex = 0b0100_0000 | (w << 3) | (0 << 2) | (0 << 1) | (is_b << 0);
                self.writer.emit8(rex);
            }
            _ => unimplemented!(),
        }
    }

    #[allow(clippy::identity_op)]
    fn emit_rex_mr(&mut self, dst: Operand, src: Operand, w: u8) {
        match (dst, src) {
            (Operand::Reg(dst_reg), Operand::Reg(src_reg))
                if (dst_reg as u8) < 8 && (src_reg as u8) < 8 && w == 0 => {}
            (Operand::Reg(dst_reg), Operand::Reg(src_reg))
            | (Operand::MemDisp(dst_reg, _), Operand::Reg(src_reg)) => {
                let is_b = (dst_reg as u8 >= 8) as u8;
                let is_r = (src_reg as u8 >= 8) as u8;
                let rex = 0b0100_0000 | (w << 3) | (is_r << 2) | (0 << 1) | (is_b << 0);
                self.writer.emit8(rex);
            }
            _ => unimplemented!(),
        };
    }

    #[allow(clippy::identity_op)]
    fn emit_rex_rm(&mut self, dst: Operand, src: Operand, w: u8) {
        match (dst, src) {
            (Operand::Reg(dst_reg), Operand::Reg(src_reg))
                if (dst_reg as u8) < 8 && (src_reg as u8) < 8 => {}
            (Operand::Reg(dst_reg), Operand::Reg(src_reg))
            | (Operand::Reg(dst_reg), Operand::MemDisp(src_reg, _)) => {
                let is_b = (src_reg as u8 >= 8) as u8;
                let is_r = (dst_reg as u8 >= 8) as u8;
                let rex = 0b0100_0000 | (w << 3) | (is_r << 2) | (0 << 1) | (is_b << 0);
                self.writer.emit8(rex);
            }
            _ => unimplemented!(),
        };
    }

    fn emit_modrm(&mut self, modrm_low: u8, rm: Operand) {
        match rm {
            Operand::Reg(_) => {
                self.writer.emit8(((ModRM::Reg as u8) << 6) | modrm_low);
            }
            Operand::MemDisp(_, offset) => {
                self.writer
                    .emit8(((ModRM::MemDisp32 as u8) << 6) | modrm_low);
                self.writer.emit32(offset as u32);
            }
            _ => unimplemented!(),
        }
    }

    fn emit_modrm_rm(&mut self, dst: Reg64, src: Operand) {
        let src_reg = match src {
            Operand::Reg(src_reg) => src_reg,
            Operand::MemDisp(base_reg, _) => base_reg,
            _ => unimplemented!(),
        };

        let modrm_low = (Self::encode_reg(dst) << 3) | Self::encode_reg(src_reg);
        self.emit_modrm(modrm_low, src);
    }

    fn emit_modrm_mr(&mut self, dst: Operand, src: Reg64) {
        let dst_reg = match dst {
            Operand::Reg(dst_reg) => dst_reg,
            Operand::MemDisp(base_reg, _) => base_reg,
            _ => unimplemented!(),
        };

        let modrm_low = (Self::encode_reg(src) << 3) | Self::encode_reg(dst_reg);
        self.emit_modrm(modrm_low, dst);
    }

    fn emit_modrm_slash(&mut self, slash: u8, rm: Operand) {
        let reg = match rm {
            Operand::Reg(dst_reg) => dst_reg,
            Operand::MemDisp(base_reg, _) => base_reg,
            _ => unimplemented!(),
        };

        let modrm_low = (slash << 3) | Self::encode_reg(reg);
        self.emit_modrm(modrm_low, rm);
    }

    fn encode_reg(reg: Reg64) -> u8 {
        reg as u8 & 0b111
    }

    pub fn dump_generated_code(&self, base_addr: u64) {
        use zydis::*;

        let fmt = Formatter::intel();
        let dec = Decoder::new64();

        for inst_info in dec.decode_all::<VisibleOperands>(self.code(), 0) {
            let (ip, raw_bytes, inst) = inst_info.unwrap();

            // We use Some(ip) here since we want absolute addressing based on the given
            // instruction pointer. If we wanted relative addressing, we'd use `None` instead.
            println!(
                "0x{:016X} {:<24} {}",
                base_addr + ip,
                hex::encode(raw_bytes),
                fmt.format(Some(ip), &inst).unwrap()
            );
        }
    }

    pub fn code(&self) -> &[u8] {
        self.writer.bytes()
    }

    pub fn neg(&mut self, dst: Operand) {
        match dst {
            Operand::Reg(_) => {
                self.emit_rex_oi(dst, 1);
                self.writer.emit8(0xf7);
                self.emit_modrm_slash(3, dst);
            }
            _ => unimplemented!(),
        }
    }

    pub fn add(&mut self, dst: Operand, src: Operand) {
        match (dst, src) {
            (Operand::Reg(_), Operand::Reg(src_reg)) => {
                self.emit_rex_mr(dst, src, 1);
                self.writer.emit8(0x01);
                self.emit_modrm_mr(dst, src_reg);
            }
            (Operand::Reg(_), Operand::Imm64(imm)) => {
                self.emit_rex_slash(dst, 1);
                self.writer.emit8(0x81);
                self.emit_modrm_slash(0, dst);
                self.writer.emit32(imm as u32);
            }
            _ => unimplemented!(),
        }
    }

    pub fn sub(&mut self, dst: Operand, src: Operand) {
        match (dst, src) {
            (Operand::Reg(_), Operand::Reg(src_reg)) => {
                self.emit_rex_mr(dst, src, 1);
                self.writer.emit8(0x29);
                self.emit_modrm_mr(dst, src_reg);
            }
            (Operand::Reg(_), Operand::Imm64(imm)) => {
                self.emit_rex_slash(dst, 1);
                self.writer.emit8(0x81);
                self.emit_modrm_slash(5, dst);
                self.writer.emit32(imm as u32);
            }
            _ => unimplemented!(),
        }
    }

    pub fn sbb(&mut self, dst: Operand, src: Operand) {
        match (dst, src) {
            (Operand::Reg(_), Operand::Reg(src_reg)) => {
                self.emit_rex_mr(dst, src, 1);
                self.writer.emit8(0x19);
                self.emit_modrm_mr(dst, src_reg);
            }
            (Operand::Reg(_), Operand::Imm64(imm)) => {
                self.emit_rex_slash(dst, 1);
                self.writer.emit8(0x81);
                self.emit_modrm_slash(3, dst);
                self.writer.emit32(imm as u32);
            }
            _ => unimplemented!(),
        }
    }

    pub fn mul(&mut self, src: Operand) {
        match src {
            Operand::Reg(_) => {
                self.emit_rex_oi(src, 1);
                self.writer.emit8(0xf7);
                self.emit_modrm_slash(4, src);
            }
            _ => unimplemented!(),
        }
    }

    pub fn imul(&mut self, src: Operand) {
        match src {
            Operand::Reg(_) => {
                self.emit_rex_oi(src, 1);
                self.writer.emit8(0xf7);
                self.emit_modrm_slash(5, src);
            }
            _ => unimplemented!(),
        }
    }

    pub fn div(&mut self, src: Operand) {
        match src {
            Operand::Reg(_) => {
                self.emit_rex_oi(src, 1);
                self.writer.emit8(0xf7);
                self.emit_modrm_slash(6, src);
            }
            _ => unimplemented!(),
        }
    }

    pub fn idiv(&mut self, src: Operand) {
        match src {
            Operand::Reg(_) => {
                self.emit_rex_oi(src, 1);
                self.writer.emit8(0xf7);
                self.emit_modrm_slash(7, src);
            }
            _ => unimplemented!(),
        }
    }

    pub fn cqo(&mut self) {
        self.writer.emit8(0x48);
        self.writer.emit8(0x99);
    }

    pub fn call(&mut self, target: Operand) {
        match target {
            Operand::Reg(_) => {
                self.emit_rex_oi(target, 1);
                self.writer.emit8(0xff);
                self.emit_modrm_slash(2, target);
            }
            _ => unimplemented!(),
        }
    }

    pub fn ret(&mut self) {
        self.writer.emit8(0xc3);
    }

    pub fn mov(&mut self, dst: Operand, src: Operand) {
        match (dst, src) {
            // mov reg, reg
            (Operand::Reg(_), Operand::Reg(src_reg)) => {
                self.emit_rex_mr(dst, src, 1);
                self.writer.emit8(0x89);
                self.emit_modrm_mr(dst, src_reg);
            }
            // mov reg, imm
            (Operand::Reg(dst_reg), Operand::Imm64(imm)) => {
                self.emit_rex_oi(dst, 1);
                self.writer.emit8(0xb8 | Self::encode_reg(dst_reg));
                self.writer.emit64(imm as u64);
            }
            // mov [base_reg + offset], reg
            (Operand::MemDisp(_, _), Operand::Reg(src_reg)) => {
                self.emit_rex_mr(dst, src, 1);
                self.writer.emit8(0x89);
                self.emit_modrm_mr(dst, src_reg);
            }
            // mov [base_reg + offset], imm
            (Operand::MemDisp(_, _), Operand::Imm64(imm)) => {
                self.emit_rex_slash(dst, 1);
                self.writer.emit8(0xc7);
                self.emit_modrm_slash(0, dst);
                self.writer.emit32(imm as u32);
            }
            // mov reg, [base_reg + offset]
            (Operand::Reg(dst_reg), Operand::MemDisp(_, _)) => {
                self.emit_rex_rm(dst, src, 1);
                self.writer.emit8(0x8b);
                self.emit_modrm_rm(dst_reg, src);
            }
            _ => {
                dbg!(&dst, &src);
                unimplemented!()
            }
        }
    }

    pub fn push(&mut self, src: Operand) {
        match src {
            Operand::Reg(src_reg) => {
                self.emit_rex_oi(src, 1);
                self.writer.emit8(0x50 | Self::encode_reg(src_reg));
            }
            Operand::Imm64(imm) => {
                self.writer.emit8(0x68);
                self.writer.emit32(imm as u32);
            }
            _ => unimplemented!(),
        }
    }

    pub fn pop(&mut self, dst: Operand) {
        match dst {
            Operand::Reg(dst_reg) => {
                self.emit_rex_oi(dst, 1);
                self.writer.emit8(0x58 | Self::encode_reg(dst_reg));
            }
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x86_64_codegen_neg() {
        use Operand::*;
        use Reg64::*;

        let mut codegen = X86Asm::new();
        codegen.neg(Reg(Rax));
        codegen.neg(Reg(R8));
        codegen.neg(Reg(Rsp));

        codegen.dump_generated_code(0);

        let code = codegen.code();
        assert_eq!(
            code,
            &[
                0x48, 0xf7, 0xd8, // neg rax
                0x49, 0xf7, 0xd8, // neg r8
                0x48, 0xf7, 0xdc, // neg rsp
            ]
        );
    }

    #[test]
    fn test_x86_64_codegen_div() {
        use Operand::*;
        use Reg64::*;

        let mut codegen = X86Asm::new();
        codegen.div(Reg(Rax));
        codegen.div(Reg(R8));
        codegen.div(Reg(Rsp));

        let code = codegen.code();
        assert_eq!(
            code,
            &[
                0x48, 0xf7, 0xf0, // div rax
                0x49, 0xf7, 0xf0, // div r8
                0x48, 0xf7, 0xf4, // div rsp
            ]
        );
    }

    #[test]
    fn test_x86_64_codegen_idiv() {
        use Operand::*;
        use Reg64::*;

        let mut codegen = X86Asm::new();
        codegen.idiv(Reg(Rax));
        codegen.idiv(Reg(R8));
        codegen.idiv(Reg(Rsp));

        codegen.dump_generated_code(0);

        let code = codegen.code();
        assert_eq!(
            code,
            &[
                0x48, 0xf7, 0xf8, // idiv rax
                0x49, 0xf7, 0xf8, // idiv r8
                0x48, 0xf7, 0xfc, // idiv rsp
            ]
        );
    }

    #[test]
    fn test_x86_64_codegen_mul() {
        use Operand::*;
        use Reg64::*;

        let mut codegen = X86Asm::new();
        codegen.mul(Reg(Rax));
        codegen.mul(Reg(R8));
        codegen.mul(Reg(Rsp));

        let code = codegen.code();
        assert_eq!(
            code,
            &[
                0x48, 0xf7, 0xe0, // mul rax
                0x49, 0xf7, 0xe0, // mul r8
                0x48, 0xf7, 0xe4, // mul rsp
            ]
        );
    }

    #[test]
    fn test_x86_64_codegen_imul() {
        use Operand::*;
        use Reg64::*;

        let mut codegen = X86Asm::new();
        codegen.imul(Reg(Rax));
        codegen.imul(Reg(R8));
        codegen.imul(Reg(Rsp));

        codegen.dump_generated_code(0);

        let code = codegen.code();
        assert_eq!(
            code,
            &[
                0x48, 0xf7, 0xe8, // imul rax
                0x49, 0xf7, 0xe8, // imul r8
                0x48, 0xf7, 0xec, // imul rsp
            ]
        );
    }

    #[test]
    fn test_x86_64_codegen_sub() {
        use Operand::*;
        use Reg64::*;

        let mut codegen = X86Asm::new();
        codegen.sub(Reg(Rax), Reg(Rbx));
        codegen.sub(Reg(Rax), Imm64(0x1234));
        codegen.sub(Reg(R15), Reg(Rbp));
        codegen.sub(Reg(R8), Imm64(0x45464748));

        codegen.dump_generated_code(0);

        let code = codegen.code();
        assert_eq!(
            code,
            &[
                0x48, 0x29, 0xd8, // sub rax, rbx
                0x48, 0x81, 0xe8, 0x34, 0x12, 0x00, 0x00, // sub rax, 0x1234
                0x49, 0x29, 0xef, // sub r15, rbp
                0x49, 0x81, 0xe8, 0x48, 0x47, 0x46, 0x45, // sub r8, 0x45464748
            ]
        );
    }

    #[test]
    fn test_x86_64_codegen_sbb() {
        use Operand::*;
        use Reg64::*;

        let mut codegen = X86Asm::new();
        codegen.sbb(Reg(Rax), Reg(Rbx));
        codegen.sbb(Reg(Rax), Imm64(0x1234));
        codegen.sbb(Reg(R15), Reg(Rbp));
        codegen.sbb(Reg(R8), Imm64(0x45464748));

        codegen.dump_generated_code(0);

        let code = codegen.code();
        assert_eq!(
            code,
            &[
                0x48, 0x19, 0xd8, // sbb rax, rbx
                0x48, 0x81, 0xd8, 0x34, 0x12, 0x00, 0x00, // sbb rax, 0x1234
                0x49, 0x19, 0xef, // sbb r15, rbp
                0x49, 0x81, 0xd8, 0x48, 0x47, 0x46, 0x45, // sbb r8, 0x45464748
            ]
        );
    }

    #[test]
    fn test_x86_64_codegen_add() {
        use Operand::*;
        use Reg64::*;

        let mut codegen = X86Asm::new();
        codegen.add(Reg(Rax), Reg(Rbx));
        codegen.add(Reg(Rax), Imm64(0x1234));
        codegen.add(Reg(R15), Reg(Rbp));
        codegen.add(Reg(R8), Imm64(0x45464748));

        codegen.dump_generated_code(0);

        let code = codegen.code();
        assert_eq!(
            code,
            &[
                0x48, 0x01, 0xd8, // add rax, rbx
                0x48, 0x81, 0xc0, 0x34, 0x12, 0x00, 0x00, // add rax, 0x1234
                0x49, 0x01, 0xef, // add r15, rbp
                0x49, 0x81, 0xc0, 0x48, 0x47, 0x46, 0x45, // add r8, 0x45464748
            ]
        );
    }

    #[test]
    fn test_x86_64_codegen_call() {
        use Operand::*;
        use Reg64::*;

        let mut codegen = X86Asm::new();
        codegen.call(Reg(Rax));
        codegen.call(Reg(R15));

        codegen.dump_generated_code(0);

        let code = codegen.code();
        assert_eq!(
            code,
            &[
                0x48, 0xff, 0xd0, // call rax
                0x49, 0xff, 0xd7, // call r15
            ]
        );
    }

    #[test]
    fn test_x86_64_codegen_ret() {
        let mut codegen = X86Asm::new();
        codegen.ret();

        let code = codegen.code();
        assert_eq!(code, &[0xc3]); // ret
    }

    #[test]
    fn test_x86_64_codegen_mov() {
        use Operand::*;
        use Reg64::*;

        let mut codegen = X86Asm::new();
        codegen.mov(Reg(Rax), Imm64(0x123456789abcdef0));
        codegen.mov(Reg(Rax), Reg(Rbx));
        codegen.mov(MemDisp(Rax, 0x1337), Reg(Rbx));
        codegen.mov(Reg(Rax), MemDisp(Rbx, 0x41414141));
        codegen.mov(Reg(R8), Imm64(0x1234));
        codegen.mov(Reg(Rsp), Reg(R15));
        codegen.mov(MemDisp(R15, 0x12345678), Imm64(0x41424344));

        codegen.dump_generated_code(0);

        let code = codegen.code();
        assert_eq!(
            code,
            &[
                0x48, 0xb8, 0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12, // mov rax, 0x1234
                0x48, 0x89, 0xd8, // mov rax, rbx
                0x48, 0x89, 0x98, 0x37, 0x13, 0x00, 0x00, // mov [rax], rbx
                0x48, 0x8b, 0x83, 0x41, 0x41, 0x41, 0x41, // mov rbx, [rax]
                0x49, 0xb8, 0x34, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov r8, 0x1234
                0x4c, 0x89, 0xfc, // mov rsp, r15
                0x49, 0xc7, 0x87, 0x78, 0x56, 0x34, 0x12, 0x44, 0x43, 0x42,
                0x41, // mov [r15], 0x41424344
            ]
        );
    }

    #[test]
    fn test_x86_64_codegen_push() {
        use Operand::*;
        use Reg64::*;

        let mut codegen = X86Asm::new();
        codegen.push(Reg(Rax));
        codegen.push(Imm64(0x12345678));

        codegen.dump_generated_code(0);

        let code = codegen.code();
        assert_eq!(
            code,
            &[
                0x48, 0x50, // push rax
                0x68, 0x78, 0x56, 0x34, 0x12, // push 0x12345678
            ]
        );
    }

    #[test]
    fn test_x86_64_codegen_pop() {
        use Operand::*;
        use Reg64::*;

        let mut codegen = X86Asm::new();
        codegen.pop(Reg(Rax));

        codegen.dump_generated_code(0);

        let code = codegen.code();
        assert_eq!(code, &[0x48, 0x58]); // pop rax
    }
}
