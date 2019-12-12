// Compiler for the Nand2Tetris Jack Programming Language
// Author: Leo Robinovitch

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::fs;
use std::path::PathBuf;
use std::io::BufReader;
use std::io::prelude::*;
use std::env;
use std::collections::HashMap;

use regex::Regex;
#[macro_use]
extern crate lazy_static;


// *****************************************
//     FILE PARSER
// *****************************************
#[derive(Debug)]
struct FileParser<'a> {
    // fns:
    // - get_filepaths
    // - get_file_contents (args: path)
    // - get_output_filepath (args: path)
    // - get_writeable_file (args: path)
    // - get_path_string (args: file)
    // - remove_comments (args: line, is_comment)
    // - tokenize_for_jack (args: file_contents)
    input_path: String,
    input_extension: &'a str,
    output_extension: &'a str,
}

impl<'a> FileParser<'a> {
    fn from_user_args(input_extension: &'a str,
        output_extension: &'a str)-> FileParser<'a> {
        let args: Vec<String> = env::args().collect();
        if args.len() < 2 {
            println!("\nMissing required argument");
            println!("Usage: cargo run FILENAME\n");
            panic!();
        };
        let input_path = args[1].clone();
        FileParser {
            input_path,
            input_extension,
            output_extension,
        }
    }

    fn get_filepaths(&self) -> Vec<PathBuf> {
        match PathBuf::from(&self.input_path).extension() {
            Some(ext) => {
                if ext == self.input_extension {
                    return vec![PathBuf::from(&self.input_path)];
                } else {
                    panic!(format!("Invalid non-.{} input found",
                        self.input_extension));
                }
            }
            _ => {
                let paths = fs::read_dir(&self.input_path)
                    .expect("Failed to read directory");
    
                let mut ext_paths: Vec<PathBuf> = Vec::new();
                for direntry in paths {
                    let path = direntry.unwrap().path();
                    if path.extension().is_some() && path.extension().unwrap() == self.input_extension {
                        ext_paths.push(path);
                    }
                }
                ext_paths
            }
        }
    }

    fn get_file_contents(&self, path: &PathBuf) -> String {
        assert_eq!(path.extension().unwrap(), self.input_extension);

        let file = fs::File::open(path).expect("Failed to open file");
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents)
            .expect("Error reading to string");
        contents
    }

    fn get_output_filepath(&self, path: &PathBuf) -> PathBuf {
        let mut output_path = path.clone();
        let mut output_file_name = path.file_name().unwrap()
            .to_str().unwrap().to_string();
        let end_idx = output_file_name.len() - self.input_extension.len() - 1;
        output_file_name = output_file_name[..end_idx].to_string();
        output_path.set_file_name(output_file_name);
        output_path.set_extension(self.output_extension);
        output_path
    }

    fn get_writeable_file(&self, path: &PathBuf) -> fs::File {
        fs::File::create(&path).unwrap()
    }

    fn get_path_string(&self, path: &PathBuf) -> String {
        path.as_path().to_str().unwrap().to_string()
    }

    fn remove_comments(&'a self, line: &'a str, is_comment: bool) -> (&'a str, bool) {
        let mut mod_line = line;
        let mut mod_comment = is_comment;

        // check if line begins multi-line comment '/**'
        let idx_start_ml: i32 = match mod_line.find("/*") {
            Some(idx) => idx as i32,
            _ => -1
        };
        // not a comment and "/**" found: keep up to the start of the multiline comment
        if !mod_comment && idx_start_ml != -1 {
            // check for end of multi-line comment, '*/'
            let idx_end_ml: i32 = match mod_line.find("*/") {
                Some(idx) => idx as i32,
                _ => -1
            };
            mod_line = &mod_line[..idx_start_ml as usize];
            // '*/' found
            if idx_end_ml != -1 {
                mod_comment = false;
            } else {
                mod_comment = true;
            }
        // no "/**" and continues comment from a previous line:
        } else if mod_comment {
            // check for end of multi-line comment, '*/'
            let idx_end_ml: i32 = match line.find("*/") {
                Some(idx) => idx as i32,
                _ => -1
            };
            // if "*/" found: take rest of line past "*/"
            if idx_end_ml != -1 {
                mod_line = &mod_line[idx_end_ml as usize..];
                mod_comment = false;
            // no "*/" found, is comment: kill line
            } else {
                mod_line = "";
            }
        }

        // check for end of multi-line comment, '*/'
        let idx_end_ml: i32 = match mod_line.find("*/") {
            Some(idx) => idx as i32,
            _ => -1
        };
        // '*/' found
        if idx_end_ml != -1 {
            mod_line = &mod_line[idx_end_ml as usize + 2..];
            mod_comment = false;
        }

        // find the index where // comments begin on the line
        let idx_comment = match mod_line.find("//") {
            Some(idx) => idx,
            _ => mod_line.len()
        };

        (&mod_line[0..idx_comment].trim(), mod_comment)
    }

    fn tokenize_for_jack(&self, file_contents: String) -> Vec<String> {
        fn add_tokens(line: &str, tokens: &mut Vec<String>) {
            let mut rest = line;
            while rest != "" {
                let (token, idx) = find_next(rest);
                rest = &rest[idx..].trim();
                tokens.push(token.to_string());
            }
        }

        fn find_next(line: &str) -> (&str, usize) {
            lazy_static! { // lazy_static ensures compilation only happens once
                static ref RE : Regex = Regex::new(
                        r####"(?x)
                        # keyword
                        ^(\bclass\b|\bconstructor\b|\bfunction\b|\bmethod\b|\bfield\b
                        |\bstatic\b|\bvar\b|\bint\b|\bchar\b|\bboolean\b|\bvoid\b|\btrue\b|\bfalse\b
                        |\bnull\b|\bthis\b|\blet\b|\bdo\b|\bif\b|\belse\b|\bwhile\b|\breturn\b
        
                        # symbol
                        |[{]|[}]|[(]|[)]|\[|\]|[.]|
                        [,]|[;]|[+]|[-]|[*]|[/]|[&]|
                        [|]|[<]|[>]|[=]|[~]
        
                        # integerConstant
                        |\d{1,5}
        
                        # StringConstant
                        |"[^"\n]+"
        
                        # identifier
                        |[a-zA-Z0-9_:]+)
                        "####
                    ).unwrap();
            };
        
            let capture = match RE.find(line){
                Some(cap) => cap,
                _ => panic!("No token found")
            };
            let token = &line[capture.start()..capture.end()];
            (token, capture.end())
        }

        let mut is_comment = false;
        let mut tokens: Vec<String> = Vec::new();
        for line in file_contents.lines() {
            let (clean_line, comment) = self.remove_comments(line, is_comment);
            is_comment = comment;
            if clean_line == "" { continue };
            add_tokens(clean_line, &mut tokens);
        }
        tokens
    }
}


// *****************************************
//     JACK TOKENIZER
// *****************************************
#[derive(PartialEq, Debug, Clone, Copy)]
enum TokenKind {
    KEYWORD,
    SYMBOL,
    IDENTIFIER,
    INT_CONST,
    STRING_CONST,
}

#[derive(Debug)]
struct JackTokenizer<'a> {
    // fns:
    // - has_more_tokens -- returns bool
    // - advance -- advances index if has_more_tokens
    // - curr_token -- return current token
    // - next_token -- return next token
    // - token_kind -- returns TokenKind of current token
    // - is_keyword
    // - keyword -- returns keyword of current token (TokenKind::KEYWORD only)
    // - is_symbol
    // - symbol -- returns symbol of current token (TokenKind::SYMBOL only)
    // - is_identifier
    // - is_int_val
    // - int_val -- returns integer value of current token (TokenKind::INT_CONST only)
    // - is_string_val
    // - string_val -- returns string value of current token (TokenKind::STRING_CONST only)
    // - is_builtin_type
    // - is_builtin_op
    // - is_builtin_unary_op
    // - math_command -- returns the MathCommand of a given op
    // - unary_math_command -- returns the UnaryMathCommand of a given op
    // - extract_dot_subroutine -- returns the subroutine name for things like (className|varName).subroutineName

    // lifetime means a JackTokenizer instance can't
    // outlive any token string references in tokens
    tokens: Vec<String>,
    index: usize,
    valid_keywords: Vec<&'a str>,
    valid_symbols: Vec<&'a str>,
    builtin_types: Vec<&'a str>,
    builtin_ops: Vec<&'a str>,
    builtin_unary_ops: Vec<&'a str>,
}

impl<'a> Default for JackTokenizer<'a> {
    fn default() -> JackTokenizer<'a> {
        JackTokenizer {
            tokens: Vec::new(),
            index: 0,
            valid_keywords: vec!["class", "constructor", "function", "method", "field",
            "static", "var", "int", "char", "boolean", "void", "true", "false",
            "null", "this", "let", "do", "if", "else", "while", "return"],
            valid_symbols: vec!["{", "}", "(", ")", "[", "]", ".",
            ",", ";", "+", "-", "*", "/", "&", "|", "<", ">", "=", "~"],
            builtin_types: vec!["int", "boolean", "char"],
            builtin_ops: vec!["+", "-", "*", "/", "&", "|", "<", ">", "="],
            builtin_unary_ops: vec!["~", "-"],
        }
    }
}

impl<'a> JackTokenizer<'a> {
    fn has_more_tokens(&self) -> bool {
        self.index < self.tokens.len() - 1
    }

    fn advance(&mut self) {
        if self.has_more_tokens() {
            self.index += 1;
        } else {
            panic!("Out of tokens -- unfinished code?");
        }
    }

    fn curr_token(&self) -> &str {
        &self.tokens[self.index]
    }

    fn next_token(&self) -> &str {
        &self.tokens[self.index + 1]
    }

    fn token_kind(&self) -> TokenKind {
        let token = self.curr_token();
        if self.valid_keywords.contains(&token) {
            TokenKind::KEYWORD
        } else if self.valid_symbols.contains(&token) {
            TokenKind::SYMBOL
        } else if token.chars().nth(0).unwrap() == '"'
                && token.chars().nth(token.len()-1).unwrap() == '"' {
            TokenKind::STRING_CONST
        } else if token.parse::<u32>().is_ok() {
            if token.parse::<u32>().unwrap() > 32767 {
                panic!("Integer larger than max val of 2^15-1=32767");
            } else {
                TokenKind::INT_CONST
            }
        } else {
            TokenKind::IDENTIFIER
        }
    }

    fn is_keyword(&self) -> bool {
        self.token_kind() == TokenKind::KEYWORD
    }

    fn keyword(&self) -> &str {
        assert!(self.is_keyword());
        self.curr_token()
    }

    fn is_symbol(&self) -> bool {
        self.token_kind() == TokenKind::SYMBOL
    }

    fn symbol(&self) -> &str {
        assert!(self.is_symbol());
        self.curr_token()
    }

    fn is_identifier(&self) -> bool {
        self.token_kind() == TokenKind::IDENTIFIER
    }

    fn is_int_val(&self) -> bool {
        self.token_kind() == TokenKind::INT_CONST
    }

    fn int_val(&self) -> u32 {
        assert!(self.is_int_val());
        self.curr_token().parse::<u32>().unwrap()
    }

    fn is_string_val(&self) -> bool {
        self.token_kind() == TokenKind::STRING_CONST
    }

    fn string_val(&self) -> &str {
        assert!(self.is_string_val());
        let token = self.curr_token();
        &token[1..token.len()-1]
    }

    fn is_builtin_type(&self) -> bool {
        self.builtin_types.contains(&self.curr_token())
    }

    fn is_builtin_op(&self) -> bool {
        self.builtin_ops.contains(&self.curr_token())
    }

    fn is_builtin_unary_op(&self) -> bool {
        self.builtin_unary_ops.contains(&self.curr_token())
    }

    fn math_command(&self) -> MathCommand {
        match self.curr_token() {
            "+" => MathCommand::ADD,
            "-" => MathCommand::SUB,
            "=" => MathCommand::EQUAL,
            ">" => MathCommand::GT,
            "<" => MathCommand::LT,
            "&" => MathCommand::AND,
            "|" => MathCommand::OR,
            "*" => MathCommand::MULT,
            "/" => MathCommand::DIV,
            _ => panic!("Invalid symbol for math command")
        }
    }

    fn unary_math_command(&self) -> UnaryMathCommand {
        match self.curr_token() {
            "-" => UnaryMathCommand::NEG,
            "~" => UnaryMathCommand::NOT,
            _ => panic!("Invalid symbol for unary math command")
        }
    }

    fn kind_to_segment(&self, kind: Kind) -> Segment {
        match kind {
            Kind::STATIC => Segment::STATIC,
            Kind::ARGUMENT => Segment::ARG,
            Kind::FIELD => Segment::THIS,
            Kind::VAR => Segment::LOCAL,
        }
    }

    fn extract_dot_subroutine(&self) -> String {
        self.tokens[self.index..self.index+3].join("")
    }
}


// *****************************************
//     SYMBOL TABLE
// *****************************************
#[derive(PartialEq, Debug, Clone, Copy)]
enum Kind {
    FIELD,
    STATIC,
    ARGUMENT,
    VAR,
}

#[derive(Debug)]
struct Identifier<> {
    symbol_type: String,
    symbol_kind: Kind,
    id: u32,
}

#[derive(Debug)]
struct SymbolTable<> {
    // fns:
    // - start_subroutine (reset subroutine scope) X
    // - define (args: name, type, kind) -- assigns scope and index X
    // - var_count (args: kind) -- returns number of vars of kind defined X
    // - kind_of (args: name) -- returns kind of var
    // - type_of (args: name) -- returns type of var X
    // - index_of (args: name) -- returns index of var X
    // - contains (args: name) -- returns bool
    SubrScope: HashMap<String, Identifier>,
    ClassScope: HashMap<String, Identifier>,
}

impl SymbolTable {
    fn start_subroutine(&mut self) {
        self.SubrScope.retain(|k, _| k == "");
    }

    fn var_count(&self, symbol_kind: Kind) -> u32 {
        let mut count_kind = 0;
        for (_, v) in self.SubrScope.iter() {
            if v.symbol_kind == symbol_kind {
                count_kind += 1;
            }
        }
        for (_, v) in self.ClassScope.iter() {
            if v.symbol_kind == symbol_kind {
                count_kind += 1;
            }
        }
        count_kind
    }

    fn define(&mut self, name: &str, symbol_type: &str, symbol_kind: Kind) {
        if name == "" { panic!("Invalid name in symbol table"); }
        match symbol_kind {
            Kind::STATIC | Kind::FIELD => {
                if !self.ClassScope.contains_key(name) {
                    let id = self.var_count(symbol_kind);
                    self.ClassScope.insert(name.to_string(),
                        Identifier {
                            symbol_type: symbol_type.to_string(),
                            symbol_kind,
                            id 
                        });
                }
            }
            Kind::ARGUMENT | Kind::VAR => {
                if !self.SubrScope.contains_key(name) {
                    let id = self.var_count(symbol_kind);
                    self.SubrScope.insert(name.to_string(),
                        Identifier {
                            symbol_type: symbol_type.to_string(),
                            symbol_kind,
                            id 
                        });
                }
            }
        }
    }

    fn kind_of(&self, name: &str) -> Kind {
        if self.SubrScope.contains_key(name) {
            self.SubrScope.get(name).unwrap().symbol_kind
        } else if self.ClassScope.contains_key(name) {
            self.ClassScope.get(name).unwrap().symbol_kind
        } else {
            panic!("Tried to get kind of something not in symbol table");
        }
    }

    fn type_of(&self, name: &str) -> String {
        if self.SubrScope.contains_key(name) {
            self.SubrScope.get(name).unwrap().symbol_type.to_string()
        } else if self.ClassScope.contains_key(name) {
            self.ClassScope.get(name).unwrap().symbol_type.to_string()
        } else {
            panic!("Tried to get type of something not in symbol table!");
        }
    }

    fn index_of(&self, name: &str) -> u32 {
        if self.SubrScope.contains_key(name) {
            self.SubrScope.get(name).unwrap().id
        } else if self.ClassScope.contains_key(name) {
            self.ClassScope.get(name).unwrap().id
        } else {
            panic!("Tried to get id of something not in symbol table!");
        }
    }

    fn contains(&self, name: &str) -> bool {
        self.SubrScope.contains_key(name) || self.ClassScope.contains_key(name)
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
    EQUAL,
    GT,
    LT,
    AND,
    OR,
    MULT,
    DIV,
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum UnaryMathCommand {
    NEG,
    NOT,
}

#[derive(Debug)]
struct VmWriter<'a> {
    // fns:
    // - write_push (args: segment, index)
    // - write_pop (args: segment, index)
    // - write_arithmetic (args: MathCommand)
    // - write_label (args: label)
    // - write_goto (args: label)
    // - write_if_goto (args: label)
    // - write_call (args: name, n_args)
    // - write_function (args: name, n_locals)
    // - write_return
    // - write_string
    output_file: &'a fs::File,
}

impl<'a> VmWriter<'a> {
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
        self.output_file.write_all(format!("{}\n", line).as_bytes())
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
        self.output_file.write_all(format!("{}\n", line).as_bytes())
            .expect("Failed to write line to file");
    }

    fn write_arithmetic(&mut self, command: MathCommand) {
        let line = match command {
            MathCommand::ADD => { "add" },
            MathCommand::SUB => { "sub" },
            MathCommand::EQUAL => { "eq" },
            MathCommand::GT => { "gt" },
            MathCommand::LT => { "lt" },
            MathCommand::AND => { "and" },
            MathCommand::OR => { "or" },
            MathCommand::MULT => { "call Math.multiply 2" },
            MathCommand::DIV => { "call Math.divide 2" },
        };
        self.output_file.write_all(format!("{}\n", line).as_bytes())
            .expect("Failed to write line to file");
    }

    fn write_unary_arithmetic(&mut self, command: UnaryMathCommand) {
        let line = match command {
            UnaryMathCommand::NEG => { "neg" },
            UnaryMathCommand::NOT => { "not" },
        };
        self.output_file.write_all(format!("{}\n", line).as_bytes())
            .expect("Failed to write line to file");
    }

    fn write_label(&mut self, label: &str) {
        let line = format!("label {}", label);
        self.output_file.write_all(format!("{}\n", line).as_bytes())
            .expect("Failed to write line to file");
    }

    fn write_goto(&mut self, label: &str) {
        let line = format!("goto {}", label);
        self.output_file.write_all(format!("{}\n", line).as_bytes())
            .expect("Failed to write line to file");
    }

    fn write_if_goto(&mut self, label: &str) {
        let line = format!("if-goto {}", label);
        self.output_file.write_all(format!("{}\n", line).as_bytes())
            .expect("Failed to write line to file");
    }

    fn write_call(&mut self, name: &str, n_args: u8) {
        let line = format!("call {} {}", name, n_args.to_string());
        self.output_file.write_all(format!("{}\n", line).as_bytes())
            .expect("Failed to write line to file");
    }

    fn write_function(&mut self, name: &str, n_locals: u8) {
        let line = format!("function {} {}", name, n_locals.to_string());
        self.output_file.write_all(format!("{}\n", line).as_bytes())
            .expect("Failed to write line to file");
    }

    fn write_return(&mut self) {
        self.output_file.write_all("return\n".as_bytes())
            .expect("Failed to write line to file");
    }

    fn write_string(&mut self, string: &str) {
        self.write_push(Segment::CONST, string.chars().count() as u32);
        self.write_call("String.new", 1);
        let mut char_decimal;
        for c in string.chars() {
            char_decimal = c as u32; // convert to unicode
            self.write_push(Segment::CONST, char_decimal);
            self.write_call("String.appendChar", 2);
        }
    }
}


// *****************************************
//     COMPILATION ENGINE
// *****************************************
#[derive(Debug)]
struct CompilationEngine<'a> {
    // fns:
    // - write_line (temp)
    // - check_nonvoid_type (temp)
    // - compile_class
    // - compile_class_var_dec
    // - compile_subroutine
    // - compile_parameterlist
    // - compile_var_dec
    // - compile_statements
    // - compile_do
    // - compile_let
    // - compile_while
    // - compile_return
    // - compile_if
    // - compile_expression
    // - compile_term
    // - compile_subroutine_call
    // - compile_expression_list
    
    tokenizer: JackTokenizer<'a>,
    symbol_table: SymbolTable,
    vm_writer: VmWriter<'a>,
    class_name: String,
    label_count: u32,
}

impl<'a> CompilationEngine<'a> {

    fn check_nonvoid_type(&mut self) {
        if !self.tokenizer.is_builtin_type() && !self.tokenizer.is_identifier() {
            panic!("Invalid type in subroutine declaration");
        }
    }

    fn compile_class(&mut self) {
        // 'class'
        assert_eq!(self.tokenizer.keyword(), "class");
        self.tokenizer.advance();

        // className
        self.class_name = self.tokenizer.curr_token().to_string();
        self.tokenizer.advance();

        // '{'
        assert_eq!(self.tokenizer.symbol(), "{");
        self.tokenizer.advance();

        // classVarDec
        self.compile_class_var_dec();

        // subroutine
        self.compile_subroutine();

        // '}'
        assert_eq!(self.tokenizer.symbol(), "}");
    }

    fn compile_class_var_dec(&mut self) {
        while vec!["static", "field"].contains(&self.tokenizer.curr_token()) {
            // ('static' | 'field')
            let mut var_kind = Kind::FIELD;
            if self.tokenizer.keyword() == "static" {
                var_kind = Kind::STATIC
            }
            self.tokenizer.advance();

            // type
            self.check_nonvoid_type();
            let symbol_type = self.tokenizer.curr_token().to_string();
            self.tokenizer.advance();

            // varName
            let name = self.tokenizer.curr_token().to_string();
            self.symbol_table.define(&name, &symbol_type, var_kind);
            self.tokenizer.advance();

            // (',' varName)*
            while self.tokenizer.curr_token() == "," {
                // ','
                assert_eq!(self.tokenizer.symbol(), ",");
                self.tokenizer.advance();

                // varName
                let name = self.tokenizer.curr_token().to_string();
                self.symbol_table.define(&name, &symbol_type, var_kind);
                self.tokenizer.advance();
            }
            // ';'
            assert_eq!(self.tokenizer.symbol(), ";");
            self.tokenizer.advance();
        }
    }

    fn compile_subroutine(&mut self) {
        while vec!["constructor", "function", "method"].contains(&self.tokenizer.curr_token()) {
            // clear previous subroutine scope from symbol table
            self.symbol_table.start_subroutine();

            // ('constructor' | 'function' | 'method')
            let subroutine_kind = self.tokenizer.curr_token().to_string();
            self.tokenizer.advance();

            // ('void' | type)
            if self.tokenizer.curr_token() == "void" {
            } else {
                self.check_nonvoid_type();
            }
            self.tokenizer.advance();

            // subroutineName
            let function_name = format!(
                "{}.{}",
                self.class_name,
                self.tokenizer.curr_token().to_string()
            );
            self.tokenizer.advance();

            // '('
            assert_eq!(self.tokenizer.symbol(), "(");
            self.tokenizer.advance();

            if subroutine_kind == "method" {
                // add current class object 'this' to symbol table (for 'method')
                self.symbol_table.define("this", &self.class_name, Kind::ARGUMENT);
            }

            // parameterList
            self.compile_parameterlist();

            // ')'
            assert_eq!(self.tokenizer.symbol(), ")");
            self.tokenizer.advance();

            // '{'
            assert_eq!(self.tokenizer.symbol(), "{");
            self.tokenizer.advance();
            
            // varDec*
            let num_locals = self.compile_var_dec();

            // vm function subroutineName num_locals
            self.vm_writer.write_function(&function_name, num_locals);

            if subroutine_kind == "constructor" {
                // get number of fields in class
                let num_fields = self.symbol_table.var_count(Kind::FIELD);
                
                // vm push constant num_fields
                self.vm_writer.write_push(Segment::CONST, num_fields);

                // vm call Memory.alloc 1
                self.vm_writer.write_call("Memory.alloc", 1);

                // vm pop pointer 0 (anchor this at base addr)
                self.vm_writer.write_pop(Segment::POINTER, 0);
            } else if subroutine_kind == "method" {
                // vm push argument 0, then pop pointer 0 (anchor this at base addr)
                self.vm_writer.write_push(Segment::ARG, 0);
                self.vm_writer.write_pop(Segment::POINTER, 0);
            }

            // statements
            self.compile_statements();

            // '}'
            assert_eq!(self.tokenizer.symbol(), "}");
            self.tokenizer.advance();
        }
    }

    fn compile_parameterlist(&mut self) {
        if self.tokenizer.curr_token() != ")" {
            // type
            self.check_nonvoid_type();
            let symbol_type = self.tokenizer.curr_token().to_string();
            self.tokenizer.advance();

            // varName
            let name = self.tokenizer.curr_token().to_string();
            self.symbol_table.define(&name, &symbol_type, Kind::ARGUMENT);
            self.tokenizer.advance();

            // (',' type varName)*
            while self.tokenizer.curr_token() == "," {
                // ','
                assert_eq!(self.tokenizer.symbol(), ",");
                self.tokenizer.advance();

                // type
                self.check_nonvoid_type();
                let symbol_type = self.tokenizer.curr_token().to_string();
                self.tokenizer.advance();

                // varName
                let name = self.tokenizer.curr_token().to_string();
                self.symbol_table.define(&name, &symbol_type, Kind::ARGUMENT);
                self.tokenizer.advance();
            }
        }
    }

    fn compile_var_dec(&mut self) -> u8 {
        let mut num_locals = 0;

        while self.tokenizer.curr_token() == "var" {
            num_locals += 1;

            // 'var'
            assert_eq!(self.tokenizer.keyword(), "var");
            self.tokenizer.advance();

            // type
            self.check_nonvoid_type();
            let symbol_type = self.tokenizer.curr_token().to_string();
            self.tokenizer.advance();

            // varName
            let name = self.tokenizer.curr_token().to_string();
            self.symbol_table.define(&name, &symbol_type, Kind::VAR);
            self.tokenizer.advance();

            // (',' varName)*
            while self.tokenizer.curr_token() == "," {
                num_locals += 1;

                // ','
                assert_eq!(self.tokenizer.symbol(), ",");
                self.tokenizer.advance();

                // varName
                let name = self.tokenizer.curr_token().to_string();
                self.symbol_table.define(&name, &symbol_type, Kind::VAR);
                self.tokenizer.advance();
            }

            // ';'
            assert_eq!(self.tokenizer.symbol(), ";");
            self.tokenizer.advance();
        }
        num_locals
    }

    fn compile_statements(&mut self) {
        while self.tokenizer.is_keyword() 
        && vec!["let", "if", "while", "do", "return"].contains(&self.tokenizer.keyword()) {
            match self.tokenizer.keyword() {
                "do" => self.compile_do(),
                "let" => self.compile_let(),
                "while" => self.compile_while(),
                "return" => self.compile_return(),
                "if" => self.compile_if(),
                _ => panic!("Invalid keyword in statement")
            }
        }
    }

    fn compile_do(&mut self) {
        // 'do'
        assert_eq!(self.tokenizer.keyword(), "do");
        self.tokenizer.advance();

        // subroutineCall
        self.compile_subroutine_call();

        // ';'
        assert_eq!(self.tokenizer.symbol(), ";");
        self.tokenizer.advance();

        // get rid of return value
        self.vm_writer.write_pop(Segment::TEMP, 0);
    }

    fn compile_let(&mut self) {
        let mut array_access = false;

        // 'let'
        assert_eq!(self.tokenizer.keyword(), "let");
        self.tokenizer.advance();

        // varName
        let kind = self.symbol_table.kind_of(self.tokenizer.curr_token());
        let index = self.symbol_table.index_of(self.tokenizer.curr_token());
        let segment = self.tokenizer.kind_to_segment(kind);
        self.tokenizer.advance();

        // ('[' expression ']')?
        if self.tokenizer.curr_token() == "[" {
            array_access = true;

            // vm push array
            self.vm_writer.write_push(segment, index);

            // '['
            assert_eq!(self.tokenizer.symbol(), "[");
            self.tokenizer.advance();

            // expression
            self.compile_expression();

            // ']'
            assert_eq!(self.tokenizer.symbol(), "]");
            self.tokenizer.advance();

            // vm add
            self.vm_writer.write_arithmetic(MathCommand::ADD);
        }

        // '='
        assert_eq!(self.tokenizer.symbol(), "=");
        self.tokenizer.advance();

        // expression
        self.compile_expression();

        if array_access {
            // vm pop temp 0 (store right side result in temp 0)
            self.vm_writer.write_pop(Segment::TEMP, 0);
            
            // vm pop pointer 1 (store RAM addr of arr[expr] in to pointer 1)
            self.vm_writer.write_pop(Segment::POINTER, 1);

            // vm push temp 0
            self.vm_writer.write_push(Segment::TEMP, 0);

            // vm pop that 0
            self.vm_writer.write_pop(Segment::THAT, 0);
        } else {
            // vm pop segment index
            self.vm_writer.write_pop(segment, index);
        }

        // ';'
        assert_eq!(self.tokenizer.symbol(), ";");
        self.tokenizer.advance();
    }

    fn compile_while(&mut self) {
        // 'while'
        assert_eq!(self.tokenizer.keyword(), "while");
        self.tokenizer.advance();

        // setup labels
        let first_label = format!("L{}", self.label_count);
        self.label_count += 1;
        let second_label = format!("L{}", self.label_count);
        self.label_count += 1;

        // '('
        assert_eq!(self.tokenizer.symbol(), "(");
        self.tokenizer.advance();

        // vm first label
        self.vm_writer.write_label(&first_label);

        // expression
        self.compile_expression();

        // vm not
        self.vm_writer.write_unary_arithmetic(UnaryMathCommand::NOT);

        // vm if-goto second label
        self.vm_writer.write_if_goto(&second_label);

        // ')'
        assert_eq!(self.tokenizer.symbol(), ")");
        self.tokenizer.advance();

        // '{'
        assert_eq!(self.tokenizer.symbol(), "{");
        self.tokenizer.advance();

        // statements
        self.compile_statements();

        // '}'
        assert_eq!(self.tokenizer.symbol(), "}");
        self.tokenizer.advance();

        // vm goto first label
        self.vm_writer.write_goto(&first_label);

        // vm second label
        self.vm_writer.write_label(&second_label);
    }

    fn compile_return(&mut self) {
        // 'return'
        assert_eq!(self.tokenizer.keyword(), "return");
        self.tokenizer.advance();

        // expression?
        if self.tokenizer.curr_token() != ";" {
            self.compile_expression();
        } else {
            // void function, push dummy value to stack
            self.vm_writer.write_push(Segment::CONST, 0);
        }

        // ';'
        assert_eq!(self.tokenizer.symbol(), ";");
        self.tokenizer.advance();

        // vm return
        self.vm_writer.write_return();
    }

    fn compile_if(&mut self) {

        // 'if'
        assert_eq!(self.tokenizer.keyword(), "if");
        self.tokenizer.advance();

        // setup labels
        let first_label = format!("L{}", self.label_count);
        self.label_count += 1;
        let second_label = format!("L{}", self.label_count);
        self.label_count += 1;

        // '('
        assert_eq!(self.tokenizer.symbol(), "(");
        self.tokenizer.advance();

        // expression
        self.compile_expression();

        // vm not
        self.vm_writer.write_unary_arithmetic(UnaryMathCommand::NOT);

        // ')'
        assert_eq!(self.tokenizer.symbol(), ")");
        self.tokenizer.advance();

        // vm if-goto first label
        self.vm_writer.write_if_goto(&first_label);

        // '{'
        assert_eq!(self.tokenizer.symbol(), "{");
        self.tokenizer.advance();

        // statements
        self.compile_statements();

        // '}'
        assert_eq!(self.tokenizer.symbol(), "}");
        self.tokenizer.advance();

        // vm goto second label
        self.vm_writer.write_goto(&second_label);
        
        // vm first label
        self.vm_writer.write_label(&first_label);

        // ('else' '{' statements '}')?
        if self.tokenizer.curr_token() == "else" {
            // 'else'
            assert_eq!(self.tokenizer.keyword(), "else");
            self.tokenizer.advance();
            
            // '{'
            assert_eq!(self.tokenizer.symbol(), "{");
            self.tokenizer.advance();

            // statements
            self.compile_statements();

            // '}'
            assert_eq!(self.tokenizer.symbol(), "}");
            self.tokenizer.advance();
        }

        // vm second label
        self.vm_writer.write_label(&second_label);
    }

    fn compile_expression(&mut self) {
        // term
        self.compile_term();

        // (op term)*
        while self.tokenizer.is_builtin_op() {
            // op
            let math_command = self.tokenizer.math_command();
            self.tokenizer.advance();

            // term
            self.compile_term();

            // vm op math command
            self.vm_writer.write_arithmetic(math_command);
        }
    }

    fn compile_term(&mut self) {
        match self.tokenizer.token_kind() {
            TokenKind::INT_CONST => {
                // vm push constant
                self.vm_writer.write_push(Segment::CONST, self.tokenizer.int_val());
                self.tokenizer.advance();
            },
            TokenKind::STRING_CONST => {
                self.vm_writer.write_string(self.tokenizer.string_val());
                self.tokenizer.advance();
            },
            TokenKind::KEYWORD => {
                match self.tokenizer.keyword() {
                    "true" => {
                        // vm push -1
                        self.vm_writer.write_push(Segment::CONST, 0);
                        self.vm_writer.write_unary_arithmetic(UnaryMathCommand::NOT);
                    },
                    "false" => {
                        // vm push 0
                        self.vm_writer.write_push(Segment::CONST, 0);
                    },
                    "null" => {
                        // vm push 0
                        self.vm_writer.write_push(Segment::CONST, 0);
                    },
                    "this" => {
                        // vm push pointer 0 (refers to base addr of this)
                        self.vm_writer.write_push(Segment::POINTER, 0);
                    },
                    _ => {
                        panic!("Unhandled keyword!");
                    }
                }
                self.tokenizer.advance();
            },
            TokenKind::IDENTIFIER => {
                match self.tokenizer.next_token() {
                    // array access array[n]
                    // pop base addr + n of array to pointer 1
                    // push that 0 to get array[n]
                    "[" => { // varName '[' expression ']'
                        // vm push segment index (push base addr of array to stack)
                        let kind = self.symbol_table.kind_of(self.tokenizer.curr_token());
                        let index = self.symbol_table.index_of(self.tokenizer.curr_token());
                        let segment = self.tokenizer.kind_to_segment(kind);
                        self.vm_writer.write_push(segment, index);
                        self.tokenizer.advance();

                        // '['
                        assert_eq!(self.tokenizer.symbol(), "[");
                        self.tokenizer.advance();

                        // expression
                        self.compile_expression();

                        // ']'
                        assert_eq!(self.tokenizer.symbol(), "]");
                        self.tokenizer.advance();

                        // vm add to get base addr + n on top of stack
                        self.vm_writer.write_arithmetic(MathCommand::ADD);

                        // vm pop base addr + n to pointer 1
                        self.vm_writer.write_pop(Segment::POINTER, 1);

                        // vm push that 0 to stack
                        self.vm_writer.write_push(Segment::THAT, 0);
                    },
                    // subroutine call
                    "(" | "." => {
                        self.compile_subroutine_call();
                    },
                    // varName
                    _ => { // vm push segment index
                        let kind = self.symbol_table.kind_of(self.tokenizer.curr_token());
                        let index = self.symbol_table.index_of(self.tokenizer.curr_token());
                        let segment = self.tokenizer.kind_to_segment(kind);
                        self.vm_writer.write_push(segment, index);
                        self.tokenizer.advance();
                    }
                }
            },
            TokenKind::SYMBOL => {
                if self.tokenizer.symbol() == "(" {
                    // '('
                    assert_eq!(self.tokenizer.symbol(), "(");
                    self.tokenizer.advance();

                    // expression
                    self.compile_expression();

                    // ')'
                    assert_eq!(self.tokenizer.symbol(), ")");
                    self.tokenizer.advance();

                } else if self.tokenizer.is_builtin_unary_op() {
                    // unaryOp
                    let unary_math_command = self.tokenizer.unary_math_command();
                    self.tokenizer.advance();

                    // term
                    self.compile_term();

                    // vm unary math command
                    self.vm_writer.write_unary_arithmetic(unary_math_command);
                } else {
                    panic!("Invalid symbol in term");
                }
            },
        }
    }

    fn compile_expression_list(&mut self) -> u8 {
        let mut num_args = 0;

        if self.tokenizer.curr_token() != ")" {
            num_args += 1;

            // expression
            self.compile_expression();

            // (',' expression)*
            while self.tokenizer.curr_token() == "," {
                num_args += 1;

                // ','
                assert_eq!(self.tokenizer.symbol(), ",");
                self.tokenizer.advance();

                // expression
                self.compile_expression();
            }
        }
        num_args
    }

    fn compile_subroutine_call(&mut self) {
        // subroutineName '(' expressionList ')'
        if self.tokenizer.next_token() == "(" {
            // subroutineName -- a method of the current class (equivalent to "this.subroutine()")
            let object_name = self.tokenizer.curr_token().to_string();
            let subroutine_name = format!("{}.{}", self.class_name, object_name);
            self.tokenizer.advance();

            // '('
            assert_eq!(self.tokenizer.symbol(), "(");
            self.tokenizer.advance();

            // vm push pointer 0
            self.vm_writer.write_push(Segment::POINTER, 0);

            // expressionList (add 1 arg for this since calling a method)
            let num_args = self.compile_expression_list() + 1;

            // ')'
            assert_eq!(self.tokenizer.symbol(), ")");
            self.tokenizer.advance();

            // vm call subroutine num_args
            self.vm_writer.write_call(&subroutine_name, num_args);
        }
        // (className | varName) '.' subroutineName '(' expressionList ')'
        else if self.tokenizer.next_token() == "." { 
            // (className | varName)
            let object_name = self.tokenizer.curr_token().to_string();
            let mut subroutine_name = self.tokenizer.extract_dot_subroutine();
            self.tokenizer.advance();

            // '.'
            assert_eq!(self.tokenizer.symbol(), ".");
            self.tokenizer.advance();

            // subroutineName
            self.tokenizer.advance();

            // '('
            assert_eq!(self.tokenizer.symbol(), "(");
            self.tokenizer.advance();

            // check if method call based on if object in scope
            if self.symbol_table.contains(&object_name) {
                subroutine_name = subroutine_name.replace(
                    &object_name,
                    &self.symbol_table.type_of(&object_name)
                );
                // vm push object on stack as first arg
                let kind = self.symbol_table.kind_of(&object_name);
                let index = self.symbol_table.index_of(&object_name);
                let segment = self.tokenizer.kind_to_segment(kind);
                self.vm_writer.write_push(segment, index);
            }

            // expressionList
            let mut num_args = self.compile_expression_list();
            if self.symbol_table.contains(&object_name) {
                num_args += 1;
            }

            // ')'
            assert_eq!(self.tokenizer.symbol(), ")");
            self.tokenizer.advance();

            // vm call subroutine num_args
            self.vm_writer.write_call(&subroutine_name, num_args);
        } else {
            panic!("Invalid subroutine call");
        }
    }
}


// *****************************************
//     MAIN
// *****************************************
fn main() {
    let file_parser = FileParser::from_user_args("jack", "vm");
    let input_paths = file_parser.get_filepaths();
    // let mut label_count = 0;
    for input_path in &input_paths {
        // path stuff
        let output_path = &file_parser.get_output_filepath(input_path);
        let input_path_string = file_parser.get_path_string(input_path);
        let output_path_string = file_parser.get_path_string(output_path);
        println!("\nCompiling {}\n       to {}", input_path_string, output_path_string);

        // file i/o
        let output_file = &file_parser.get_writeable_file(output_path);
        let file_contents = file_parser.get_file_contents(input_path);
        let vm_writer = VmWriter{ output_file };

        // tokenize
        let tokens = file_parser.tokenize_for_jack(file_contents);
        let tokenizer = JackTokenizer{ tokens, ..Default::default() };

        // create symbol table
        let symbol_table = SymbolTable{
            ClassScope: HashMap::new(),
            SubrScope: HashMap::new(),
        };

        // create compilation engine and compile
        let mut compiler = CompilationEngine{
            tokenizer, 
            symbol_table,
            vm_writer,
            class_name: "".to_string(),
            label_count: 0,
        };
        compiler.compile_class();
        // label_count = compiler.label_count;
    }
}