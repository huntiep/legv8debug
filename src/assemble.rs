use Register;
use bytecode::{Instruction, Opcode};
use tokenizer::Token;

use std::collections::HashMap;
use std::slice::Iter;

pub fn assemble(tokens: Vec<Token>) -> (Vec<Opcode>, Vec<usize>) {
    let mut code = Vec::new();
    let mut line_map = Vec::new();
    let mut labels = HashMap::new();
    let mut jumps = Vec::new();

    let mut i =0;
    let mut line_number = 1;
    let mut tokens = tokens.iter();
    while let Some(t) = tokens.next() {
        match t {
            Token::Label(line, s) => {
                handle_line_map(&mut line_map, i, &mut line_number, *line);
                if labels.contains_key(s) {
                    panic!("label {} define more than once on line {}", s, line);
                }
                labels.insert(s, i);
            }
            Token::Instruction(line, instr) => {
                use bytecode::Instruction::*;
                code.push(match instr {
                    Prnt => if let Some(Token::Register(_, _)) = tokens.next() {
                        Opcode::Prnt()
                    } else {
                        panic!("Expected register on line {}", line);
                    },
                    Prnl => Opcode::Prnl(),
                    Dump => Opcode::Dump(),
                    Stur | Ldur => handle_d(*instr, &mut tokens, *line),
                    Cbz | Cbnz => handle_cb(*instr, &mut tokens, &labels, &mut jumps, i, *line),
                    // NOTE B.cond instructions are encoded differently but they are written the
                    // same as B-form instructions.
                    B | Bl | Beq | Bgt | Bge | Blt | Ble => handle_b(*instr, &mut tokens, &labels, &mut jumps, i, *line),
                    Addi | Addis | Andis | Eori | Orri | Subi | Subis => handle_i(*instr, &mut tokens, *line),
                    Add | Adds | And | Ands | Eor | Orr | Sub | Subs | Mul => handle_r(*instr, &mut tokens, *line),
                    Lsl | Lsr => handle_shift(*instr, &mut tokens, *line),
                    Br => if let Some(Token::Register(_, r)) = tokens.next() {
                        Opcode::Br(*r)
                    } else {
                        panic!("Invalid instruction on line {}", line);
                    },
                    _ => panic!("This instruction is unimplemented: {:?}", instr),
                });
                handle_line_map(&mut line_map, i, &mut line_number, *line);
                i += 1;
            }
            _ => panic!("Expected label or instruction on line {}", t.line()),
        }
    }

    for (l, pos) in jumps {
        // NOTE that these must be forward jumps as they would otherwise already have been handled.
        if let Some(i) = labels.get(&l) {
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

fn handle_line_map(map: &mut Vec<usize>, i: usize, line_number: &mut usize, l: usize) {
    for _ in *line_number..=l {
        map.push(i);
    }
    *line_number = l + 1;
}

fn read_imm(tokens: &mut Iter<Token>, line_number: usize) -> u16 {
    if let Some(Token::Immediate(_, imm)) = tokens.next() {
        *imm
    } else {
        panic!("Expected immediate on line {}", line_number);
    }
}

fn handle_d(instr: Instruction, tokens: &mut Iter<Token>, line_number: usize) -> Opcode {
    let rt = read_register(tokens, true, line_number);
    if let Some(Token::LBrace(_)) = tokens.next() {
    } else {
        panic!("Expected `[` on line {}", line_number);
    };

    let rn = read_register(tokens, true, line_number);
    let addr = read_imm(tokens, line_number);
    if let Some(Token::RBrace(_)) = tokens.next() {
    } else {
        panic!("Expected `]` on line {}", line_number);
    };

    match instr {
        Instruction::Stur => Opcode::Stur(rn, rt, addr),
        Instruction::Ldur => Opcode::Ldur(rn, rt, addr),
        _ => unreachable!(),
    }
}

fn handle_b(instr: Instruction, tokens: &mut Iter<Token>, labels: &HashMap<&String, usize>,
            jumps: &mut Vec<(String, usize)>, code_pos: usize, line_number: usize)
    -> Opcode
{
    let addr = handle_label(tokens, labels, jumps, code_pos, line_number);
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

fn read_register(tokens: &mut Iter<Token>, trailing_comma: bool, line_number: usize) -> Register {
    let r = if let Some(Token::Register(_, r)) = tokens.next() {
        r
    } else {
        panic!("Expected register on line {}", line_number);
    };

    if trailing_comma {
        if let Some(Token::Comma(_)) = tokens.next() {
        } else {
            panic!("Expected comma on line {}", line_number);
        }
    }

    *r
}

fn handle_label(tokens: &mut Iter<Token>, labels: &HashMap<&String, usize>,
                jumps: &mut Vec<(String, usize)>, code_pos: usize, line_number: usize)
    -> u32
{
    let label = if let Some(Token::Label(_, l)) = tokens.next() {
        l
    } else {
        panic!("Expected label on line {}", line_number);
    };

    // NOTE if the label is found it must be behind this point.
    if let Some(i) = labels.get(label) {
        -((code_pos as i32) - (*i as i32)) as u32
    } else {
        jumps.push((label.to_string(), code_pos));
        0
    }
}

fn handle_cb(instr: Instruction, tokens: &mut Iter<Token>, labels: &HashMap<&String, usize>,
            jumps: &mut Vec<(String, usize)>, code_pos: usize, line_number: usize)
    -> Opcode
{
    let rt = read_register(tokens, true, line_number);
    let addr = handle_label(tokens, labels, jumps, code_pos, line_number);
    match instr {
        Instruction::Cbz => Opcode::Cbz(rt, addr),
        Instruction::Cbnz => Opcode::Cbnz(rt, addr),
        _ => unreachable!(),
    }
}

fn handle_i(instr: Instruction, tokens: &mut Iter<Token>, line_number: usize) -> Opcode {
    let rd = read_register(tokens, true, line_number);
    let rn = read_register(tokens, true, line_number);
    let imm = read_imm(tokens, line_number);

    match instr {
        Instruction::Addi => Opcode::Addi(rn, rd, imm),
        Instruction::Subi => Opcode::Subi(rn, rd, imm),
        _ => unreachable!(),
    }
}

fn handle_shift(instr: Instruction, tokens: &mut Iter<Token>, line_number: usize) -> Opcode {
    let rd = read_register(tokens, true, line_number);
    let rn = read_register(tokens, true, line_number);
    let imm = read_imm(tokens, line_number);

    match instr {
        Instruction::Lsl => Opcode::Lsl(rn, rd, imm as u32),
        Instruction::Lsr => Opcode::Lsr(rn, rd, imm as u32),
        _ => unreachable!(),
    }
}

fn handle_r(instr: Instruction, tokens: &mut Iter<Token>, line_number: usize) -> Opcode {
    let rd = read_register(tokens, true, line_number);
    let rn = read_register(tokens, true, line_number);
    let rm = read_register(tokens, false, line_number);

    match instr {
        Instruction::Add => Opcode::Add(rm, rn, rd),
        Instruction::Sub => Opcode::Sub(rm, rn, rd),
        Instruction::Subs => Opcode::Subs(rm, rn, rd),
        _ => unreachable!(),
    }
}
