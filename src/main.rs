mod assemble;
mod bytecode;

use bytecode::{Instruction, Opcode};

use std::fs::File;
use std::io::{self, Read, Write};
use std::ops::Deref;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <LEGv8 Assembly file", args[0]);
        std::process::exit(1);
    }

    let mut f = File::open(&args[1]).unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();

    let (code, line_map) = assemble::assemble(&buf);

    let mut vm = VM::new();
    vm.load_code(code);
    vm.load_line_map(line_map);

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        match &input[0..1] {
            "q" => break,
            "r" => vm.run(),
            "d" => vm.dump(),
            "s" => {
                let input = &input[1..].trim();
                let i = if input.is_empty() {
                    1
                } else if let Ok(i) = input.parse() {
                    i
                } else {
                    println!("Expected a positive integer");
                    continue;
                };

                for _ in 0..i {
                    vm.step();
                }
            }
            "b" => {
                let input = &input[1..].trim();
                let i = if let Ok(i) = input.parse() {
                    i
                } else {
                    println!("Expected a positive integer");
                    continue;
                };
                vm.add_breakpoint(i);
            }
            "p" => {
                let input = &input[1..].trim();
                if let Some(r) = Register::from_str(input) {
                    vm.print_register(r);
                } else {
                    println!("Invalid register name: {}", input);
                }
            }
            _ => {
                println!("Unknown command");
                continue;
            }
        }
    }
    //vm.run();
    //vm.dump();
}

struct VM {
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
    fn new() -> Self {
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

        // Initialise SP to end of stack
        vm.registers[28] = 64*8;
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

    fn print_register(&self, r: Register) {
        print!("{} 0x", Register::to_str(*r as usize));
        let x = self.get_register(r).to_le_bytes();
        for i in &x {
            print!("{:02x}", i);
        }
        println!(" ({})", self.get_register(r));
    }

    fn dump(&self) {
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

    fn add_breakpoint(&mut self, mut line: usize) {
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
        use Instruction::*;
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
            Addi => self.addi(op),
            Add => self.add(op),
            Sub => self.sub(op),
            Subi => self.subi(op),
            Subs => self.subs(op),
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
            Lsl => self.lsl(op),
            Lsr => self.lsr(op),
            Prnt | Prn | Dump => self.pc += 1,
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

    fn addi(&mut self, op: Opcode) {
        let rd = op.addi_rd();
        let rn = op.addi_rn();
        let imm = op.addi_imm();
        let v = self.get_register(rn) + imm as u64;
        self.assign_register(rd, v);
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

    fn subs(&mut self, op: Opcode) {
        let rd = op.subs_rd();
        let rn = op.subs_rn();
        let rm = op.subs_rm();
        let v = self.get_register(rn) - self.get_register(rm);
        self.assign_register(rd, v);
        self.flags = v;
        self.pc += 1;
    }

    fn cbz(&mut self, op: Opcode) {
        let rt = op.cbz_rt();
        if self.get_register(rt) == 0 {
            self.pc += op.cbz_addr() as usize;
        } else {
            self.pc += 1;
        }
    }

    fn cbnz(&mut self, op: Opcode) {
        let rt = op.cbnz_rt();
        if self.get_register(rt) != 0 {
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
            self.pc += op.beq_addr() as usize;
        } else {
            self.pc += 1;
        }
    }

    fn bgt(&mut self, op: Opcode) {
        if (self.flags as i64) > 0 {
            self.pc += op.beq_addr() as usize;
        } else {
            self.pc += 1;
        }
    }

    fn bge(&mut self, op: Opcode) {
        if (self.flags as i64) >= 0 {
            self.pc += op.beq_addr() as usize;
        } else {
            self.pc += 1;
        }
    }

    fn blt(&mut self, op: Opcode) {
        if (self.flags as i64) < 0 {
            self.pc += op.beq_addr() as usize;
        } else {
            self.pc += 1;
        }
    }

    fn ble(&mut self, op: Opcode) {
        if (self.flags as i64) <= 0 {
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
        assert!(addr % 8 == 0);
        let addr = addr / 8;

        if addr < 0 {
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
        assert!(addr % 8 == 0);
        let addr = addr / 8;

        if addr < 0 {
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

#[derive(Copy, Clone, Debug)]
pub struct Register(u8);

impl Deref for Register {
    type Target = u8;

    fn deref(&self) -> &u8 {
        &self.0
    }
}

impl Register {
    pub fn as_u32(self) -> u32 {
        *self as u32
    }

    pub fn from_str(r: &str) -> Option<Self> {
        Some(Register(match r {
            "X0" => 0,
            "X1" => 1,
            "X2" => 2,
            "X3" => 3,
            "X4" => 4,
            "X5" => 5,
            "X6" => 6,
            "X7" => 7,
            "X8" => 8,
            "X9" => 9,
            "X10" => 10,
            "X11" => 11,
            "X12" => 12,
            "X13" => 13,
            "X14" => 14,
            "X15" => 15,
            "X16" | "IP0" => 16,
            "X17" | "IP1" => 17,
            "X18" => 18,
            "X19" => 19,
            "X20" => 20,
            "X21" => 21,
            "X22" => 22,
            "X23" => 23,
            "X24" => 24,
            "X25" => 25,
            "X26" => 26,
            "X27" => 27,
            "X28" | "SP" => 28,
            "X29" | "FR" => 29,
            "X30" | "LR" => 30,
            "X31" | "XZR" => 31,
            _ => return None,
        }))
    }

    fn to_str(i: usize) -> String {
        let special = match i {
            16 => "(IP0) ",
            17 => "(IP1) ",
            28 => " (SP) ",
            29 => " (FR) ",
            30 => " (LR) ",
            31 => "(XZR) ",
            _ => "      ",
        };
        format!("{}X{}:{}", special, i, if i < 10 { " " } else { "" })
    }
}

use std::fmt;

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            16 => write!(f, "IP0"),
            17 => write!(f, "IP1"),
            28 => write!(f, "SP"),
            29 => write!(f, "FR"),
            30 => write!(f, "LR"),
            31 => write!(f, "XZR"),
            _ => write!(f, "X{}", self.0),
        }
    }
}
