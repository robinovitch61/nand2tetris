// Compiler for the Nand2Tetris Jack Programming Language
// Author: Leo Robinovitch


/* TODO:
- [] Refactor to use actual modules
- [] Get subroutine names with dots, e.g. 'a.b()'
- [] Implement expression VM code using provided algorithm
- [] Implement if and while loop VM code using provided VM code
  - [] Must generate unique labels L1, L2, etc. for every one
- [] Implement VM code for objects using this (pointer 0)
- [] Implement VM code for arrays using this (pointer 1)
- [] Implement compilation of constructors
  - NOTE: No VM code for "class" definition or method definition
  (e.g. constructor Point new()), just adds to symbol table
- [] Implement VM code for method calls and void method calls
- [] Implement VM code for array initialization and access
- [] Implement VM code for 'do subroutine' (throwing out return value with pop temp 0!)
*/

#![allow(clippy::cognitive_complexity)]
#![allow(non_snake_case)]
#![allow(clippy::too_many_arguments)]

use std::fs;
use std::path::PathBuf;
use std::io::BufReader;
use std::io::prelude::*;
use std::env;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::collections::HashMap;

use regex::Regex;
#[macro_use]
extern crate lazy_static;


// *****************************************
//     JACK TOKENIZER
// *****************************************
#[derive(PartialEq, Debug, Clone, Copy)]
enum TokenType {
    KEYWORD,
    SYMBOL,
    IDENTIFIER,
    INT_CONST,
    STRING_CONST,
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum KwType {
    CLASS,
    METHOD,
    FUNCTION,
    CONSTRUCTOR,
    INT,
    BOOLEAN,
    CHAR,
    VOID,
    VAR,
    STATIC,
    FIELD,
    LET,
    DO,
    IF,
    ELSE,
    WHILE,
    RETURN,
    TRUE,
    FALSE,
    NULL,
    THIS,
}

#[derive(Debug)]
struct JackTokenizer<'a > {
    // fns:
    // - has_more_tokens -- returns bool
    // - advance -- advances index if has_more_tokens
    // - token_type -- returns TokenType of current token
    // - keyword -- returns KwType of current token
    // - symbol -- returns symbol of current token (TokenType::SYMBOL only)
    // - identifier -- returns identifier of current token (TokenType::IDENTIFIER only)
    // - int_val -- returns integer value of current token (TokenType::INT_CONST only)
    // - string_val -- returns string value of current token (TokenType::STRING_CONST only)

    // lifetime means a JackTokenizer instance can't
    // outlive any token string references in tokens
    tokens: VecDeque<&'a str>,
    index: u32,
}

impl JackTokenizer {

}


// *****************************************
//     SYMBOL TABLE
// *****************************************
#[derive(PartialEq, Debug, Clone, Copy)]
enum Kind {
    NONE,
    FIELD,
    STATIC,
    ARGUMENT,
    VAR,
}

#[derive(Debug)]
struct Identifier<> {
    name: String,
    symbol_type: String,
    symbol_kind: Kind,
    id: u16,
}

#[derive(Debug)]
struct SymbolTable<> {
    // fns:
    // - start_subroutine (reset subroutine scope) X
    // - define (args: name, type, kind) -- assigns scope and index X
    // - var_count (args: kind) -- returns number of vars of kind defined X
    // - kind_of (args: name) -- returns kind of var or NONE X
    // - type_of (args: name) -- returns type of var X
    // - index_of (args: name) -- returns index of var X
    SubrScope: HashMap<String, Identifier>,
    ClassScope: HashMap<String, Identifier>,
}

impl SymbolTable {
    fn start_subroutine(&mut self) {
        self.SubrScope.retain(|k, _| k == "");
    }

    fn var_count(&self, symbol_kind: &Kind) -> u16 {
        let mut count_kind = 0;
        for (_, v) in self.SubrScope.iter() {
            if &v.symbol_kind == symbol_kind {
                count_kind += 1;
            }
        }
        for (_, v) in self.ClassScope.iter() {
            if &v.symbol_kind == symbol_kind {
                count_kind += 1;
            }
        }
        count_kind
    }

    fn define(&mut self, name: String, symbol_type: String, symbol_kind: Kind) {
        if name == "".to_string() { panic!("Invalid name in symbol table"); }
        match symbol_kind {
            Kind::STATIC | Kind::FIELD => {
                if !self.ClassScope.contains_key(&name) {
                    let id = self.var_count(&symbol_kind);
                    self.ClassScope.insert(name.clone(), Identifier { name, symbol_type, symbol_kind, id });
                }
            }
            Kind::ARGUMENT | Kind::VAR => {
                if !self.SubrScope.contains_key(&name) {
                    let id = self.var_count(&symbol_kind);
                    self.SubrScope.insert(name.clone(), Identifier { name, symbol_type, symbol_kind, id });
                }
            }
            Kind::NONE => {
                
            }
        }
    }

    fn kind_of(&self, name: String) -> Kind {
        if self.SubrScope.contains_key(&name) {
            self.SubrScope.get(&name).unwrap().symbol_kind
        } else if self.ClassScope.contains_key(&name) {
            self.ClassScope.get(&name).unwrap().symbol_kind
        } else {
            Kind::NONE
        }
    }

    fn type_of(&self, name: String) -> String {
        if self.SubrScope.contains_key(&name) {
            self.SubrScope.get(&name).unwrap().symbol_type.clone()
        } else if self.ClassScope.contains_key(&name) {
            self.ClassScope.get(&name).unwrap().symbol_type.clone()
        } else {
            panic!("Tried to get type of something not in symbol table!");
        }
    }

    fn index_of(&self, name: String) -> u16 {
        if self.SubrScope.contains_key(&name) {
            self.SubrScope.get(&name).unwrap().id
        } else if self.ClassScope.contains_key(&name) {
            self.ClassScope.get(&name).unwrap().id
        } else {
            panic!("Tried to get id of something not in symbol table!");
        }
    }
}


// *****************************************
//     VM WRITER
// *****************************************
#[derive(PartialEq, Debug, Clone, Copy)]
enum Segment {
    CONST,
    ARG,
    LOCAL,
    STATIC,
    THIS,
    THAT,
    POINTER,
    TEMP,
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum MathCommand {
    ADD,
    SUB,
    NEG,
    Equal,
    GT,
    LT,
    AND,
    OR,
    NOT,
}

#[derive(Debug)]
struct VmWriter<> {
    // fns:
    // - write_push (args: segment, index) X
    // - write_pop (args: segment, index) X
    // - write_arithmetic (args: MathCommand) X
    // - write_label (args: label) X
    // - write_goto (args: label) X
    // - write_if_goto (args: label) X
    // - write_call (args: name, n_args) X
    // - write_function (args: name, n_locals) X
    // - write_return X
    OutputFile: fs::File,
}

impl VmWriter {
    fn write_push(&mut self, segment: Segment, index: u32) {
        let line = match segment {
            Segment::CONST => { format!("push constant {}",  index.to_string()) },
            Segment::ARG => { format!("push argument {}",  index.to_string()) },
            Segment::LOCAL => { format!("push local {}",  index.to_string()) },
            Segment::STATIC => { format!("push static {}",  index.to_string()) },
            Segment::THIS => { format!("push this {}",  index.to_string()) },
            Segment::THAT => { format!("push that {}",  index.to_string()) },
            Segment::POINTER => { format!("push pointer {}",  index.to_string()) },
            Segment::TEMP => { format!("push temp {}",  index.to_string()) },
        };
        self.OutputFile.write_all(format!("{}\n", line).as_bytes())
            .expect("Failed to write line to file");
    }

    fn write_pop(&mut self, segment: Segment, index: u32) {
        let line = match segment {
            Segment::CONST => { format!("pop constant {}",  index.to_string()) },
            Segment::ARG => { format!("pop argument {}",  index.to_string()) },
            Segment::LOCAL => { format!("pop local {}",  index.to_string()) },
            Segment::STATIC => { format!("pop static {}",  index.to_string()) },
            Segment::THIS => { format!("pop this {}",  index.to_string()) },
            Segment::THAT => { format!("pop that {}",  index.to_string()) },
            Segment::POINTER => { format!("pop pointer {}",  index.to_string()) },
            Segment::TEMP => { format!("pop temp {}",  index.to_string()) },
        };
        self.OutputFile.write_all(format!("{}\n", line).as_bytes())
            .expect("Failed to write line to file");
    }

    fn write_arithmetic(&mut self, command: MathCommand) {
        let line = match command {
            MathCommand::ADD => { "add" },
            MathCommand::SUB => { "sub" },
            MathCommand::NEG => { "neg" },
            MathCommand::Equal => { "eq" },
            MathCommand::GT => { "gt" },
            MathCommand::LT => { "lt" },
            MathCommand::AND => { "and" },
            MathCommand::OR => { "or" },
            MathCommand::NOT => { "not" },
        };
        self.OutputFile.write_all(format!("{}\n", line).as_bytes())
            .expect("Failed to write line to file");
    }

    fn write_label(&mut self, label: &str) {
        let line = format!("label {}", label);
        self.OutputFile.write_all(format!("{}\n", line).as_bytes())
            .expect("Failed to write line to file");
    }

    fn write_goto(&mut self, label: &str) {
        let line = format!("goto {}", label);
        self.OutputFile.write_all(format!("{}\n", line).as_bytes())
            .expect("Failed to write line to file");
    }

    fn write_if_goto(&mut self, label: &str) {
        let line = format!("if-goto {}", label);
        self.OutputFile.write_all(format!("{}\n", line).as_bytes())
            .expect("Failed to write line to file");
    }

    fn write_call(&mut self, name: &str, n_args: u8) {
        let line = format!("call {} {}", name, n_args.to_string());
        self.OutputFile.write_all(format!("{}\n", line).as_bytes())
            .expect("Failed to write line to file");
    }

    fn write_function(&mut self, name: &str, n_locals: u8) {
        let line = format!("function {} {}", name, n_locals.to_string());
        self.OutputFile.write_all(format!("{}\n", line).as_bytes())
            .expect("Failed to write line to file");
    }

    fn write_return(&mut self) {
        self.OutputFile.write_all("return\n".as_bytes())
            .expect("Failed to write line to file");
    }
}



fn main() {
    let mut symbol_table = SymbolTable{
        ClassScope: HashMap::new(),
        SubrScope: HashMap::new(),
    };

    symbol_table.define("test".to_string(), "fuck".to_string(), Kind::ARGUMENT);
    println!("{:?}", symbol_table);
}
