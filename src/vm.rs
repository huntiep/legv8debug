use bytecode::Opcode;
use register::Register;

pub struct VM {
    registers: [u64; 32],
    flags: u64,
    pc: usize,
    code: Vec<Opcode>,
    stack: Vec<u64>,
    heap: Vec<u64>,
    line_map: Vec<usize>,
    breakpoints: Vec<usize>,
    hit_br: bool,
    steps: usize,
    loads: usize,
    stores: usize,
}

impl VM {
    pub fn new() -> Self {
        let mut vm = VM {
            registers: [0; 32],
            flags: 0,
            pc: 0,
            code: Vec::new(),
            stack: vec![0; 64],
            heap: vec![0; 4096/8],
            line_map: Vec::new(),
            breakpoints: Vec::new(),
            hit_br: false,
            steps: 0,
            loads: 0,
            stores: 0,
        };

        // Initialise SP and FP to end of stack
        vm.registers[28] = 64*8;
        vm.registers[29] = 64*8;
        vm
    }

    pub fn load_code(&mut self, code: Vec<Opcode>) {
        self.code = code;
    }

    pub fn load_line_map(&mut self, line_map: Vec<usize>) {
        self.line_map = line_map;
    }

    fn get_register(&self, r: Register) -> u64 {
        if *r == 31 {
            0
        } else {
            self.registers[*r as usize]
        }
    }

    fn assign_register(&mut self, r: Register, v: u64) {
        self.registers[*r as usize] = v;
    }

    pub fn print_register(&self, r: Register) {
        print!("{} 0x", Register::to_str(*r as usize));
        let x = self.get_register(r).to_le_bytes();
        for i in &x {
            print!("{:02x}", i);
        }
        println!(" ({})", self.get_register(r));
    }

    pub fn dump(&self) {
        println!("Registers:");
        for i in 0..32 {
            self.print_register(Register(i));
        }

        println!("\nStack:\n");
        println!("                         *** HOW TO READ THIS TABLE ***");
        println!("The left-most column is the offset in hexidecimal of the beginning of the line.");
        println!("The next 16 columns are the values of the 16 bytes following the line offset,");
        println!("also in hex.  The final column, between vertical bars, gives the text value of");
        println!("the same 16 bytes; if the value is not printable, or if it is a literal period,");
        println!("it is represented with a period.  The bars are for demarkation; they are not");
        println!("part of the data.  The final line, a single hexidecimal number on the left");
        println!("column, gives the size of the data.\n");

        for i in 0..self.stack.len()/2 {
            print!("{:08x}  ", i*2*8);
            Self::print_little_endian(self.stack[2*i]);
            print!(" ");
            Self::print_little_endian(self.stack[2*i + 1]);
            print!(" |");
            Self::print_little_endian_ascii(self.stack[2*i]);
            Self::print_little_endian_ascii(self.stack[2*i + 1]);
            println!("|");
        }
        println!("{:08x}", self.stack.len()*8);

        println!("\nMain Memory:");

        for i in 0..self.heap.len()/2 {
            print!("{:08x}  ", i*2*8);
            Self::print_little_endian(self.heap[2*i]);
            print!(" ");
            Self::print_little_endian(self.heap[2*i + 1]);
            print!(" |");
            Self::print_little_endian_ascii(self.heap[2*i]);
            Self::print_little_endian_ascii(self.heap[2*i + 1]);
            println!("|");
        }
        println!("{:08x}", self.heap.len()*8);

        println!("\nExtra:");
        println!("Instructions executed: {}", self.steps);
        println!("         Loads issued: {}", self.loads);
        println!("        Stores issued: {}", self.stores);
    }

    fn print_little_endian(x: u64) {
        let x = x.to_le_bytes();
        for i in &x {
            print!("{:02x} ", i);
        }
    }

    fn print_little_endian_ascii(x: u64) {
        let x = x.to_le_bytes();
        for &i in &x {
            if i >= 32 && i <= 126 {
                print!("{}", i as char);
            } else {
                print!(".");
            }
        }
    }

    pub fn add_breakpoint(&mut self, line: usize) {
        if line > self.line_map.len() {
            println!("There are only {} lines in this program", self.line_map.len());
            return;
        }

        self.breakpoints.push(self.line_map[line-1]);
    }

    pub fn run(&mut self) {
        while self.pc < self.code.len() {
            if !self.hit_br && self.breakpoints.contains(&self.pc) {
                println!("Reached breakpoint");
                self.hit_br = true;
                return;
            } else if self.hit_br {
                self.hit_br = false;
            }
            self.step();
        }
        println!("Reached end of program");
    }

    pub fn step(&mut self) {
        use bytecode::Instruction::*;
        if self.pc >= self.code.len() {
            println!("Reached end of program");
            return;
        }

        if !self.hit_br && self.breakpoints.contains(&self.pc) {
            println!("Reached breakpoint");
            self.hit_br = true;
        } else if self.hit_br {
            self.hit_br = false;
        }

        println!("    {}", self.code[self.pc]);

        let op = self.code[self.pc];
        match op.instruction() {
            Addis => self.addis(op),
            Addi => self.addi(op),
            Adds => self.adds(op),
            Add => self.add(op),
            Sub => self.sub(op),
            Subi => self.subi(op),
            Subs => self.subs(op),
            Subis => self.subis(op),
            Cbz => self.cbz(op),
            Cbnz => self.cbnz(op),
            B => self.b(op),
            Beq => self.beq(op),
            Bgt => self.bgt(op),
            Bge => self.bge(op),
            Blt => self.blt(op),
            Ble => self.ble(op),
            Bl => self.bl(op),
            Br => self.br(op),
            Stur => self.stur(op),
            Ldur => self.ldur(op),
            Orr => self.orr(op),
            Orri => self.orri(op),
            Eor => self.eor(op),
            Eori => self.eori(op),
            And => self.and(op),
            Ands => self.ands(op),
            Andi => self.andi(op),
            Andis => self.andis(op),
            Mul => self.mul(op),
            Lsl => self.lsl(op),
            Lsr => self.lsr(op),
            Prnt | Prnl | Dump => self.pc += 1,
            _ => unimplemented!(),
        }

        self.steps += 1;
    }

    fn add(&mut self, op: Opcode) {
        let rd = op.add_rd();
        let rn = op.add_rn();
        let rm = op.add_rm();
        let v = self.get_register(rn) + self.get_register(rm);
        self.assign_register(rd, v);
        self.pc += 1;
    }

    fn adds(&mut self, op: Opcode) {
        let rd = op.add_rd();
        let rn = op.add_rn();
        let rm = op.add_rm();
        let v = self.get_register(rn) + self.get_register(rm);
        self.assign_register(rd, v);
        self.flags = v;
        self.pc += 1;
    }

    fn addi(&mut self, op: Opcode) {
        let rd = op.addi_rd();
        let rn = op.addi_rn();
        let imm = op.addi_imm();
        let v = self.get_register(rn) + imm as u64;
        self.assign_register(rd, v);
        self.pc += 1;
    }

    fn addis(&mut self, op: Opcode) {
        let rd = op.addi_rd();
        let rn = op.addi_rn();
        let imm = op.addi_imm();
        let v = self.get_register(rn) + imm as u64;
        self.assign_register(rd, v);
        self.flags = v;
        self.pc += 1;
    }

    fn sub(&mut self, op: Opcode) {
        let rd = op.sub_rd();
        let rn = op.sub_rn();
        let rm = op.sub_rm();
        let v = self.get_register(rn) - self.get_register(rm);
        self.assign_register(rd, v);
        self.pc += 1;
    }

    fn subi(&mut self, op: Opcode) {
        let rd = op.subi_rd();
        let rn = op.subi_rn();
        let imm = op.subi_imm();
        let v = self.get_register(rn) - imm as u64;
        self.assign_register(rd, v);
        self.pc += 1;
    }

    fn subis(&mut self, op: Opcode) {
        let rd = op.subis_rd();
        let rn = op.subis_rn();
        let imm = op.subis_imm();
        let v = self.get_register(rn) - imm as u64;
        self.assign_register(rd, v);
        self.flags = v;
        self.pc += 1;
    }

    fn subs(&mut self, op: Opcode) {
        let rd = op.subs_rd();
        let rn = op.subs_rn();
        let rm = op.subs_rm();
        let v = self.get_register(rn) - self.get_register(rm);
        self.assign_register(rd, v);
        self.flags = v;
        self.pc += 1;
    }

    fn and(&mut self, op: Opcode) {
        let rd = op.and_rd();
        let rn = op.and_rn();
        let rm = op.and_rm();
        let v = self.get_register(rn) & self.get_register(rm);
        self.assign_register(rd, v);
        self.pc += 1;
    }

    fn ands(&mut self, op: Opcode) {
        let rd = op.ands_rd();
        let rn = op.ands_rn();
        let rm = op.ands_rm();
        let v = self.get_register(rn) & self.get_register(rm);
        self.assign_register(rd, v);
        self.flags = v;
        self.pc += 1;
    }

    fn andi(&mut self, op: Opcode) {
        let rd = op.andi_rd();
        let rn = op.andi_rn();
        let imm = op.andi_imm();
        let v = self.get_register(rn) & imm as u64;
        self.assign_register(rd, v);
        self.pc += 1;
    }

    fn andis(&mut self, op: Opcode) {
        let rd = op.andis_rd();
        let rn = op.andis_rn();
        let imm = op.andis_imm();
        let v = self.get_register(rn) & imm as u64;
        self.assign_register(rd, v);
        self.flags = v;
        self.pc += 1;
    }

    fn orr(&mut self, op: Opcode) {
        let rd = op.orr_rd();
        let rn = op.orr_rn();
        let rm = op.orr_rm();
        let v = self.get_register(rn) | self.get_register(rm);
        self.assign_register(rd, v);
        self.pc += 1;
    }

    fn orri(&mut self, op: Opcode) {
        let rd = op.orri_rd();
        let rn = op.orri_rn();
        let imm = op.orri_imm();
        let v = self.get_register(rn) | imm as u64;
        self.assign_register(rd, v);
        self.pc += 1;
    }

    fn eor(&mut self, op: Opcode) {
        let rd = op.eor_rd();
        let rn = op.eor_rn();
        let rm = op.eor_rm();
        let v = self.get_register(rn) ^ self.get_register(rm);
        self.assign_register(rd, v);
        self.pc += 1;
    }

    fn eori(&mut self, op: Opcode) {
        let rd = op.eori_rd();
        let rn = op.eori_rn();
        let imm = op.eori_imm();
        let v = self.get_register(rn) ^ imm as u64;
        self.assign_register(rd, v);
        self.pc += 1;
    }

    fn mul(&mut self, op: Opcode) {
        let rd = op.mul_rd();
        let rn = op.mul_rn();
        let rm = op.mul_rm();
        let v = self.get_register(rn) * self.get_register(rm);
        self.assign_register(rd, v);
        self.pc += 1;
    }

    fn cbz(&mut self, op: Opcode) {
        let rt = op.cbz_rt();
        if self.get_register(rt) == 0 {
            println!("    Branch taken");
            self.pc += op.cbz_addr() as usize;
        } else {
            self.pc += 1;
        }
    }

    fn cbnz(&mut self, op: Opcode) {
        let rt = op.cbnz_rt();
        if self.get_register(rt) != 0 {
            println!("    Branch taken");
            self.pc += op.cbnz_addr() as usize;
        } else {
            self.pc += 1;
        }
    }

    fn b(&mut self, op: Opcode) {
        let addr = op.b_addr();
        self.pc = (self.pc as u32 + addr) as usize;
    }

    fn beq(&mut self, op: Opcode) {
        if self.flags == 0 {
            println!("    Branch taken");
            self.pc += op.beq_addr() as usize;
        } else {
            self.pc += 1;
        }
    }

    fn bgt(&mut self, op: Opcode) {
        if (self.flags as i64) > 0 {
            println!("    Branch taken");
            self.pc += op.beq_addr() as usize;
        } else {
            self.pc += 1;
        }
    }

    fn bge(&mut self, op: Opcode) {
        if (self.flags as i64) >= 0 {
            println!("    Branch taken");
            self.pc += op.beq_addr() as usize;
        } else {
            self.pc += 1;
        }
    }

    fn blt(&mut self, op: Opcode) {
        if (self.flags as i64) < 0 {
            println!("    Branch taken");
            self.pc += op.beq_addr() as usize;
        } else {
            self.pc += 1;
        }
    }

    fn ble(&mut self, op: Opcode) {
        if (self.flags as i64) <= 0 {
            println!("    Branch taken");
            self.pc += op.beq_addr() as usize;
        } else {
            self.pc += 1;
        }
    }

    fn br(&mut self, op: Opcode) {
        let rt = op.br_rt();
        self.pc = self.get_register(rt) as usize;
    }

    fn bl(&mut self, op: Opcode) {
        self.assign_register(Register(30), (self.pc+1) as u64);
        self.pc = (self.pc as u32 + op.bl_addr()) as usize;
    }

    fn stur(&mut self, op: Opcode) {
        let rn = op.stur_rn();
        let rt = op.stur_rt();

        let addr = (op.stur_addr() as u64 + self.get_register(rn)) as usize;
        if addr % 8 != 0 {
            println!("Addresses must be divisible by 8: {:#x}", addr);
            self.pc = self.code.len();
            return;
        }
        let addr = addr / 8;

        if (addr as isize) < 0 {
            println!("Address {:#x} out of bounds", addr);
            self.pc = self.code.len();
            return;
        }

        let v = self.get_register(rt);

        // handle SP specially
        if *rn == 28 {
            if addr >= self.stack.len() {
                println!("Address {:#x} out of bounds", addr);
                self.pc = self.code.len();
                return;
            }

            self.stack[addr] = v;
        } else {
            if addr >= self.heap.len() {
                println!("Address {:#x} out of bounds", addr);
                self.pc = self.code.len();
                return;
            }

            self.heap[addr] = v;
        }
        self.stores += 1;
        self.pc += 1;
    }

    fn ldur(&mut self, op: Opcode) {
        let rn = op.stur_rn();
        let rt = op.stur_rt();

        let addr = (op.stur_addr() as u64 + self.get_register(rn)) as usize;
        if addr % 8 != 0 {
            println!("Addresses must be divisible by 8: {:#x}", addr);
            self.pc = self.code.len();
            return;
        }
        let addr = addr / 8;

        if (addr as isize) < 0 {
            println!("Address {:#x} out of bounds", addr);
            self.pc = self.code.len();
            return;
        }

        // Handle SP specially
        let v = if *rn == 28 {
            if addr >= self.stack.len() {
                println!("Address {:#x} out of bounds", addr);
                self.pc = self.code.len();
                return;
            }

            self.stack[addr]
        } else {
            if addr >= self.heap.len() {
                println!("Address {:#x} out of bounds", addr);
                self.pc = self.code.len();
                return;
            }

            self.heap[addr]
        };
        self.assign_register(rt, v);
        self.loads += 1;
        self.pc += 1;
    }

    fn lsl(&mut self, op: Opcode) {
        let rd = op.lsl_rd();
        let rn = op.lsl_rn();
        let shamt = op.lsl_shamt();
        let v = self.get_register(rn) << shamt;
        self.assign_register(rd, v);
        self.pc += 1;
    }

    fn lsr(&mut self, op: Opcode) {
        let rd = op.lsr_rd();
        let rn = op.lsr_rn();
        let shamt = op.lsr_shamt();
        // TODO: is this actually supposed to rotate?
        let v = self.get_register(rn) >> shamt;
        self.assign_register(rd, v);
        self.pc += 1;
    }
}
