#![allow(non_snake_case, unused)]

use Register;

use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Opcode(pub u32);

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Instruction::*;
        match self.instruction() {
            Prnt | Prnl | Dump | Halt => self.print_special(f),
            B | Bl => self.print_b(f),
            Cbz | Cbnz | Beq | Bne | Bhs | Blo | Bmi | Bpl | Bvs | Bvc | Bhi | Bls | Bgt | Bge | Blt | Ble => self.print_cb(f),
            Movk | Movz => self.print_im(f),
            Ldur | Ldurb | Ldurh | Ldursw | Ldxr | Stur | Sturb | Sturh | Sturw | Stxr => self.print_d(f),
            Addi | Addis | Andi | Andis | Eori | Orri | Subi | Subis => self.print_i(f),
            Fmuls | Fdivs | Fcmps | Fadds | Fsubs | Fmuld | Fdivd | Fcmpd | Faddd | Fsubd | Ldurs |
                Ldurd | Mul | Sdiv | Smulh | Sturs | Sturd | Udiv | Umulh | Add | Adds | And | Ands |
                Br | Eor | Lsl | Lsr | Orr | Sub | Subs => self.print_r(f),
        }
    }
}

macro_rules! r {
    ($instruction:ident, $rm:ident, $rn:ident, $rd:ident) => {
        pub fn $instruction(rm: Register, rn: Register, rd: Register) -> Self {
            Opcode(Instruction::$instruction.as_u32() | (rm.as_u32() << 16) | (rn.as_u32() << 5) | rd.as_u32())
        }

        pub fn $rd(self) -> Register {
            Register((self.0 & 0b11111) as u8)
        }

        pub fn $rn(self) -> Register {
            Register(((self.0 >> 5) & 0b11111) as u8)
        }

        pub fn $rm(self) -> Register {
            Register(((self.0 >> 16) & 0b11111) as u8)
        }
    };
}

macro_rules! im {
    ($instruction:ident, $rd:ident, $imm:ident) => {
        pub fn $instruction(rd: Register, imm: u16) -> Self {
            Opcode(Instruction::$instruction.as_u32() | ((imm as u32) << 5) | rd.as_u32())
        }

        pub fn $rd(self) -> Register {
            Register((self.0 & 0b11111) as u8)
        }

        pub fn $imm(self) -> u16 {
            ((self.0 >> 5) & 0xff_ff) as u16
        }
    };
}

macro_rules! b {
    ($instruction:ident, $addr:ident, $set:ident) => {
        pub fn $instruction(addr: u32) -> Self {
            Opcode(Instruction::$instruction.as_u32() | (addr & 0b11111111111111111111111111))
        }

        pub fn $set(self, addr: u32) -> Self {
            Opcode(self.0 | (addr & 0b11111111111111111111111111))
        }

        pub fn $addr(self) -> u32 {
            let x = 0b10_00000000_00000000_00000000;
            let i = self.0 & 0b11_11111111_11111111_11111111;
            if (i & x) == x {
                i | 0b11111110_00000000_00000000_00000000
            } else {
                i
            }
        }
    };
}

macro_rules! cb {
    ($instruction:ident, $rt:ident, $addr:ident, $set:ident) => {
        pub fn $instruction(rt: Register, addr: u32) -> Self {
            Opcode(Instruction::$instruction.as_u32() | ((addr & 0b1111111111111111111) << 5) | rt.as_u32())
        }

        pub fn $rt(self) -> Register {
            Register((self.0 & 0b11111) as u8)
        }

        pub fn $addr(self) -> u32 {
            let x = 0b10000000_00000000_000;
            let i = (self.0 & 0b11111111_11111111_11100000) >> 5;
            if (i & x) == x {
                i | 0b11111111_11111100_00000000_00000000
            } else {
                i
            }
        }

        pub fn $set(self, addr: u32) -> Self {
            Opcode(self.0 | ((addr & 0b1111111111111111111) << 5))
        }
    };
}

macro_rules! bcond {
    ($instruction:ident, $addr:ident, $set:ident) => {
        pub fn $instruction(addr: u32) -> Self {
            Opcode(Instruction::$instruction.as_u32() | ((addr & 0b1111111111111111111) << 5))
        }

        pub fn $addr(self) -> u32 {
            let x = 0b10000000_00000000_000;
            let i = (self.0 & 0b11111111_11111111_11100000) >> 5;
            if (i & x) == x {
                i | 0b11111111_11111100_00000000_00000000
            } else {
                i
            }
        }

        pub fn $set(self, addr: u32) -> Self {
            Opcode(self.0 | ((addr & 0b1111111111111111111) << 5))
        }
    };
}

macro_rules! i {
    ($instruction:ident, $rn:ident, $rd:ident, $imm:ident) => {
        pub fn $instruction(rn: Register, rd: Register, imm: u16) -> Self {
            assert!(imm < 0b10000_00000000);
            Opcode(Instruction::$instruction.as_u32() | ((imm as u32) << 10) | (rn.as_u32() << 5) | rd.as_u32())
        }

        pub fn $rn(self) -> Register {
            Register(((self.0 >> 5) & 0b11111) as u8)
        }

        pub fn $rd(self) -> Register {
            Register((self.0 & 0b11111) as u8)
        }

        pub fn $imm(self) -> u16 {
            ((self.0 >> 10) & 0b1111_11111111) as u16
        }
    };
}

macro_rules! d {
    ($instruction:ident, $rn:ident, $rt:ident, $addr:ident) => {
        pub fn $instruction(rn: Register, rt: Register, addr: u16) -> Self {
            assert!(addr < 0b10_00000000);
            Opcode(Instruction::$instruction.as_u32() | ((addr as u32) << 11) | (rn.as_u32() << 5) | rt.as_u32())
        }

        pub fn $rn(self) -> Register {
            Register(((self.0 >> 5) & 0b11111) as u8)
        }

        pub fn $rt(self) -> Register {
            Register((self.0 & 0b11111) as u8)
        }

        pub fn $addr(self) -> u16 {
            ((self.0 >> 11) & 0b1_11111111) as u16
        }
    };
}

impl Opcode {
    pub fn instruction(&self) -> Instruction {
        Instruction::from(self.0)
    }

    fn print_special(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Instruction::*;
        match self.instruction() {
            Prnt => write!(f, "PRNT {}", self.prnt_rd()),
            Prnl => write!(f, "PRNL"),
            Dump => write!(f, "DUMP"),
            Halt => write!(f, "HALT"),
            _ => unreachable!(),
        }
    }

    fn print_b(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Instruction::*;
        match self.instruction() {
            // TODO: print in hex
            B => write!(f, "B {}", self.b_addr() as i32),
            Bl => write!(f, "BL {}", self.bl_addr() as i32),
            _ => unreachable!(),
        }
    }

    pub fn print_branch_label(&self, label: &str) -> String {
        use self::Instruction::*;
        match self.instruction() {
            Cbz => format!("CBZ {}, {}", self.cbz_rt(), label),
            Cbnz => format!("CBNZ {}, {}", self.cbnz_rt(), label),
            B => format!("B {}", label),
            Bl => format!("BL {}", label),
            Beq => format!("B.EQ {}", label),
            Bne => format!("B.NE {}", label),
            Bhs => format!("B.HS {}", label),
            Blo => format!("B.LO {}", label),
            Bmi => format!("B.MI {}", label),
            Bpl => format!("B.PL {}", label),
            Bvs => format!("B.VS {}", label),
            Bvc => format!("B.VC {}", label),
            Bhi => format!("B.HI {}", label),
            Bls => format!("B.LS {}", label),
            Bgt => format!("B.GT {}", label),
            Bge => format!("B.GE {}", label),
            Blt => format!("B.LT {}", label),
            Ble => format!("B.LE {}", label),
            _ => unreachable!(),
        }
    }

    fn print_cb(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Instruction::*;
        match self.instruction() {
            Cbz => write!(f, "CBZ {}, {}", self.cbz_rt(), self.cbz_addr()),
            Cbnz => write!(f, "CBNZ {}, {}", self.cbnz_rt(), self.cbnz_addr()),
            Beq => write!(f, "B.EQ {}", self.beq_addr()),
            Bne => write!(f, "B.NE {}", self.bne_addr()),
            Bhs => write!(f, "B.HS {}", self.bhs_addr()),
            Blo => write!(f, "B.LO {}", self.blo_addr()),
            Bmi => write!(f, "B.MI {}", self.bmi_addr()),
            Bpl => write!(f, "B.PL {}", self.bpl_addr()),
            Bvs => write!(f, "B.VS {}", self.bvs_addr()),
            Bvc => write!(f, "B.VC {}", self.bvc_addr()),
            Bhi => write!(f, "B.HI {}", self.bhi_addr()),
            Bls => write!(f, "B.LS {}", self.bls_addr()),
            Bgt => write!(f, "B.GT {}", self.bgt_addr()),
            Bge => write!(f, "B.GE {}", self.bge_addr()),
            Blt => write!(f, "B.LT {}", self.blt_addr()),
            Ble => write!(f, "B.LE {}", self.ble_addr()),
            _ => unreachable!(),
        }
    }

    fn print_im(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Instruction::*;
        match self.instruction() {
            // TODO: handle shift amount
            Movk => write!(f, "MOVK {}, {}", self.movk_rd(), self.movk_imm()),
            Movz => write!(f, "MOVZ {}, {}", self.movz_rd(), self.movz_imm()),
            _ => unreachable!(),
        }
    }

    fn print_d(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Instruction::*;
        match self.instruction() {
            Ldur => write!(f, "LDUR {}, [{}, #{}]", self.ldur_rt(), self.ldur_rn(), self.ldur_addr()),
            Ldurb => write!(f, "LDURB {}, [{}, #{}]", self.ldurb_rt(), self.ldurb_rn(), self.ldurb_addr()),
            Ldurh => write!(f, "LDURH {}, [{}, #{}]", self.ldurh_rt(), self.ldurh_rn(), self.ldurh_addr()),
            Ldursw => write!(f, "LDURSW {}, [{}, #{}]", self.ldursw_rt(), self.ldursw_rn(), self.ldursw_addr()),
            Ldxr => write!(f, "LDXR {}, [{}, #{}]", self.ldxr_rt(), self.ldxr_rn(), self.ldxr_addr()),
            Stur => write!(f, "STUR {}, [{}, #{}]", self.stur_rt(), self.stur_rn(), self.stur_addr()),
            Sturb => write!(f, "STURB {}, [{}, #{}]", self.sturb_rt(), self.sturb_rn(), self.sturb_addr()),
            Sturh => write!(f, "STURH {}, [{}, #{}]", self.sturh_rt(), self.sturh_rn(), self.sturh_addr()),
            Sturw => write!(f, "STURW {}, [{}, #{}]", self.sturw_rt(), self.sturw_rn(), self.sturw_addr()),
            Stxr => write!(f, "STXR {}, [{}, #{}]", self.stxr_rt(), self.stxr_rn(), self.stxr_addr()),
            _ => unreachable!(),
        }
    }

    fn print_i(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Instruction::*;
        match self.instruction() {
            Addi => write!(f, "ADDI {}, {}, #{}", self.addi_rd(), self.addi_rn(), self.addi_imm()),
            Addis => write!(f, "ADDIS {}, {}, #{}", self.addis_rd(), self.addis_rn(), self.addis_imm()),
            Andi => write!(f, "ANDI {}, {}, #{}", self.andi_rd(), self.andi_rn(), self.andi_imm()),
            Andis => write!(f, "ANDIS {}, {}, #{}", self.andis_rd(), self.andis_rn(), self.andis_imm()),
            Eori => write!(f, "EORI {}, {}, #{}", self.eori_rd(), self.eori_rn(), self.eori_imm()),
            Orri => write!(f, "ORRI {}, {}, #{}", self.orri_rd(), self.orri_rn(), self.orri_imm()),
            Subi => write!(f, "SUBI {}, {}, #{}", self.subi_rd(), self.subi_rn(), self.subi_imm()),
            Subis => write!(f, "SUBIS {}, {}, #{}", self.subis_rd(), self.subis_rn(), self.subis_imm()),
            _ => unreachable!(),
        }
    }

    fn print_r(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Instruction::*;
        match self.instruction() {
            Add => write!(f, "ADD {}, {}, {}", self.add_rd(), self.add_rn(), self.add_rm()),
            Adds => write!(f, "ADDS {}, {}, {}", self.adds_rd(), self.adds_rn(), self.adds_rm()),
            And => write!(f, "AND {}, {}, {}", self.and_rd(), self.and_rn(), self.and_rm()),
            Ands => write!(f, "ANDS {}, {}, {}", self.ands_rd(), self.ands_rn(), self.ands_rm()),
            Eor => write!(f, "ANDS {}, {}, {}", self.eor_rd(), self.eor_rn(), self.eor_rm()),
            Orr => write!(f, "ANDS {}, {}, {}", self.orr_rd(), self.orr_rn(), self.orr_rm()),
            Sub => write!(f, "SUB {}, {}, {}", self.sub_rd(), self.sub_rn(), self.sub_rm()),
            Subs => write!(f, "SUBS {}, {}, {}", self.subs_rd(), self.subs_rn(), self.subs_rm()),
            Lsl => write!(f, "LSL {}, {}, #{}", self.lsl_rd(), self.lsl_rn(), self.lsl_shamt()),
            Lsr => write!(f, "LSR {}, {}, #{}", self.lsr_rd(), self.lsr_rn(), self.lsr_shamt()),
            Br => write!(f, "BR {}", self.br_rt()),
            // TODO: arithmetic instructions
            _ => unreachable!(),
        }
    }

    pub fn Prnt(rd: Register) -> Self {
        Opcode(Instruction::Prnt.as_u32() | rd.as_u32())
    }

    pub fn prnt_rd(self) -> Register {
        Register((self.0 & 0b11111) as u8)
    }

    pub fn Prnl() -> Self {
        Opcode(Instruction::Prnl.as_u32())
    }

    pub fn Dump() -> Self {
        Opcode(Instruction::Dump.as_u32())
    }

    pub fn Halt() -> Self {
        Opcode(Instruction::Halt.as_u32())
    }

    b!(B, b_addr, b_set_addr);
    b!(Bl, bl_addr, bl_set_addr);

    cb!(Cbz, cbz_rt, cbz_addr, cbz_set_addr);
    cb!(Cbnz, cbnz_rt, cbnz_addr, cbnz_set_addr);
    bcond!(Beq, beq_addr, beq_set_addr);
    bcond!(Bne, bne_addr, bne_set_addr);
    bcond!(Bhs, bhs_addr, bhs_set_addr);
    bcond!(Blo, blo_addr, blo_set_addr);
    bcond!(Bmi, bmi_addr, bmi_set_addr);
    bcond!(Bpl, bpl_addr, bpl_set_addr);
    bcond!(Bvs, bvs_addr, bvs_set_addr);
    bcond!(Bvc, bvc_addr, bvc_set_addr);
    bcond!(Bhi, bhi_addr, bhi_set_addr);
    bcond!(Bls, bls_addr, bls_set_addr);
    bcond!(Bgt, bgt_addr, bgt_set_addr);
    bcond!(Bge, bge_addr, bge_set_addr);
    bcond!(Blt, blt_addr, blt_set_addr);
    bcond!(Ble, ble_addr, ble_set_addr);

    im!(Movk, movk_rd, movk_imm);
    im!(Movz, movz_rd, movz_imm);

    i!(Addi, addi_rn, addi_rd, addi_imm);
    i!(Addis, addis_rn, addis_rd, addis_imm);
    i!(Andi, andi_rn, andi_rd, andi_imm);
    i!(Andis, andis_rn, andis_rd, andis_imm);
    i!(Eori, eori_rn, eori_rd, eori_imm);
    i!(Orri, orri_rn, orri_rd, orri_imm);
    i!(Subi, subi_rn, subi_rd, subi_imm);
    i!(Subis, subis_rn, subis_rd, subis_imm);

    d!(Ldur, ldur_rn, ldur_rt, ldur_addr);
    d!(Ldurb, ldurb_rn, ldurb_rt, ldurb_addr);
    d!(Ldurh, ldurh_rn, ldurh_rt, ldurh_addr);
    d!(Ldursw, ldursw_rn, ldursw_rt, ldursw_addr);
    d!(Ldxr, ldxr_rn, ldxr_rt, ldxr_addr);
    d!(Stur, stur_rn, stur_rt, stur_addr);
    d!(Sturb, sturb_rn, sturb_rt, sturb_addr);
    d!(Sturh, sturh_rn, sturh_rt, sturh_addr);
    d!(Sturw, sturw_rn, sturw_rt, sturw_addr);
    d!(Stxr, stxr_rn, stxr_rt, stxr_addr);

    r!(Add, add_rm, add_rn, add_rd);
    r!(Adds, adds_rm, adds_rn, adds_rd);
    r!(And, and_rm, and_rn, and_rd);
    r!(Ands, ands_rm, ands_rn, ands_rd);

    pub fn Br(rt: Register) -> Self {
        Opcode(Instruction::Br.as_u32() | (rt.as_u32() << 5))
    }

    pub fn br_rt(self) -> Register {
        Register(((self.0 >> 5) & 0b111110) as u8)
    }

    r!(Eor, eor_rm, eor_rn, eor_rd);

    pub fn Lsl(rn: Register, rd: Register, imm: u32) -> Self {
        assert!(imm < 64);
        Opcode(Instruction::Lsl.as_u32() | ((imm & 0b111111) << 10) | (rn.as_u32() << 5) | rd.as_u32())
    }

    pub fn lsl_rd(self) -> Register {
        Register((self.0 & 0b11111) as u8)
    }

    pub fn lsl_rn(self) -> Register {
        Register(((self.0 >> 5) & 0b11111) as u8)
    }

    pub fn lsl_shamt(self) -> u8 {
        ((self.0 >> 10) & 0b111111) as u8
    }

    pub fn Lsr(rn: Register, rd: Register, imm: u32) -> Self {
        assert!(imm < 64);
        Opcode(Instruction::Lsr.as_u32() | ((imm & 0b111111) << 10) | (rn.as_u32() << 5) | rd.as_u32())
    }

    pub fn lsr_rd(self) -> Register {
        Register((self.0 & 0b11111) as u8)
    }

    pub fn lsr_rn(self) -> Register {
        Register(((self.0 >> 5) & 0b11111) as u8)
    }

    pub fn lsr_shamt(self) -> u8 {
        ((self.0 >> 10) & 0b111111) as u8
    }

    r!(Orr, orr_rm, orr_rn, orr_rd);
    r!(Sub, sub_rm, sub_rn, sub_rd);
    r!(Subs, subs_rm, subs_rn, subs_rd);
    r!(Fadds, fadds_rm, fadds_rn, fadds_rd);
    r!(Fcmps, fcmps_rm, fcmps_rn, fcmps_rd);
    r!(Fdivs, fdivs_rm, fdivs_rn, fdivs_rd);
    r!(Fmuls, fmuls_rm, fmuls_rn, fmuls_rd);
    r!(Fsubs, fsubs_rm, fsubs_rn, fsubs_rd);
    r!(Faddd, faddd_rm, faddd_rn, faddd_rd);
    r!(Fcmpd, fcmpd_rm, fcmpd_rn, fcmpd_rd);
    r!(Fdivd, fdivd_rm, fdivd_rn, fdivd_rd);
    r!(Fmuld, fmuld_rm, fmuld_rn, fmuld_rd);
    r!(Fsubd, fsubd_rm, fsubd_rn, fsubd_rd);
    r!(Ldurs, ldurs_rm, ldurs_rn, ldurs_rd);
    r!(Ldurd, ldurd_rm, ldurd_rn, ldurd_rd);
    r!(Mul, mul_rm, mul_rn, mul_rd);
    r!(Sdiv, sdiv_rm, sdiv_rn, sdiv_rd);
    r!(Smulh, smulh_rm, smulh_rn, smulh_rd);
    r!(Sturs, sturs_rm, sturs_rn, sturs_rd);
    r!(Sturd, sturd_rm, sturd_rn, sturd_rd);
    r!(Udiv, udiv_rm, udiv_rn, udiv_rd);
    r!(Umulh, umulh_rm, umulh_rn, umulh_rd);
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Instruction {
    B,
    Fmuls,
    Fdivs,
    Fcmps,
    Fadds,
    Fsubs,
    Fmuld,
    Fdivd,
    Fcmpd,
    Faddd,
    Fsubd,
    Sturb,
    Ldurb,
    Beq,
    Bne,
    Bhs,
    Blo,
    Bmi,
    Bpl,
    Bvs,
    Bvc,
    Bhi,
    Bls,
    Bgt,
    Blt,
    Bge,
    Ble,
    Sturh,
    Ldurh,
    And,
    Add,
    Addi,
    Andi,
    Bl,
    Sdiv,
    Udiv,
    Mul,
    Smulh,
    Umulh,
    Orr,
    Adds,
    Addis,
    Orri,
    Cbz,
    Cbnz,
    Sturw,
    Ldursw,
    Sturs,
    Ldurs,
    Stxr,
    Ldxr,
    Eor,
    Sub,
    Subi,
    Eori,
    Movz,
    Lsr,
    Lsl,
    Br,
    Ands,
    Subs,
    Subis,
    Andis,
    Movk,
    Stur,
    Ldur,
    Sturd,
    Ldurd,
    Prnt,
    Prnl,
    Dump,
    Halt,
}

impl Instruction {
    pub fn from_str(s: &str) -> Option<Self> {
        use self::Instruction::*;
        Some(match s {
            "B" => B,
            "FMULS" => Fmuls,
            "FDIVS" => Fdivs,
            "FCMPS" => Fcmps,
            "FADDS" => Fadds,
            "FSUBS" => Fsubs,
            "FMULD" => Fmuld,
            "FDIVD" => Fdivd,
            "FCMPD" => Fcmpd,
            "FADDD" => Faddd,
            "FSUBD" => Fsubd,
            "STURB" => Sturb,
            "LDURB" => Ldurb,
            "B.EQ" => Beq,
            "B.NE" => Bne,
            "B.HS" => Bhs,
            "B.LO" => Blo,
            "B.MI" => Bmi,
            "B.PL" => Bpl,
            "B.VS" => Bvs,
            "B.VC" => Bvc,
            "B.HI" => Bhi,
            "B.LS" => Bls,
            "B.GT" => Bgt,
            "B.LT" => Blt,
            "B.GE" => Bge,
            "B.LE" => Ble,
            "STURH" => Sturh,
            "LDURH" => Ldurh,
            "AND" => And,
            "ADD" => Add,
            "ADDI" => Addi,
            "ANDI" => Andi,
            "BL" => Bl,
            "SDIV" => Sdiv,
            "UDIV" => Udiv,
            "MUL" => Mul,
            "SMULH" => Smulh,
            "UMULH" => Umulh,
            "ORR" => Orr,
            "ADDS" => Adds,
            "ADDIS" => Addis,
            "ORRI" => Orri,
            "CBZ" => Cbz,
            "CBNZ" => Cbnz,
            "STURW" => Sturw,
            "LDURSW" => Ldursw,
            "STURS" => Sturs,
            "LDURS" => Ldurs,
            "STXR" => Stxr,
            "LDXR" => Ldxr,
            "EOR" => Eor,
            "SUB" => Sub,
            "SUBI" => Subi,
            "EORI" => Eori,
            "MOVZ" => Movz,
            "LSR" => Lsr,
            "LSL" => Lsl,
            "BR" => Br,
            "ANDS" => Ands,
            "SUBS" => Subs,
            "SUBIS" => Subis,
            "ANDIS" => Andis,
            "MOVK" => Movk,
            "STUR" => Stur,
            "LDUR" => Ldur,
            "STURD" => Sturd,
            "LDURD" => Ldurd,
            "PRNT" => Prnt,
            "PRNL" => Prnl,
            "DUMP" => Dump,
            "HALT" => Halt,
            _ => return None,
        })
    }

    fn as_u32(&self) -> u32 {
        use self::Instruction::*;
        match self {
            // B instructions
            B => 0b000101 << 26,
            Bl => 0b100101 << 26,
            // CB instructions
            Beq | Bne | Bhs | Blo | Bmi | Bpl | Bvs | Bvc |
            Bhi | Bls | Bgt | Blt | Ble | Bge => (0b1010100 << 24) | match self {
                Beq => 0x00,
                Bne => 0x01,
                Bhs => 0x02,
                Blo => 0x03,
                Bmi => 0x04,
                Bpl => 0x05,
                Bvs => 0x06,
                Bvc => 0x07,
                Bhi => 0x08,
                Bls => 0x09,
                Bgt => 0x0c,
                Bge => 0x0a,
                Blt => 0x0b,
                Ble => 0x0d,
                _ => unreachable!(),
            },
            Cbz => 0b10110100 << 24,
            Cbnz => 0b10110101 << 24,
            // IM instructions
            Movz => 0b110100101 << 23,
            Movk => 0b111100101 << 23,
            // I instructions
            Addi => 0b1001000100 << 22,
            Andi => 0b1001001000 << 22,
            Addis => 0b1011000100 << 22,
            Orri => 0b1011001000 << 22,
            Subi => 0b1101000100 << 22,
            Eori => 0b1101001000 << 22,
            Subis => 0b1111000100 << 22,
            Andis => 0b1111001000 << 22,
            // D instructions
            Sturb => 0b00111000000 << 21,
            Ldurb => 0b00111000010 << 21,
            Sturh => 0b01111000000 << 21,
            Ldurh => 0b01111000010 << 21,
            Sturw => 0b10111000000 << 21,
            Ldursw => 0b10111000100 << 21,
            Stxr => 0b11001000000 << 21,
            Ldxr => 0b11001000010 << 21,
            Stur => 0b11111000000 << 21,
            Ldur => 0b11111000010 << 21,
            // R instructions
            And => 0b10001010000 << 21,
            Smulh => 0b10011011010 << 21,
            Umulh => 0b10011011110 << 21,
            Orr => 0b10101010000 << 21,
            Adds => 0b10101011000 << 21,
            Sturs => 0b10111100000 << 21,
            Ldurs => 0b10111100010 << 21,
            Eor => 0b11001010000 << 21,
            Sub => 0b11001011000 << 21,
            Lsr => 0b11010011010 << 21,
            Lsl => 0b11010011011 << 21,
            Br => 0b11010110000 << 21,
            Ands => 0b11101010000 << 21,
            Subs => 0b11101011000 << 21,
            Sturd => 0b11111100000 << 21,
            Ldurd => 0b11111100010 << 21,
            Add => 0b10001011000 << 21,
            Sdiv | Udiv => (0b10011010110 << 21) | if Sdiv == *self { 0b000010 << 10 } else { 0b000011 << 10 },
            Mul => (0b10011011000 << 21) | if Mul == *self { 0b011111 << 10 } else { 0 },
            Fmuls | Fdivs | Fcmps | Fadds | Fsubs => (0b00011110001 << 21) | match self {
                Fmuls => 0b000010 << 10,
                Fdivs => 0b000110 << 10,
                Fcmps => 0b001000 << 10,
                Fadds => 0b001010 << 10,
                Fsubs => 0b001110 << 10,
                _ => unreachable!(),
            },
            Fmuld | Fdivd | Fcmpd | Faddd | Fsubd => (0b00011110011 << 21) | match self {
                Fmuld => 0b000010 << 10,
                Fdivd => 0b000110 << 10,
                Fcmpd => 0b001000 << 10,
                Faddd => 0b001010 << 10,
                Fsubd => 0b001110 << 10,
                _ => unreachable!(),
            },

            Prnt => 0b11111111101 << 21,
            Prnl => 0b11111111100 << 21,
            Dump => 0b11111111110 << 21,
            Halt => 0b11111111111 << 21,
        }
    }
}

impl From<u32> for Instruction {
    fn from(r: u32) -> Self {
        use self::Instruction::*;
        // check if branch instruction
        match r >> 26 {
            0b000101 => return B,
            0b100101 => return Bl,
            _ => (),
        }

        // check if conditional branch instruction
        match r >> 24 {
            0b10110100 => return Cbz,
            0b10110101 => return Cbnz,
            0b01010100 => match r & 0b11111 {
                0x00 => return Beq,
                0x01 => return Bne,
                0x02 => return Bhs,
                0x03 => return Blo,
                0x04 => return Bmi,
                0x05 => return Bpl,
                0x06 => return Bvs,
                0x07 => return Bvc,
                0x08 => return Bhi,
                0x09 => return Bls,
                0x0c => return Bgt,
                0x0a => return Bge,
                0x0b => return Blt,
                0x0d => return Ble,
                _ => unreachable!(),
            }
            _ => (),
        }

        // check if IM instruction
        match r >> 23 {
            0b110100101 => return Movz,
            0b111100101 => return Movk,
            _ => (),
        }

        // check if I instruction
        match r >> 22 {
            0b1001000100 => return Addi,
            0b1001001000 => return Andi,
            0b1011000100 => return Addis,
            0b1011001000 => return Orri,
            0b1101000100 => return Subi,
            0b1101001000 => return Eori,
            0b1111000100 => return Subis,
            0b1111001000 => return Andis,
            _ => (),
        }

        // check if D instruction
        match r >> 21 {
            0b00111000000 => return Sturb,
            0b00111000010 => return Ldurb,
            0b01111000000 => return Sturh,
            0b01111000010 => return Ldurh,
            0b10111000000 => return Sturw,
            0b10111000100 => return Ldursw,
            0b11001000000 => return Stxr,
            0b11001000010 => return Ldxr,
            0b11111000000 => return Stur,
            0b11111000010 => return Ldur,
            _ => (),
        }

        // check if R instruction
        match r >> 21 {
            0b10001010000 => return And,
            0b10001011000 => return Add,
            0b10011011000 => return Mul,
            // sdiv/udiv
            0b10011010110 => match (r >> 10) & 0b111111 {
                0b000010 => return Sdiv,
                0b000011 => return Udiv,
                _ => (),
            },
            0b10011011010 => return Smulh,
            0b10011011110 => return Umulh,
            0b10101010000 => return Orr,
            0b10101011000 => return Adds,
            0b10111100000 => return Sturs,
            0b10111100010 => return Ldurs,
            0b11001010000 => return Eor,
            0b11001011000 => return Sub,
            0b11010011010 => return Lsr,
            0b11010011011 => return Lsl,
            0b11010110000 => return Br,
            0b11101010000 => return Ands,
            0b11101011000 => return Subs,
            0b11111100000 => return Sturd,
            0b11111100010 => return Ldurd,
            // floating point instructions
            0b00011110001 => match (r >> 10) & 0b111111 {
                0b000010 => return Fmuls,
                0b000110 => return Fdivs,
                0b001000 => return Fcmps,
                0b001010 => return Fadds,
                0b001110 => return Fsubs,
                _ => (),
            },
            0b00011110011 => match (r >> 10) & 0b111111 {
                0b000010 => return Fmuld,
                0b000110 => return Fdivd,
                0b001000 => return Fcmpd,
                0b001010 => return Faddd,
                0b001110 => return Fsubd,
                _ => (),
            },
            0b11111111101 => return Prnt,
            0b11111111100 => return Prnl,
            0b11111111110 => return Dump,
            0b11111111111=> return Halt,
            _ => (),
        }

        unreachable!();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn abc() {
        let i = 0b00010100_00000000_00000000_00000000;
        assert_eq!(Instruction::B, Instruction::from(i));
        let i = 0b10010100_00000000_00000000_00000000;
        assert_eq!(Instruction::Bl, Instruction::from(i));
        let a = Opcode::Add(Register(31), Register(31), Register(10)).0;
        let b = 0b10001011000_11111_000000_11111_01010;
        println!("{:032b}", a);
        println!("{:032b}", b);
        assert_eq!(a, b);
    }
}
