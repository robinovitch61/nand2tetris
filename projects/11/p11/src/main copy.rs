// Compiler for the Nand2Tetris Jack Programming Language
// Author: Leo Robinovitch


/* TODO:
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

// enum for class var type
#[derive(PartialEq, Eq, Debug)]
enum ClassKind {
    Null,
    Field,
    Static
}

// enum for symbol types
#[derive(PartialEq, Eq, Debug)]
enum SubroutineKind {
    Argument,
    Var
}

// struct for class symbols
#[derive(Debug)]
struct ClassSymbol<> {
    name: String,
    class_type: String,
    class_kind: ClassKind,
    id: u16,
}

// struct for subroutine symbols
#[derive(Debug)]
struct SubroutineSymbol<> {
    name: String,
    subr_type: String,
    subr_kind: SubroutineKind,
    id: u16,
}

/// Returns a String of the file contents at path
/// Note: path is referenced from the root directory of the project
/// 
/// # Arguments
/// 
/// * `path` - A std::path::Path that contains the input file path
/// * `extension` - required extension for file
fn get_file_contents(path: &PathBuf, extension: &str) -> String {
    assert_eq!(path.extension().unwrap(), extension);

    let file = fs::File::open(path).expect("Failed to open file");
    let mut buf_reader = BufReader::new(file); // buffer reader for file
    let mut contents = String::new(); // mutable string variable for contents
    buf_reader.read_to_string(&mut contents) // mutable reference to contents
        .expect("Error reading to string");
    contents // returns contents
}


/// Write line to file
/// 
/// # Arguments
/// 
/// * `file` - writable file
/// * `line` - line to write to file
fn write_to_file(mut file: &fs::File, line: String) {
    // Write the `LOREM_IPSUM` string to `file`, returns `io::Result<()>`
    file.write_all(format!("{}\n", line).as_bytes())
        .expect("Failed to write line to file!");
}


/// Get filepaths of given extension in a directory. If file specified, wraps
/// input filename in vector.
/// 
/// # Arguments
/// 
/// * vector of pathbufs in dir with given extension
fn get_filepaths(dir: &str, extension: &str) -> Vec<PathBuf>{

    match PathBuf::from(dir).extension() {
        Some(ext) => {
            if ext == extension {
                return vec![PathBuf::from(dir)];
            } else {
                panic!(format!("Cannot translate non-.{} file", extension));
            }
        }
        _ => {
            let paths = fs::read_dir(dir).unwrap();

            let mut ext_paths: Vec<PathBuf> = Vec::new();
            for direntry in paths {
                let path = direntry.unwrap().path();
                if path.extension().unwrap() == extension {
                    ext_paths.push(path);
                }
            }
            ext_paths
        }
    }
}


/// Parse command line arguments and return input file
/// contents and output file to write to
fn parse_args() -> (Vec<String>, Vec<String>, Vec<fs::File>, Vec<String>) {
    // get user args
    let args: Vec<String> = env::args().collect();

    // check user args
    if args.len() < 2 {
        println!("\nMissing required argument");
        println!("Usage: cargo run FILENAME\n");
        panic!();
    };

    // .jack file contents
    let extension = "jack";
    let vm_filepaths = get_filepaths(&args[1], extension);
    let mut file_contents: Vec<String> = Vec::new();
    let mut input_files: Vec<String> = Vec::new();
    let mut output_files: Vec<fs::File> = Vec::new();
    let mut out_path_str : Vec<String> = Vec::new();
    for filepath in vm_filepaths {
        // get file contents
        file_contents.push(get_file_contents(&filepath, extension));

        // save filepath string
        input_files.push(filepath.as_path().to_str().unwrap().to_string());

        // generate vm output with modified filename
        let mut vm_filepath = filepath;
        let mut vm_file_name = vm_filepath.file_name().unwrap().to_str().unwrap().to_string();
        vm_file_name = vm_file_name[..(vm_file_name.len() - extension.len() - 1)].to_string();
        // vm_file_name.push_str("_gen");
        vm_filepath.set_file_name(vm_file_name);
        vm_filepath.set_extension("vm");
        output_files.push(fs::File::create(&vm_filepath).unwrap());

        // save output filepath string
        out_path_str.push(vm_filepath.as_path().to_str().unwrap().to_string());
    }

    (file_contents, input_files, output_files, out_path_str)
}


/// Returns a cleaned string slice after removing comments and white space
/// 
/// # Arguments
/// 
/// * `line` - the raw line
fn remove_comments(line: &str, is_comment: bool) -> (&str, bool) {
    
    // remove stuff like /** STUFF */
    // and /** 
    //   STUFF
    //   STUFF */

    let mut mod_line = line;
    let mut mod_comment = is_comment;

    // check if line begins multi-line comment '/**'
    let idx_start_ml: i32 = match mod_line.find("/**") { // TODO: remove e.g. /* STUFF */
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

    // find the index where comments begin on the line
    let idx_comment = match mod_line.find("//") {
        Some(idx) => idx,
        _ => mod_line.len()
    };

    // return a reference to the reduced str with no start/end whitespace
    // note that memory contents are the same, just pointer and/or len changed
    (&mod_line[0..idx_comment].trim(), mod_comment)
}


fn get_kw_set() -> HashSet<String> {
    let mut kw_set = HashSet::new();
    for kw in &["class", "constructor", "function", "method", "field",
    "static", "var", "int", "char", "boolean", "void", "true", "false",
    "null", "this", "let", "do", "if", "else", "while", "return"] {
        kw_set.insert(kw.to_string());
    };
    kw_set
}


/// Finds next token in line. Returns token and rest of line.
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


fn tokenize_line(line: &str, tokens: &mut VecDeque<String>) {
    let mut rest = line;
    while rest != "" {
        let (token, idx) = find_next(rest);
        rest = &rest[idx..].trim();
        tokens.push_back(token.to_string());
    }
}


fn is_identifier(token: &str) -> bool{
    lazy_static! { // lazy_static ensures compilation only happens once
        static ref RE : Regex = Regex::new(
                r####"(?x)
                # identifier
                [a-zA-Z0-9_:]+
                "####
            ).unwrap();
    };

    match RE.find(token){
        Some(_) => true,
        _ => false
    }
}


fn is_type(token: &str) -> bool{
    token == "int" || token == "char" || token == "boolean" || is_identifier(token)
}


fn is_op(token: &str) -> bool{
    token == "+" || token == "-" || token == "*" || token == "/" || token == "&"
    || token == "|" || token == "<" || token == ">" || token == "="
}


fn is_unaryOp(token: &str) -> bool{
    token == "-" || token == "~"
}


fn is_integerConstant(token: &str) -> bool{
    token.parse::<i32>().is_ok()
    && token.parse::<i32>().unwrap() >= 0
    && token.parse::<i32>().unwrap() <= 32767
}


fn check_size(token: &str) {
    if token.parse::<i32>().unwrap() < 0
    || token.parse::<i32>().unwrap() > 32767 {
        panic!("Integer negative or larger than 32767");
    }
}


fn is_stringConstant(token: &str) -> bool{
    lazy_static! { // lazy_static ensures compilation only happens once
        static ref RE : Regex = Regex::new(
                r####"(?x)
                # StringConstant
                "[^"\n]+"
                "####
            ).unwrap();
    };

    match RE.find(token){
        Some(_) => true,
        _ => false
    }
}


fn is_keywordConstant(token: &str) -> bool{
    token == "true" || token == "false" || token == "null" || token == "this"
}


fn get_token(tokens: &mut VecDeque<String>) -> String {
    if tokens.is_empty() {
        return "".to_string();
    }
    tokens.pop_front().unwrap()
}


fn write_symbol(symbol: &str, tokens: &mut VecDeque<String>, file: &fs::File, err: &str) {
    let token = get_token(tokens);
    let err = err.to_string();
    if symbol != "any" && token != symbol {
        panic!(err);
    }
    if token == "<" {
        write_to_file(file, "<symbol> &lt; </symbol>".to_string());
    } else if token == ">" {
        write_to_file(file, "<symbol> &gt; </symbol>".to_string());
    } else if token == "\"" {
        write_to_file(file, "<symbol> &quot; </symbol>".to_string());
    } else if token == "&" {
        write_to_file(file, "<symbol> &amp; </symbol>".to_string());
    } else {
        write_to_file(file, format!("<symbol> {} </symbol>", token));
    }
}


fn write_keyword(kw: &str, tokens: &mut VecDeque<String>, file: &fs::File, err: &str) {
    let token = get_token(tokens);
    let err = err.to_string();
    if kw != "any" && token != kw {
        panic!(err);
    }
    write_to_file(file, format!("<keyword> {} </keyword>", token));
}


fn write_type(tokens: &mut VecDeque<String>, file: &fs::File,
        kw_set: &HashSet<String>, err: &str) {

    // if is_type, write keyword
    // else if is_identifier, write identifier 
    // else raise

    if !is_type(&tokens[0]) {
        let err = err.to_string();
        panic!(err);
    }

    if !kw_set.contains(&tokens[0]) && is_identifier(&tokens[0]) {
        // know it's a custom type, i.e. user defined class / type
        let token = get_token(tokens);
        write_to_file(file, format!("<identifier> {} (UDT) </identifier>", token));
    } else {
        write_keyword("any", tokens, file, err);
    }
}


fn add_subr_symbol(scope: &mut HashMap<String, SubroutineSymbol>, name: &str, subr_type: &str, subr_kind: SubroutineKind) {
    let mut id = 0 as u16;
    if name == "" { panic!("Blank name variable somehow being added to symbol table!"); }
    let key = name.to_string();
    let name = name.to_string();
    let subr_type = subr_type.to_string();
    for (_, v) in scope.iter() {
        if v.subr_kind == subr_kind {
            id += 1;
        }
    }
    let symbol = SubroutineSymbol { name, subr_type, subr_kind, id };
    scope.insert(key, symbol);
    println!("\nSubroutine Scope: {:?}", scope);
}


fn add_class_symbol(scope: &mut HashMap<String, ClassSymbol>, name: &str, class_type: &str, class_kind: ClassKind) {
    let mut id = 0 as u16;
    if name == "" { panic!("Blank name variable somehow being added to symbol table!"); }
    let key = name.to_string();
    let name = name.to_string();
    let class_type = class_type.to_string();
    for (_, v) in scope.iter() {
        if v.class_kind == class_kind {
            id += 1;
        }
    }
    let symbol = ClassSymbol { name, class_type, class_kind, id };
    scope.insert(key, symbol);
    // println!("\nClass Scope: {:?}", scope);
}


fn write_identifier(tokens: &mut VecDeque<String>, file: &fs::File, parent: &str,
    class_var_type: &ClassKind, symbol_type: &str, class_scope: &mut HashMap<String, ClassSymbol>,
    subr_scope: &mut HashMap<String, SubroutineSymbol>, err: &str) {

    let token = get_token(tokens);
    let err = err.to_string();
    if !is_identifier(&token) {
        panic!(err);
    }

    // add to symbol tables if necessary
    if parent == "parameterList" && !subr_scope.contains_key(&token) { // argument, subr
        // scope map, identifier/name, type, kind --> infers id automatically
        add_subr_symbol(subr_scope, &token, symbol_type, SubroutineKind::Argument);
    } else if parent == "varDecs" && !subr_scope.contains_key(&token) { // var, subr
        add_subr_symbol(subr_scope, &token, symbol_type, SubroutineKind::Var);
    } else if parent == "classVarDecs" && !class_scope.contains_key(&token) { // static or field, class
        if class_var_type == &ClassKind::Field {
            add_class_symbol(class_scope, &token, symbol_type, ClassKind::Field);
        } else if class_var_type == &ClassKind::Static {
            add_class_symbol(class_scope, &token, symbol_type, ClassKind::Static);
        }
    }

    // write out
    if parent == "class" {
        write_to_file(file, format!("<identifier> {} (class) </identifier>", token));
    } else if parent == "parameterList" {
        write_to_file(file, format!("<identifier> {} (argument) </identifier>", token));
    } else if parent == "subroutineDecs" || parent == "subroutineCall" {
        write_to_file(file, format!("<identifier> {} (subroutine) </identifier>", token));
    } else if parent == "varDecs" {
        write_to_file(file, format!("<identifier> {} (var/local) </identifier>", token));
    } else if parent == "classVarDecs" && class_var_type != &ClassKind::Null {
        // println!("{}, {}, {:?}", token, parent, class_var_type);
        if class_var_type == &ClassKind::Field {
            write_to_file(file, format!("<identifier> {} (field) </identifier>", token));
            // println!("{}, {}, {:?}", token, parent, class_var_type);
        } else if class_var_type == &ClassKind::Static {
            write_to_file(file, format!("<identifier> {} (static) </identifier>", token));
        }
    } else if parent == "letStatement" || parent == "term"{
        write_to_file(file, format!("<identifier> {} </identifier>", token));
    } else {
        println!("{}, {}, {:?}", token, parent, class_var_type);
        panic!("Invalid parent to identifier.");
    }
}


fn write_vm_code(tokens: &mut VecDeque<String>, file: &fs::File,
        class_scope: &mut HashMap::<String, ClassSymbol>, subr_scope: &mut HashMap::<String, SubroutineSymbol>,
        parent: &str) {
    let kw_set = get_kw_set();
    let token;
    let mut class_var_type = &ClassKind::Null;
    let mut symbol_type = "".to_string();

    while !tokens.is_empty() {

        if parent == "class" {
            // 'class'
            write_keyword("class", tokens, file,
                "File does not begin with 'class' declaration");
            // className
            write_identifier(tokens, file, parent, class_var_type, &symbol_type, class_scope, subr_scope,
                "Missing or invalid identifier for class");
            // '{'
            write_symbol("{", tokens, file,
                "Missing '{' after className identifier");
            // 'classVarDecs'
            write_vm_code(tokens, file, class_scope, subr_scope, "classVarDecs");
            // 'subroutineDecs'
            write_vm_code(tokens, file, class_scope, subr_scope, "subroutineDecs");
            // '}'
            write_symbol("}", tokens, file,
                "Missing closing '}' for class");
            return;
        } else if parent == "classVarDecs" {
            loop {
                if tokens[0] != "static" && tokens[0] != "field" {
                    return;
                }
                write_to_file(file, "<classVarDec>".to_string());
                // ('static' | 'field')
                if &tokens[0] == "static" {
                    class_var_type = &ClassKind::Static;
                } else {
                    class_var_type = &ClassKind::Field;
                }
                write_keyword("any", tokens, file, "");
                // type
                symbol_type = tokens[0].clone();
                write_type(tokens, file, &kw_set,
                    "Missing or invalid type specified for class variable");
                // varName
                write_identifier(tokens, file, parent, class_var_type, &symbol_type, class_scope, subr_scope,
                    "Missing or invalid identifier for class variable");
                // (',' varName)*
                loop {
                    if tokens[0] != "," { break; }
                    // ','
                    write_symbol(",", tokens, file, "");
                    // varName
                    write_identifier(tokens, file, parent, class_var_type, &symbol_type, class_scope, subr_scope,
                        "Missing or invalid identifier or extra ',' in class variable");
                }
                // ';'
                write_symbol(";", tokens, file,
                    "Missing ';' after class variable declaration");   
                write_to_file(file, "</classVarDec>".to_string());
            }
        } else if parent == "subroutineDecs" {
            loop {
                if tokens[0] != "constructor" && tokens[0] != "function" && tokens[0] != "method" {
                    return;
                }
                // clear subroutine scope
                subr_scope.retain(|k, _| k == "");

                write_to_file(file, "<subroutineDec>".to_string());
                // ('constructor' | 'function' | 'method')
                write_keyword("any", tokens, file, "");
                // ('void' | type)
                match tokens[0].as_ref() {
                    "void" => {
                        write_keyword("void", tokens, file,
                            "Expected 'void' return type");
                    },
                    _ => write_type(tokens, file, &kw_set,
                            "Failed to write return type for subroutineDec")
                };
                // subroutineName
                write_identifier(tokens, file, parent, class_var_type, &symbol_type, class_scope, subr_scope,
                    "Missing or invalid identifier in subroutine");
                // '('
                write_symbol("(", tokens, file,
                    "Missing '(' in subroutine");   
                // parameterList
                write_vm_code(tokens, file, class_scope, subr_scope, "parameterList");
                // ')'
                write_symbol(")", tokens, file,
                    "Missing ')' in subroutine");  
                // subroutineBody
                write_vm_code(tokens, file, class_scope, subr_scope, "subroutineBody");
                write_to_file(file, "</subroutineDec>".to_string());
            }
        } else if parent == "parameterList" {
            // leave tokens alone if no parameters
            write_to_file(file, "<parameterList>".to_string());
            if tokens[0] != ")" {
                // type
                symbol_type = tokens[0].clone();
                write_type(tokens, file, &kw_set,
                    "Missing or invalid type specified for parameter");
                // varName
                write_identifier(tokens, file, parent, class_var_type, &symbol_type, class_scope, subr_scope, 
                    "Missing or invalid identifier for parameter");
                // (',' type varName)*
                loop {
                    if tokens[0] != "," { break; }
                    // ','
                    write_symbol(",", tokens, file, "");
                    // type
                    symbol_type = tokens[0].clone();
                    write_type(tokens, file, &kw_set,
                        "Missing or invalid type specified for parameter");
                    // varName
                    write_identifier(tokens, file, parent, class_var_type, &symbol_type, class_scope, subr_scope, 
                        "Missing/invalid identifier or extra ',' in parameterList");
                }
            }
            write_to_file(file, "</parameterList>".to_string());
            return;
        } else if parent == "subroutineBody" {
            write_to_file(file, "<subroutineBody>".to_string());
            // '{'
            write_symbol("{", tokens, file,
                "Missing '{' in subroutine body"); 
            // varDec*
            write_vm_code(tokens, file, class_scope, subr_scope, "varDecs");
            // statements
            write_vm_code(tokens, file, class_scope, subr_scope, "statements");
            // '}'
            write_symbol("}", tokens, file,
                "Missing '}' in subroutine body"); 
            write_to_file(file, "</subroutineBody>".to_string());
            return;
        } else if parent == "varDecs" {
            loop {
                if tokens[0] != "var" { break; }
                write_to_file(file, "<varDec>".to_string());
                // 'var'
                write_keyword("var", tokens, file,
                    "Expected 'var' keyword in varDec");
                // type
                symbol_type = tokens[0].clone();
                write_type(tokens, file, &kw_set,
                    "Missing or invalid type in variable declaration");
                // varName
                write_identifier(tokens, file, parent, class_var_type, &symbol_type, class_scope, subr_scope,
                    "Missing or invalid identifier in variable declaration");
                // (',' varName)*
                loop {
                    if tokens[0] != "," { break; }
                    // ','
                    write_symbol(",", tokens, file, "");
                    // varName
                    write_identifier(tokens, file, parent, class_var_type, &symbol_type, class_scope, subr_scope,
                        "Missing or invalid identifier or extra ',' in variable declaration");
                }
                write_symbol(";", tokens, file,
                    "Missing ';' in variable declaration");
                write_to_file(file, "</varDec>".to_string());
            }
            return;
        } else if parent == "statements" {
            if tokens[0] == "let" || tokens[0] == "if" || tokens[0] == "while"
            || tokens[0] == "do" || tokens[0] == "return" {
                write_to_file(file, "<statements>".to_string());
            }
            loop {
                if tokens[0] != "let" && tokens[0] != "if" && tokens[0] != "while"
                && tokens[0] != "do" && tokens[0] != "return" {
                    break;
                }
                if tokens[0] == "let" {
                    // letStatement
                    write_vm_code(tokens, file, class_scope, subr_scope, "letStatement");
                } else if tokens[0] == "if" {
                    // ifStatement
                    write_vm_code(tokens, file, class_scope, subr_scope, "ifStatement");
                } else if tokens[0] == "while" {
                    // whileStatement
                    write_vm_code(tokens, file, class_scope, subr_scope, "whileStatement");
                } else if tokens[0] == "do" {
                    // doStatement
                    write_vm_code(tokens, file, class_scope, subr_scope, "doStatement");
                } else {
                    // returnStatement
                    write_vm_code(tokens, file, class_scope, subr_scope, "returnStatement");
                }
            }
            write_to_file(file, "</statements>".to_string());
            return;
        } else if parent == "letStatement" {
            write_to_file(file, "<letStatement>".to_string());
            // 'let'
            write_keyword("let", tokens, file,
                "Expected 'let' keyword in letStatement");
            // varName
            write_identifier(tokens, file, parent, class_var_type, &symbol_type, class_scope, subr_scope,
                "Missing or invalid identifier in let statement");
            // ('[' expression ']')?
            if tokens[0] == "[" {
                // '['
                write_symbol("[", tokens, file, "");
                // expression
                write_vm_code(tokens, file, class_scope, subr_scope, "expression");
                // ']'
                write_symbol("]", tokens, file,
                    "Missing ']' for expression in let statement");
            }
            // '='
            write_symbol("=", tokens, file,
                "Missing '=' in let statement");
            // expression
            write_vm_code(tokens, file, class_scope, subr_scope, "expression");
            // ';'
            write_symbol(";", tokens, file,
                "Missing ';' in let statement");
            write_to_file(file, "</letStatement>".to_string());
            return;
        } else if parent == "ifStatement" {
            write_to_file(file, "<ifStatement>".to_string());
            // 'if'
            write_keyword("if", tokens, file,
                "Expected 'if' keyword in ifStatement");
            // '('
            write_symbol("(", tokens, file,
                "Missing '(' for conditional expression in if statement");
            // expression
            write_vm_code(tokens, file, class_scope, subr_scope, "expression");
            // ')'
            write_symbol(")", tokens, file,
                "Missing ')' for conditional expression in if statement");
            // '{'
            write_symbol("{", tokens, file,
                "Missing '{' in if statement");
            // statements
            write_vm_code(tokens, file, class_scope, subr_scope, "statements");
            // '}'
            write_symbol("}", tokens, file,
                "Missing '}' in if statement");
            // ('else' '{' statements '}')?
            if tokens[0] == "else" {
                // 'else'
                write_keyword("else", tokens, file,
                    "Found 'else' keyword but failed to write");
                // '{'
                write_symbol("{", tokens, file,
                    "Missing '{' in else part of if statement");
                // statements
                write_vm_code(tokens, file, class_scope, subr_scope, "statements");
                // '}'
                write_symbol("}", tokens, file,
                    "Missing '}' in else part of if statement");
            }
            write_to_file(file, "</ifStatement>".to_string());
            return;
        } else if parent == "whileStatement" {
            write_to_file(file, "<whileStatement>".to_string());
            // 'while'
            write_keyword("while", tokens, file,
                "Expected 'while' keyword in whileStatement");
            // '('
            write_symbol("(", tokens, file,
                "Missing '(' for conditional expression in while statement");
            // expression
            write_vm_code(tokens, file, class_scope, subr_scope, "expression");
            // ')'
            write_symbol(")", tokens, file,
                "Missing ')' for conditional expression in while statement");
            // '{'
            write_symbol("{", tokens, file,
                "Missing '{' in while statement");
            // statements
            write_vm_code(tokens, file, class_scope, subr_scope, "statements");
            // '}'
            write_symbol("}", tokens, file,
                "Missing '}' in while statement");
            write_to_file(file, "</whileStatement>".to_string());
            return;
        } else if parent == "doStatement" {
            write_to_file(file, "<doStatement>".to_string());
            // 'do'
            write_keyword("do", tokens, file,
                "Expected 'do' keyword in doStatement");
            // subroutineCall
            write_vm_code(tokens, file, class_scope, subr_scope, "subroutineCall");
            // ';'
            write_symbol(";", tokens, file,
                "Missing ';' in do statement");
            write_to_file(file, "</doStatement>".to_string());
            return;
        } else if parent == "returnStatement" {
            write_to_file(file, "<returnStatement>".to_string());
            // 'return'
            write_keyword("return", tokens, file,
                "Expected 'return' keyword in returnStatement");
            // expression?
            if tokens[0] != ";" {
                // expression
                write_vm_code(tokens, file, class_scope, subr_scope, "expression");
            }
            // ';'
            write_symbol(";", tokens, file,
                "Missing ';' in return statement");
            write_to_file(file, "</returnStatement>".to_string());
            return;
        } else if parent == "expression" {
            write_to_file(file, "<expression>".to_string());
            // term
            write_vm_code(tokens, file, class_scope, subr_scope, "term");
            // (op term)*
            loop {
                if !is_op(&tokens[0]) { break; }
                // op
                write_symbol("any", tokens, file, "");
                // term
                write_vm_code(tokens, file, class_scope, subr_scope, "term");
            }
            write_to_file(file, "</expression>".to_string());
            return;
        } else if parent == "term" {
            write_to_file(file, "<term>".to_string());
            if is_integerConstant(&tokens[0]) {
                check_size(&tokens[0]);
                token = get_token(tokens);
                write_to_file(file, format!("<integerConstant> {} </integerConstant>", token));
            } else if is_stringConstant(&tokens[0]) {
                token = get_token(tokens);
                write_to_file(file, format!("<stringConstant> {} </stringConstant>", &token[1..token.len()-1]));
            } else if is_keywordConstant(&tokens[0]) {
                write_keyword("any", tokens, file,
                    "Failed to write keyword");
            } else if is_identifier(&tokens[0]) {
                if &tokens[1] == "[" {
                    // varName
                    write_identifier(tokens, file, parent, class_var_type, &symbol_type, class_scope, subr_scope,
                        "Invalid identifier for varName[...]");
                    // '['
                    write_symbol("[", tokens, file,
                        "Missing '[' in term");
                    // expression
                    write_vm_code(tokens, file, class_scope, subr_scope, "expression");
                    // ']'
                    write_symbol("]", tokens, file,
                        "Missing ']' in term");
                } else if &tokens[1] == "(" || &tokens[1] == "." {
                    // subroutineCall
                    write_vm_code(tokens, file, class_scope, subr_scope, "subroutineCall");
                } else {
                    write_identifier(tokens, file, parent, class_var_type, &symbol_type, class_scope, subr_scope, 
                        "Invalid or missing identifier in term");
                }
            } else if &tokens[0] == "(" {
                // '('
                write_symbol("(", tokens, file, "");
                // expression
                write_vm_code(tokens, file, class_scope, subr_scope, "expression");
                // ')'
                write_symbol(")", tokens, file, "");
            } else if is_unaryOp(&tokens[0]) {
                // unaryOp
                write_symbol("any", tokens, file, "");
                // term
                write_vm_code(tokens, file, class_scope, subr_scope, "term");
            } else {
                panic!("Invalid term");
            }
            write_to_file(file, "</term>".to_string());
            return;
        } else if parent == "subroutineCall" {
            if &tokens[1] == "." {
                // (className | varName)
                write_identifier(tokens, file, parent, class_var_type, &symbol_type, class_scope, subr_scope,
                    "Invalid or missing identifier in subroutineCall");
                // '.'
                write_symbol(".", tokens, file,
                    "Missing '.' in term");
                // subroutineName
                write_identifier(tokens, file, parent, class_var_type, &symbol_type, class_scope, subr_scope,
                    "Invalid or missing identifier in term");
                // '('
                write_symbol("(", tokens, file,
                    "Missing '(' in term");
                // expressionList
                write_vm_code(tokens, file, class_scope, subr_scope, "expressionList");
                // ')'
                write_symbol(")", tokens, file,
                    "Missing ')' in term");
                return;
            } else if &tokens[1] == "(" {
                // subroutineName
                write_identifier(tokens, file, parent, class_var_type, &symbol_type, class_scope, subr_scope,
                    "Invalid or missing identifier in subroutineCall");
                // '('
                write_symbol("(", tokens, file,
                    "Missing '(' in term");
                // expressionList
                write_vm_code(tokens, file, class_scope, subr_scope, "expressionList");
                // ')'
                write_symbol(")", tokens, file,
                    "Missing ')' in term");
                return;
            } else {
                panic!("Invalid subroutineCall");
            }
        } else if parent == "expressionList" {
            write_to_file(file, "<expressionList>".to_string());
            if tokens[0] != ")" {
                // expression
                write_vm_code(tokens, file, class_scope, subr_scope, "expression");
                // (',' expression)*
                loop {
                    if tokens[0] != "," { break; }
                    // ','
                    write_symbol(",", tokens, file, "");
                    // expression
                    write_vm_code(tokens, file, class_scope, subr_scope, "expression");
                }
            }
            write_to_file(file, "</expressionList>".to_string());
            return;
        } else {
            panic!("No condition met");
        }
    }
}

/// ********************************
/// ************* MAIN *************
/// ********************************
fn main () {

    // file stuff
    let (file_contents, in_paths, output_files, out_paths) = parse_args();
    let mut it_file_contents = file_contents.iter();
    let mut it_in_paths = in_paths.iter();
    let mut it_out_files = output_files.iter();
    let mut it_out_paths = out_paths.iter();
    
    while let (Some(contents), Some(in_path), Some(out_file), Some(out_path))
        = (it_file_contents.next(), it_in_paths.next(), it_out_files.next(), it_out_paths.next()) {

        // tokenize
        let mut is_comment = false;
        let mut tokens: VecDeque<String> = VecDeque::new();
        for line in contents.lines() {
            let (clean_line, comment) = remove_comments(line, is_comment);
            is_comment = comment;
            if clean_line == "" { continue };
            tokenize_line(clean_line, &mut tokens);
        }

        // write vm code
        let mut class_scope = HashMap::<String, ClassSymbol>::new();
        let mut subr_scope = HashMap::<String, SubroutineSymbol>::new();
        write_to_file(out_file, "<class>".to_string());
        write_vm_code(&mut tokens, out_file, &mut class_scope, &mut subr_scope, "class");
        write_to_file(out_file, "</class>".to_string());

        println!("Wrote {} to {}\n", in_path, out_path);
    }
}