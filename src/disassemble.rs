use bytecode::Opcode;

use std::collections::HashMap;

pub fn disassemble(code: Vec<Opcode>) -> Vec<String> {
    let mut out = Vec::new();
    let mut jumps = HashMap::new();
    let mut ln = 0;
    let mut pc = 0;

    for op in code {
        use bytecode::Instruction::*;
        match op.instruction() {
            B | Bl | Beq | Bne | Bhs | Blo | Bmi | Bpl | Bvs | Bvc | Bhi | Bls |
            Bge | Blt | Bgt | Ble | Cbz | Cbnz => {
                let label = get_label(op, &mut pc, &mut ln, &mut jumps);
                out.push(op.print_branch_label(&label));
            }
            _ => out.push(op.to_string()),
        }
        pc += 1;
    }

    let mut i = 0;
    for (j, l) in jumps {
        out.insert((j+i) as usize, format!("{}:", l));
        i += 1;
    }

    out
}

fn get_label(op: Opcode, pc: &mut u32, ln: &mut usize, jumps: &mut HashMap<u32, String>) -> String {
        use bytecode::Instruction::*;
    let addr = match op.instruction() {
        B => op.b_addr(),
        Bl => op.bl_addr(),
        Cbz => op.cbz_addr(),
        Cbnz => op.cbnz_addr(),
        Beq => op.beq_addr(),
        Bne => op.bne_addr(),
        Bhs => op.bhs_addr(),
        Blo => op.blo_addr(),
        Bmi => op.bmi_addr(),
        Bpl => op.bpl_addr(),
        Bvs => op.bvs_addr(),
        Bvc => op.bvc_addr(),
        Bhi => op.bhi_addr(),
        Bls => op.bls_addr(),
        Bgt => op.bgt_addr(),
        Bge => op.bge_addr(),
        Blt => op.blt_addr(),
        Ble => op.ble_addr(),
        _ => unreachable!(),
    };
    let addr = *pc + addr;

    if let Some(l) = jumps.get(&addr) {
        l.clone()
    } else {
        let label = format!("label{}", ln);
        jumps.insert(addr, label.clone());
        *ln += 1;
        label
    }
}
