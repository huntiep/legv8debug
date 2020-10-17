extern crate byteorder;
extern crate clap;

mod assemble;
mod disassemble;
mod bytecode;
mod register;
mod tokenizer;
mod vm;

use bytecode::Opcode;
use register::Register;
use vm::VM;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use clap::{App, Arg, SubCommand};

use std::fs::File;
use std::io::{self, Cursor, Read, Write};

fn main() {
    let matches = App::new("legv8debug")
        .subcommand(SubCommand::with_name("assemble")
            .arg(Arg::with_name("little-endian")
                .short("le"))
            .arg(Arg::with_name("LEGv8 Assembly file")
                .required(true)
                .index(1)))
        .subcommand(SubCommand::with_name("disassemble")
            .arg(Arg::with_name("little-endian")
                .short("le"))
            .arg(Arg::with_name("LEGv8 Binary file")
                .required(true)
                .index(1)))
        .subcommand(SubCommand::with_name("debug")
            //.arg(Arg::with_name("binary")
            //    .short("b"))
            .arg(Arg::with_name("LEGv8 Assembly file")
                .required(true)
                .index(1)))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("assemble") {
        assemble(matches.value_of("LEGv8 Assembly file").unwrap(),
                 matches.is_present("little-endian"));
    } else if let Some(matches) = matches.subcommand_matches("disassemble") {
        disassemble(matches.value_of("LEGv8 Binary file").unwrap(),
                    matches.is_present("little-endian"));
    } else if let Some(matches) = matches.subcommand_matches("debug") {
        debug(matches.value_of("LEGv8 Assembly file").unwrap());
    }
}

fn assemble(filename: &str, le: bool) {
    let mut f = File::open(filename).unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();

    let tokens = tokenizer::Tokenizer::tokenize(&buf);
    let (code, _) = assemble::assemble(tokens);
    let mut c = Vec::new();
    for op in code {
        if le {
            c.write_u32::<LittleEndian>(op.0).unwrap();
        } else {
            c.write_u32::<BigEndian>(op.0).unwrap();
        }
    }

    let mut f = File::create(format!("{}.machine", filename)).unwrap();
    f.write_all(&c).unwrap();
}

fn disassemble(filename: &str, le: bool) {
    let mut f = File::open(filename).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    assert!(buf.len() % 4 == 0);
    let len = buf.len();

    let mut rdr = Cursor::new(buf);

    let mut i = 0;
    let mut code = Vec::new();
    while i < len {
        if le {
            code.push(Opcode(rdr.read_u32::<LittleEndian>().unwrap()));
        } else {
            code.push(Opcode(rdr.read_u32::<BigEndian>().unwrap()));
        }
        i += 4;
    }

    let asm = disassemble::disassemble(code);
    for a in asm {
        println!("{}", a);
    }
}

fn debug(filename: &str) {
    let mut f = File::open(filename).unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();

    let tokens = tokenizer::Tokenizer::tokenize(&buf);
    let (code, line_map) = assemble::assemble(tokens);

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
            "r" => {
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
                    vm.run();
                }
            }
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
