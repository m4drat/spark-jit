mod instructions;

use crate::writer::Writer;

pub struct X86Asm {
    writer: Writer,
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

    fn emit_rex_slash(&mut self, arg: Operand, w: u8) {
        match arg {
            Operand::Reg(reg) | Operand::Mem(reg, _) => {
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

    fn emit_rex_mr(&mut self, dst: Operand, src: Operand, w: u8) {
        match (dst, src) {
            (Operand::Reg(dst_reg), Operand::Reg(src_reg))
                if (dst_reg as u8) < 8 && (src_reg as u8) < 8 && w == 0 =>
            {
                return;
            }
            (Operand::Reg(dst_reg), Operand::Reg(src_reg))
            | (Operand::Mem(dst_reg, _), Operand::Reg(src_reg)) => {
                let is_b = (dst_reg as u8 >= 8) as u8;
                let is_r = (src_reg as u8 >= 8) as u8;
                let rex = 0b0100_0000 | (w << 3) | (is_r << 2) | (0 << 1) | (is_b << 0);
                self.writer.emit8(rex);
            }
            _ => unimplemented!(),
        };
    }

    fn emit_rex_rm(&mut self, dst: Operand, src: Operand, w: u8) {
        match (dst, src) {
            (Operand::Reg(dst_reg), Operand::Reg(src_reg))
                if (dst_reg as u8) < 8 && (src_reg as u8) < 8 =>
            {
                return;
            }
            (Operand::Reg(dst_reg), Operand::Reg(src_reg))
            | (Operand::Reg(dst_reg), Operand::Mem(src_reg, _)) => {
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
            Operand::Mem(_, offset) => {
                self.writer
                    .emit8(((ModRM::MemDisp32 as u8) << 6) | modrm_low);
                self.writer.emit32(offset as u32);
            }
            _ => unimplemented!(),
        }
    }

    fn emit_modrm_rm(&mut self, dst: Register, src: Operand) {
        let src_reg = match src {
            Operand::Reg(src_reg) => src_reg,
            Operand::Mem(base_reg, _) => base_reg,
            _ => unimplemented!(),
        };

        let modrm_low = (Self::encode_reg(dst) << 3) | Self::encode_reg(src_reg);
        self.emit_modrm(modrm_low, src);
    }

    fn emit_modrm_mr(&mut self, dst: Operand, src: Register) {
        let dst_reg = match dst {
            Operand::Reg(dst_reg) => dst_reg,
            Operand::Mem(base_reg, _) => base_reg,
            _ => unimplemented!(),
        };

        let modrm_low = (Self::encode_reg(src) << 3) | Self::encode_reg(dst_reg);
        self.emit_modrm(modrm_low, dst);
    }

    fn emit_modrm_slash(&mut self, slash: u8, rm: Operand) {
        let reg = match rm {
            Operand::Reg(dst_reg) => dst_reg,
            Operand::Mem(base_reg, _) => base_reg,
            _ => unimplemented!(),
        };

        let modrm_low = (slash << 3) | Self::encode_reg(reg);
        self.emit_modrm(modrm_low, rm);
    }

    fn encode_reg(reg: Register) -> u8 {
        reg as u8 & 0b111
    }
}
