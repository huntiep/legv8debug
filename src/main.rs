mod assemble;
mod bytecode;
mod register;
mod vm;

use register::Register;
use vm::VM;

use std::fs::File;
use std::io::{self, Read, Write};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <LEGv8 Assembly file>", args[0]);
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
}
