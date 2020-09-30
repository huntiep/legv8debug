use bytecode::Instruction;
use register::Register;

use std::str::Chars;

pub enum Token {
    Comma(usize),
    LBrace(usize),
    RBrace(usize),
    Immediate(usize, u16),
    Label(usize, String),
    Register(usize, Register),
    Instruction(usize, Instruction),
}

impl Token {
    pub fn line(&self) -> usize {
        match self {
            Token::Comma(l) => *l,
            Token::LBrace(l) => *l,
            Token::RBrace(l) => *l,
            Token::Immediate(l, _) => *l,
            Token::Label(l, _) => *l,
            Token::Register(l, _) => *l,
            Token::Instruction(l, _) => *l,
        }
    }
}

pub struct Tokenizer<'a> {
    line: usize,
    input: Chars<'a>,
    tokens: Vec<Token>,
}

impl<'a> Tokenizer<'a> {
    pub fn tokenize(input: &'a str) -> Vec<Token> {
        let input = input.chars();
        let mut tokenizer = Tokenizer {
            line: 1,
            input: input,
            tokens: Vec::new(),
        };
        tokenizer._tokenize();
        tokenizer.tokens
    }

    fn next(&mut self) -> Option<char> {
        self.input.next()
    }

    fn _tokenize(&mut self) {
        while let Some(c) = self.next() {
            match c {
                '/' => if self.next() == Some('/') {
                    while let Some(c) = self.next() {
                        if c == '\n' {
                            self.line += 1;
                            break;
                        }
                    }
                } else {
                    panic!("Unexpected `/` on line {}", self.line);
                },
                'a' ..= 'z' | 'A' ..= 'Z' | '_' | '.' => self.handle_symbol(c),
                '[' => self.tokens.push(Token::LBrace(self.line)),
                ']' => self.tokens.push(Token::RBrace(self.line)),
                ',' => self.tokens.push(Token::Comma(self.line)),
                '#' => self.handle_immediate(),
                '\n' => self.line += 1,
                _ if c.is_whitespace() => (),
                _ => panic!("Unexpected token on line {}", self.line),
            }
        }
    }

    fn handle_symbol(&mut self, c: char) {
        let mut buf = String::new();
        buf.push(c);
        while let Some(c) = self.next() {
            match c {
                'a' ..= 'z' | 'A' ..= 'Z' | '0' ..= '9' | '_' | '.' => buf.push(c),
                ':' => return self.tokens.push(Token::Label(self.line, buf)),
                '\n' => {
                    if let Some(r) = Register::from_str(&buf) {
                        self.tokens.push(Token::Register(self.line, r));
                    } else if let Some(i) = Instruction::from_str(&buf) {
                        self.tokens.push(Token::Instruction(self.line, i));
                    } else {
                        self.tokens.push(Token::Label(self.line, buf));
                    }
                    self.line += 1;
                    return;
                },
                ',' => {
                    if let Some(r) = Register::from_str(&buf) {
                        self.tokens.push(Token::Register(self.line, r));
                    } else if let Some(i) = Instruction::from_str(&buf) {
                        self.tokens.push(Token::Instruction(self.line, i));
                    } else {
                        self.tokens.push(Token::Label(self.line, buf));
                    }
                    self.tokens.push(Token::Comma(self.line));
                    return;
                }
                _ if c.is_whitespace() => break,
                _ => panic!("Unexpected token on line {}", self.line),
            }
        }

        if let Some(r) = Register::from_str(&buf) {
            self.tokens.push(Token::Register(self.line, r));
        } else if let Some(i) = Instruction::from_str(&buf) {
            self.tokens.push(Token::Instruction(self.line, i));
        } else {
            self.tokens.push(Token::Label(self.line, buf));
        }
    }

    fn handle_immediate(&mut self) {
        let mut buf = String::new();
        while let Some(c) = self.next() {
            match  c {
                '0' ..= '9' => buf.push(c),
                '\n' => if let Ok(i) = buf.parse() {
                    self.tokens.push(Token::Immediate(self.line, i));
                    self.line += 1;
                    return;
                } else {
                    panic!("Invalid format for immediate value on line {}", self.line);
                },
                ']' => if let Ok(i) = buf.parse() {
                    self.tokens.push(Token::Immediate(self.line, i));
                    self.tokens.push(Token::RBrace(self.line));
                    return;
                } else {
                    panic!("Invalid format for immediate value on line {}", self.line);
                },
                _ if c.is_whitespace() => break,
                _ => panic!("Unexpected token on line {}", self.line),
            }
        }

        if let Ok(i) = buf.parse() {
            self.tokens.push(Token::Immediate(self.line, i));
        } else {
            panic!("Invalid format for immediate value on line {}", self.line);
        }
    }
}
