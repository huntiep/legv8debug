use Register;
use bytecode::{Instruction, Opcode};

use std::collections::HashMap;

pub fn assemble(program: &str) -> (Vec<Opcode>, Vec<usize>) {
    let mut code = Vec::new();
    let mut line_map = Vec::new();
    let mut labels = HashMap::new();
    let mut jumps = Vec::new();

    let mut line_number = 0;
    let mut i = 0;
    for line in program.lines() {
        line_number += 1;

        let line = line.trim();
        if line.is_empty() {
            line_map.push(i);
            continue;
        } else if line.ends_with(':') {
            line_map.push(i);
            let line = &line[0..line.len()-1];
            if labels.contains_key(line) {
                panic!("label {} define more than once on line {}", line, line_number);
            }
            labels.insert(line, i);
            continue;
        }

        line_map.push(i);

        let line: Vec<_> = line.split_whitespace().collect();
        code.push(match line[0] {
            // Pseduo instructions
            "PRNT" => Opcode::Prnt(),
            "PRN" => Opcode::Prn(),
            "DUMP" => Opcode::Dump(),
            // D instructions
            "STUR" => handle_d(Instruction::Stur, line, line_number),
            "LDUR" => handle_d(Instruction::Ldur, line, line_number),
            // B instructions
            "B" => handle_b(Instruction::B, line, &labels, &mut jumps, i, line_number),
            "BL" => handle_b(Instruction::Bl, line, &labels, &mut jumps, i, line_number),
            // CB instructions
            "CBZ" => handle_cb(Instruction::Cbz, line, &labels, &mut jumps, i, line_number),
            "CBNZ" => handle_cb(Instruction::Cbnz, line, &labels, &mut jumps, i, line_number),
            // NOTE these are encoded differently but they are written the same as B-form
            // instructions.
            "B.EQ" => handle_b(Instruction::Beq, line, &labels, &mut jumps, i, line_number),
            "B.GT" => handle_b(Instruction::Bgt, line, &labels, &mut jumps, i, line_number),
            "B.GE" => handle_b(Instruction::Bge, line, &labels, &mut jumps, i, line_number),
            "B.LT" => handle_b(Instruction::Blt, line, &labels, &mut jumps, i, line_number),
            "B.LE" => handle_b(Instruction::Ble, line, &labels, &mut jumps, i, line_number),
            // I instructions
            "ADDI" => handle_i(Instruction::Addi, line, line_number),
            "SUBI" => handle_i(Instruction::Subi, line, line_number),
            // R instructions
            "ADD" => handle_r(Instruction::Add, line, line_number),
            "SUB" => handle_r(Instruction::Sub, line, line_number),
            "SUBS" => handle_r(Instruction::Subs, line, line_number),
            "LSL" => handle_shift(Instruction::Lsl, line, line_number),
            "LSR" => handle_shift(Instruction::Lsr, line, line_number),
            "BR" => {
                if line.len() != 2 {
                    panic!("Invalid instruction on line {}", line_number);
                }
                let r = read_register(line[1], false, line_number);
                Opcode::Br(r)
            }
            _ => panic!("Invalid instruction name {} on line {}", line[0], line_number),
        });

        i += 1;
    }

    for (l, pos) in jumps {
        // NOTE that these must be forward jumps as they would otherwise already have been handled.
        if let Some(i) = labels.get(l.as_str()) {
            let addr = (*i - pos) as u32;
            match code[pos].instruction() {
                Instruction::B => code[pos] = code[pos].b_set_addr(addr),
                Instruction::Bl => code[pos] = code[pos].bl_set_addr(addr),
                Instruction::Cbz => code[pos] = code[pos].cbz_set_addr(addr),
                Instruction::Cbnz => code[pos] = code[pos].cbnz_set_addr(addr),
                Instruction::Beq => code[pos] = code[pos].beq_set_addr(addr),
                Instruction::Bgt => code[pos] = code[pos].bgt_set_addr(addr),
                Instruction::Bge => code[pos] = code[pos].bge_set_addr(addr),
                Instruction::Blt => code[pos] = code[pos].blt_set_addr(addr),
                Instruction::Ble => code[pos] = code[pos].ble_set_addr(addr),
                _ => unreachable!(),
            }
        } else {
            panic!("Label {} not found", l);
        }
    }

    (code, line_map)
}

fn read_register(r: &str, trailing_comma: bool, line_number: usize) -> Register {
    if trailing_comma && r.ends_with(',') {
        read_register(&r[0..r.len()-1], false, line_number)
    } else if !trailing_comma && r.ends_with(',') {
        panic!("Expected comma after register {} on line {}", r, line_number);
    } else if trailing_comma && !r.ends_with(',') {
        panic!("Unexpected comma after register {} on line {}", r, line_number);
    } else if let Some(r) = Register::from_str(r) {
        r
    } else {
        panic!("Invalid register name {} on line {}", r, line_number);
    }
}

fn read_imm(imm: &str, line_number: usize) -> u16 {
    if !imm.starts_with('#') {
        panic!("Invalid format for immediate value {} on line {}", imm, line_number);
    }

    if let Ok(i) = &imm[1..].parse() {
        *i
    } else {
        panic!("Invalid format for immediate value {} on line {}", imm, line_number);
    }
}

fn handle_i(instr: Instruction, line: Vec<&str>, line_number: usize) -> Opcode {
    if line.len() != 4 {
        panic!("Invalid instruction on line {}", line_number);
    }

    let rd = read_register(line[1], true, line_number);
    let rn = read_register(line[2], true, line_number);
    let imm = read_imm(line[3], line_number);

    match instr {
        Instruction::Addi => Opcode::Addi(rn, rd, imm),
        Instruction::Subi => Opcode::Subi(rn, rd, imm),
        _ => unreachable!(),
    }
}

fn handle_r(instr: Instruction, line: Vec<&str>, line_number: usize) -> Opcode {
    if line.len() != 4 {
        panic!("Invalid instruction on line {}", line_number);
    }

    let rd = read_register(line[1], true, line_number);
    let rn = read_register(line[2], true, line_number);
    let rm = read_register(line[3], false, line_number);

    match instr {
        Instruction::Add => Opcode::Add(rm, rn, rd),
        Instruction::Sub => Opcode::Sub(rm, rn, rd),
        Instruction::Subs => Opcode::Subs(rm, rn, rd),
        _ => unreachable!(),
    }
}

fn handle_shift(instr: Instruction, line: Vec<&str>, line_number: usize) -> Opcode {
    if line.len() != 4 {
        panic!("Invalid instruction on line {}", line_number);
    }

    let rd = read_register(line[1], true, line_number);
    let rn = read_register(line[2], true, line_number);
    let imm = read_imm(line[3], line_number);

    match instr {
        Instruction::Lsl => Opcode::Lsl(rn, rd, imm as u32),
        Instruction::Lsr => Opcode::Lsr(rn, rd, imm as u32),
        _ => unreachable!(),
    }
}

fn handle_b(instr: Instruction, line: Vec<&str>, labels: &HashMap<&str, usize>,
            jumps: &mut Vec<(String, usize)>, code_pos: usize, line_number: usize)
    -> Opcode
{
    if line.len() != 2 {
        panic!("Invalid instruction on line {}", line_number);
    }

    let addr = handle_label(line[1], labels, jumps, code_pos);

    match instr {
        Instruction::B => Opcode::B(addr),
        Instruction::Bl => Opcode::Bl(addr),
        Instruction::Beq => Opcode::Beq(addr),
        Instruction::Bgt => Opcode::Bgt(addr),
        Instruction::Bge => Opcode::Bge(addr),
        Instruction::Blt => Opcode::Blt(addr),
        Instruction::Ble => Opcode::Ble(addr),
        _ => unreachable!(),
    }
}

fn handle_label(label: &str, labels: &HashMap<&str, usize>, jumps: &mut Vec<(String, usize)>, code_pos: usize) -> u32 {
    // NOTE if the label is found it must be behind this point.
    if let Some(i) = labels.get(label) {
        -((code_pos as i32) - (*i as i32)) as u32
    } else {
        jumps.push((label.to_string(), code_pos));
        0
    }
}

fn handle_cb(instr: Instruction, line: Vec<&str>, labels: &HashMap<&str, usize>,
             jumps: &mut Vec<(String, usize)>, code_pos: usize, line_number: usize)
    -> Opcode
{
    if line.len() != 3 {
        panic!("Invalid instruction on line {}", line_number);
    }

    let rt = read_register(line[1], true, line_number);

    let addr = handle_label(line[2], labels, jumps, code_pos);

    match instr {
        Instruction::Cbz => Opcode::Cbz(rt, addr),
        Instruction::Cbnz => Opcode::Cbnz(rt, addr),
        _ => unreachable!(),
    }
}

fn handle_d(instr: Instruction, line: Vec<&str>, line_number: usize) -> Opcode {
    if line.len() != 4 {
        panic!("Invalid instruction on line {}", line_number);
    }

    let rt = read_register(line[1], true, line_number);
    if !line[2].starts_with('[') {
        panic!("Invalid instruction on line {}", line_number);
    }

    let rn = read_register(&line[2][1..], true, line_number);

    let imm = line[3];
    if !imm.ends_with(']') {
        panic!("Invalid instruction on line {}", line_number);
    }
    let addr = read_imm(&imm[0..imm.len()-1],  line_number);

    match instr {
        Instruction::Stur => Opcode::Stur(rn, rt, addr),
        Instruction::Ldur => Opcode::Ldur(rn, rt, addr),
        _ => unreachable!(),
    }
}
