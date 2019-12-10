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
// use std::collections::Vec;
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
                let paths = fs::read_dir(&self.input_path).unwrap();
    
                let mut ext_paths: Vec<PathBuf> = Vec::new();
                for direntry in paths {
                    let path = direntry.unwrap().path();
                    if path.extension().unwrap() == self.input_extension {
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

        // TODO: add removal of "/* a comment */"

        // check if line begins multi-line comment '/**'
        let idx_start_ml: i32 = match mod_line.find("/**") {
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
struct JackTokenizer<'a> {
    // fns:
    // - has_more_tokens -- returns bool
    // - advance -- advances index if has_more_tokens
    // - token_type -- returns TokenType of current token
    // - is_keyword
    // - keyword -- returns keyword of current token (TokenType::KEYWORD only)
    // - is_symbol
    // - symbol -- returns symbol of current token (TokenType::SYMBOL only)
    // - is_identifier
    // - identifier -- returns identifier of current token (TokenType::IDENTIFIER only)
    // - is_int_val
    // - int_val -- returns integer value of current token (TokenType::INT_CONST only)
    // - is_string_val
    // - string_val -- returns string value of current token (TokenType::STRING_CONST only)

    // lifetime means a JackTokenizer instance can't
    // outlive any token string references in tokens
    tokens: Vec<String>,
    index: usize,
    valid_keywords: Vec<&'a str>,
    valid_symbols: Vec<&'a str>,
    builtin_types: Vec<&'a str>,
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
        }
    }
}

impl<'a> JackTokenizer<'a> {
    fn has_more_tokens(&self) -> bool {
        self.index > self.tokens.len()
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn curr_token(&self) -> &str {
        &self.tokens[self.index]
    }

    fn token_type(&self) -> TokenType {
        let token = self.curr_token();
        if self.valid_keywords.contains(&token) {
            TokenType::KEYWORD
        } else if self.valid_symbols.contains(&token) {
            TokenType::SYMBOL
        } else if token.chars().nth(0).unwrap() == '"'
                && token.chars().nth(token.len()-1).unwrap() == '"' {
            TokenType::STRING_CONST
        } else if token.parse::<u32>().is_ok() {
            TokenType::INT_CONST
        } else {
            TokenType::IDENTIFIER
        }
    }

    fn is_keyword(&self) -> bool {
        self.token_type() == TokenType::KEYWORD
    }

    fn keyword(&self) -> &str {
        assert!(self.is_keyword());
        self.curr_token()
    }

    fn is_symbol(&self) -> bool {
        self.token_type() == TokenType::SYMBOL
    }

    fn symbol(&self) -> &str {
        assert!(self.is_symbol());
        self.curr_token()
    }

    fn is_identifier(&self) -> bool {
        self.token_type() == TokenType::IDENTIFIER
    }

    fn identifier(&self) -> &str {
        assert!(self.is_identifier());
        self.curr_token()
    }

    fn is_int_val(&self) -> bool {
        self.token_type() == TokenType::INT_CONST
    }

    fn int_val(&self) -> &str {
        assert!(self.is_int_val());
        self.curr_token()
    }

    fn is_string_val(&self) -> bool {
        self.token_type() == TokenType::STRING_CONST
    }

    fn string_val(&self) -> &str {
        assert!(self.is_string_val());
        let token = self.curr_token();
        &token[1..token.len()-2]
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
    // - kind_of (args: name) -- returns kind of var
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
        }
    }

    fn kind_of(&self, name: String) -> Option::<Kind> {
        if self.SubrScope.contains_key(&name) {
            Some(self.SubrScope.get(&name).unwrap().symbol_kind)
        } else if self.ClassScope.contains_key(&name) {
            Some(self.ClassScope.get(&name).unwrap().symbol_kind)
        } else {
            None
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


// *****************************************
//     COMPILATION ENGINE
// *****************************************
#[derive(Debug)]
struct CompilationEngine<'a> {
    // fns:
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
    // - compile_expression_list
    
    tokenizer: JackTokenizer<'a>,
    symbol_table: SymbolTable,
    output_file: fs::File,
}

impl<'a> CompilationEngine<'a> {

    fn write_line(&mut self, line: &str) {
        self.output_file.write_all(format!("{}\n", line).as_bytes())
            .expect("Failed to write line to file!");
    }

    fn compile_class(&mut self) {
        self.write_line("<class>");

        // 'class'
        assert_eq!(self.tokenizer.keyword(), "class");
        self.write_line(&format!("<keyword> {} </keyword>", self.tokenizer.keyword()));
        self.tokenizer.advance();

        // className
        self.write_line(&format!("<identifier> {} </identifier>", self.tokenizer.identifier()));
        self.tokenizer.advance();

        // '{'
        assert_eq!(self.tokenizer.symbol(), "{");
        self.write_line(&format!("<symbol> {} </symbol>", self.tokenizer.symbol()));
        self.tokenizer.advance();

        // classVarDec
        self.compile_class_var_dec();

        // subroutine
        self.compile_subroutine();

        // '}'
        assert_eq!(self.tokenizer.symbol(), "}");
        self.write_line(&format!("<symbol> {} </symbol>", self.tokenizer.symbol()));
        self.tokenizer.advance();

        self.write_line("</class>");
    }

    fn compile_class_var_dec(&mut self) {
        loop {
            if !self.tokenizer.is_keyword()
            || !vec!["static", "field"].contains(&self.tokenizer.keyword()) {
                return;
            }
            
            // MORE TODO
        }
    }

    fn compile_subroutine(&mut self) {
        loop {
            // ('constructor' | 'function' | 'method')
            if !self.tokenizer.is_keyword()
            || !vec!["constructor", "function", "method"].contains(&self.tokenizer.keyword()) {
                return;
            }
            self.symbol_table.start_subroutine(); // clear subroutine scope
            self.write_line(&format!("<keyword> {} </keyword>", self.tokenizer.keyword()));
            self.tokenizer.advance();

            // ('void' | type)
            if self.tokenizer.is_keyword() && self.tokenizer.keyword() == "void" {
                self.write_line(&format!("<keyword> {} </keyword>", self.tokenizer.keyword()));
            // keyword is builtin type
            } else if self.tokenizer.is_keyword()
                && self.tokenizer.builtin_types.contains(&self.tokenizer.keyword()) {
                self.write_line(&format!("<keyword> {} </keyword>", self.tokenizer.keyword()))
            // identifier assumed to be user defined type
            } else if self.tokenizer.is_identifier() { 
                self.write_line(&format!("<identifier> {} </identifier>", self.tokenizer.identifier()))
            } else {
                panic!("Invalid type in subroutine declaration");
            }
            self.tokenizer.advance();

            // subroutineName
            self.write_line(&format!("<identifier> {} </identifier>", self.tokenizer.identifier()));
            self.tokenizer.advance();

            // '('
            assert_eq!(self.tokenizer.symbol(), "(");
            self.write_line(&format!("<symbol> {} </symbol>", self.tokenizer.symbol()));
            self.tokenizer.advance();

            // parameterList
            self.compile_parameterlist();

            // ')'
            assert_eq!(self.tokenizer.symbol(), ")");
            self.write_line(&format!("<symbol> {} </symbol>", self.tokenizer.symbol()));
            self.tokenizer.advance();

           // '{'
            assert_eq!(self.tokenizer.symbol(), "{");
            self.write_line(&format!("<symbol> {} </symbol>", self.tokenizer.symbol()));
            self.tokenizer.advance();
            
            // varDec*
            self.compile_var_dec();

            // statements
            self.compile_statements();

            // '}'
            assert_eq!(self.tokenizer.symbol(), "}");
            self.write_line(&format!("<symbol> {} </symbol>", self.tokenizer.symbol()));
            self.tokenizer.advance();
        }
    }

    fn compile_parameterlist(&mut self) {

    }

    fn compile_var_dec(&mut self) {

    }

    fn compile_statements(&mut self) {

    }

    fn compile_do(&mut self) {

    }

    fn compile_let(&mut self) {

    }

    fn compile_while(&mut self) {

    }

    fn compile_return(&mut self) {

    }

    fn compile_if(&mut self) {

    }

    fn compile_expression(&mut self) {

    }

    fn compile_term(&mut self) {

    }

    fn compile_expression_list(&mut self) {

    }
}


// *****************************************
//     MAIN
// *****************************************
fn main() {
    let file_parser = FileParser::from_user_args("jack", "vm");
    let input_paths = file_parser.get_filepaths();
    for input_path in &input_paths {
        // path stuff
        let output_path = &file_parser.get_output_filepath(input_path);
        let input_path_string = file_parser.get_path_string(input_path);
        let output_path_string = file_parser.get_path_string(output_path);
        println!("Compiling {} to {}", input_path_string, output_path_string);

        // file i/o
        let output_file = file_parser.get_writeable_file(output_path);
        let file_contents = file_parser.get_file_contents(input_path);

        // tokenize
        let tokens = file_parser.tokenize_for_jack(file_contents);
        let tokenizer = JackTokenizer{ tokens, ..Default::default() };

        // new symbol table
        let mut symbol_table = SymbolTable{
            ClassScope: HashMap::new(),
            SubrScope: HashMap::new(),
        };

        // create compilation engine and compile
        let mut compiler = CompilationEngine{
            tokenizer, symbol_table, output_file
        };
        compiler.compile_class();
    }
}