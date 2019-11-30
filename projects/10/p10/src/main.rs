// Compiler for the Nand2Tetris Jack Programming Language
// Author: Leo Robinovitch

#![allow(clippy::cognitive_complexity)]
#![allow(non_snake_case)]

use std::fs;
use std::path::PathBuf;
use std::io::BufReader;
use std::io::prelude::*;
use std::env;
use std::collections::HashSet;
use std::collections::VecDeque;

use regex::Regex;
#[macro_use]
extern crate lazy_static;


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


// /// Create and return writable file based on path
// /// 
// /// # Arguments
// /// 
// /// * `path`
// fn create_file(path: &Path) -> fs::File {
//     fs::File::create(&path).unwrap()
// }


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

    // .vm file contents
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

        // generate xml output with modified filename
        let mut xml_filepath = filepath;
        let mut xml_file_name = xml_filepath.file_name().unwrap().to_str().unwrap().to_string();
        xml_file_name = xml_file_name[..(xml_file_name.len() - extension.len() - 1)].to_string();
        xml_file_name.push_str("_gen");
        xml_filepath.set_file_name(xml_file_name);
        xml_filepath.set_extension("xml");
        output_files.push(fs::File::create(&xml_filepath).unwrap());

        // save output filepath string
        out_path_str.push(xml_filepath.as_path().to_str().unwrap().to_string());
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
    let idx_start_ml: i32 = match mod_line.find("/**") { // TO DO: /* STUFF */
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


// fn get_symbol_set() -> HashSet<String> {
//     let mut symb_set = HashSet::new();
//     for symb in &['{', '}', '(', ')', '[', ']', '.',
//     ',', ';', '+', '-', '*', '/', '&', '|', '<', '>',
//     '=', '~'] {
//         symb_set.insert(symb.to_string());
//     };
//     symb_set
// }


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

                # integerConstant, 0-32767
                |[0-9]|[1-8][0-9]|9[0-9]|[1-8][0-9]{2}|9[0-8][0-9]|99[0-9]
                |[1-8][0-9]{3}|9[0-8][0-9]{2}|99[0-8][0-9]|999[0-9]
                |[12][0-9]{4}|3[01][0-9]{3}|32[0-6][0-9]{2}
                |327[0-5][0-9]|3276[0-7]

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
    token == "int" || token == "char" || token == "boolean" || is_identifier(&token.to_string())
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
    if symbol != "next" && token != symbol {
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


fn write_type(tokens: &mut VecDeque<String>, file: &fs::File,
        kw_set: &HashSet<String>, err: &str) {
    let token = get_token(tokens);
    let err = err.to_string();
    if !is_type(&token) {
        panic!(err);
    }
    write_keyword(&token, file, kw_set);
}


fn write_identifier(tokens: &mut VecDeque<String>, file: &fs::File, err: &str) {
    let token = get_token(tokens);
    let err = err.to_string();
    if !is_identifier(&token) {
        panic!(err);
    }
    write_to_file(file, format!("<identifier> {} </identifier>", token));
}


fn write_keyword(token: &str, file: &fs::File, kw_set: &HashSet<String>) {
    if kw_set.contains(token) {
        write_to_file(file, format!("<keyword> {} </keyword>", token));
    } else {
        write_to_file(file, format!("<identifier> {} </identifier>", token));
    }
}


fn write_xml_tree(tokens: &mut VecDeque<String>, file: &fs::File, parent: &str) {
    let kw_set = get_kw_set();
    // let symbol_set = get_symbol_set();
    let mut token;

    while !tokens.is_empty() {
        if parent == "class" {
            // 'class'
            token = get_token(tokens);
            if token != "class" {
                panic!("File does not begin with class declaration");
            }
            write_keyword(&token, file, &kw_set);
            // className
            token = get_token(tokens);
            if !is_identifier(&token) {
                panic!("Missing valid className identifier");
            }
            write_to_file(file, format!("<identifier> {} </identifier>", token));
            // '{'
            write_symbol("{", tokens, file,
                "Missing '{' after className identifier");
            // 'classVarDecs'
            write_xml_tree(tokens, file, "classVarDecs");
            // 'subroutineDecs'
            write_xml_tree(tokens, file, "subroutineDecs");
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
                // ('static | 'field')
                token = get_token(tokens);
                write_keyword(&token, file, &kw_set);
                // type
                write_type(tokens, file, &kw_set,
                    "Missing or invalid type specified for class variable");
                // varName
                write_identifier(tokens, file,
                    "Misisng or invalid identifier for class variable");
                // (',' varName)*
                loop {
                    if tokens[0] != "," { break; }
                    // ','
                    write_symbol(",", tokens, file, "");
                    // varName
                    write_identifier(tokens, file,
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
                write_to_file(file, "<subroutineDec>".to_string());
                // ('constructor' | 'function' | 'method')
                token = get_token(tokens);
                write_keyword(&token, file, &kw_set);
                // ('void' | type)
                token = get_token(tokens);
                if token != "void" && !is_type(&token) {
                    panic!("Missing or invalid type (or 'void') specified in subroutine");
                }
                write_keyword(&token, file, &kw_set);
                // subroutineName
                write_identifier(tokens, file,
                    "Missing or invalid identifier in subroutine");
                // '('
                write_symbol("(", tokens, file,
                    "Missing '(' in subroutine");   
                // parameterList
                write_xml_tree(tokens, file, "parameterList");
                // ')'
                write_symbol(")", tokens, file,
                    "Missing ')' in subroutine");  
                // subroutineBody
                write_xml_tree(tokens, file, "subroutineBody");
                write_to_file(file, "</subroutineDec>".to_string());
            }
        } else if parent == "parameterList" {
            // leave tokens alone if no parameters
            write_to_file(file, "<parameterList>".to_string());
            if tokens[0] != ")" {
                // type
                write_type(tokens, file, &kw_set,
                    "Missing or invalid type specified for parameter");
                // varName
                write_identifier(tokens, file, 
                    "Missing or invalid identifier for parameter");
                // (',' type varName)*
                loop {
                    if tokens[0] != "," { break; }
                    // ','
                    write_symbol(",", tokens, file, "");
                    // type
                    write_type(tokens, file, &kw_set,
                        "Missing or invalid type specified for parameter");
                    // varName
                    token = get_token(tokens);
                    if !is_identifier(&token) {
                        panic!("Invalid identifier or extra ',' in parameters");
                    }
                    write_to_file(file, format!("<identifier> {} </identifier>", token));
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
            write_xml_tree(tokens, file, "varDecs");
            // statements
            write_xml_tree(tokens, file, "statements");
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
                token = get_token(tokens);
                write_keyword(&token, file, &kw_set);
                // type
                write_type(tokens, file, &kw_set,
                    "Missing or invalid type in variable declaration");
                // varName
                write_identifier(tokens, file,
                    "Missing or invalid identifier in variable declaration");
                // (',' varName)*
                loop {
                    if tokens[0] != "," { break; }
                    // ','
                    write_symbol(",", tokens, file, "");
                    // varName
                    write_identifier(tokens, file,
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
                    write_xml_tree(tokens, file, "letStatement");
                } else if tokens[0] == "if" {
                    // ifStatement
                    write_xml_tree(tokens, file, "ifStatement");
                } else if tokens[0] == "while" {
                    // whileStatement
                    write_xml_tree(tokens, file, "whileStatement");
                } else if tokens[0] == "do" {
                    // doStatement
                    write_xml_tree(tokens, file, "doStatement");
                } else {
                    // returnStatement
                    write_xml_tree(tokens, file, "returnStatement");
                }
            }
            write_to_file(file, "</statements>".to_string());
            return;
        } else if parent == "letStatement" {
            write_to_file(file, "<letStatement>".to_string());
            // 'let'
            token = get_token(tokens);
            write_keyword(&token, file, &kw_set);
            // varName
            write_identifier(tokens, file,
                "Missing or invalid identifier in let statement");
            // ('[' expression ']')?
            if tokens[0] == "[" {
                // '['
                write_symbol("[", tokens, file, "");
                // expression
                write_xml_tree(tokens, file, "expression");
                // ']'
                write_symbol("]", tokens, file,
                    "Missing ']' for expression in let statement");
            }
            // '='
            write_symbol("=", tokens, file,
                "Missing '=' in let statement");
            // expression
            write_xml_tree(tokens, file, "expression");
            // ';'
            write_symbol(";", tokens, file,
                "Missing ';' in let statement");
            write_to_file(file, "</letStatement>".to_string());
            return;
        } else if parent == "ifStatement" {
            write_to_file(file, "<ifStatement>".to_string());
            // 'if'
            token = get_token(tokens);
            write_keyword(&token, file, &kw_set);
            // '('
            write_symbol("(", tokens, file,
                "Missing '(' for conditional expression in if statement");
            // expression
            write_xml_tree(tokens, file, "expression");
            // ')'
            write_symbol(")", tokens, file,
                "Missing ')' for conditional expression in if statement");
            // '{'
            write_symbol("{", tokens, file,
                "Missing '{' in if statement");
            // statements
            write_xml_tree(tokens, file, "statements");
            // '}'
            write_symbol("}", tokens, file,
                "Missing '}' in if statement");
            // ('else' '{' statements '}')?
            if tokens[0] == "else" {
                // 'else'
                token = get_token(tokens);
                write_keyword(&token, file, &kw_set);
                // '{'
                write_symbol("{", tokens, file,
                    "Missing '{' in else part of if statement");
                // statements
                write_xml_tree(tokens, file, "statements");
                // '}'
                write_symbol("}", tokens, file,
                    "Missing '}' in else part of if statement");
            }
            write_to_file(file, "</ifStatement>".to_string());
            return;
        } else if parent == "whileStatement" {
            write_to_file(file, "<whileStatement>".to_string());
            // 'while'
            token = get_token(tokens);
            write_keyword(&token, file, &kw_set);
            // '('
            write_symbol("(", tokens, file,
                "Missing '(' for conditional expression in while statement");
            // expression
            write_xml_tree(tokens, file, "expression");
            // ')'
            write_symbol(")", tokens, file,
                "Missing ')' for conditional expression in while statement");
            // '{'
            write_symbol("{", tokens, file,
                "Missing '{' in while statement");
            // statements
            write_xml_tree(tokens, file, "statements");
            // '}'
            write_symbol("}", tokens, file,
                "Missing '}' in while statement");
            write_to_file(file, "</whileStatement>".to_string());
            return;
        } else if parent == "doStatement" {
            write_to_file(file, "<doStatement>".to_string());
            // 'do'
            token = get_token(tokens);
            write_keyword(&token, file, &kw_set);
            // subroutineCall
            write_xml_tree(tokens, file, "subroutineCall");
            // ';'
            write_symbol(";", tokens, file,
                "Missing ';' in do statement");
            write_to_file(file, "</doStatement>".to_string());
            return;
        } else if parent == "returnStatement" {
            write_to_file(file, "<returnStatement>".to_string());
            // 'return'
            token = get_token(tokens);
            write_keyword(&token, file, &kw_set);
            // expression?
            if tokens[0] != ";" {
                // expression
                write_xml_tree(tokens, file, "expression");
            }
            // ';'
            write_symbol(";", tokens, file,
                "Missing ';' in return statement");
            write_to_file(file, "</returnStatement>".to_string());
            return;
        } else if parent == "expression" {
            write_to_file(file, "<expression>".to_string());
            // term
            write_xml_tree(tokens, file, "term");
            // (op term)*
            loop {
                if !is_op(&tokens[0]) { break; }
                // op
                write_symbol("next", tokens, file, "");
                // term
                write_xml_tree(tokens, file, "term");
            }
            write_to_file(file, "</expression>".to_string());
            return;
        } else if parent == "term" {
            write_to_file(file, "<term>".to_string());
            if is_integerConstant(&tokens[0]) {
                token = get_token(tokens);
                write_to_file(file, format!("<integerConstant> {} </integerConstant>", token));
            } else if is_stringConstant(&tokens[0]) {
                token = get_token(tokens);
                write_to_file(file, format!("<stringConstant> {} </stringConstant>", &token[1..token.len()-1]));
            } else if is_keywordConstant(&tokens[0]) {
                token = get_token(tokens);
                write_keyword(&token, file, &kw_set);
            } else if is_identifier(&tokens[0]) {
                if &tokens[1] == "[" {
                    // varName
                    write_identifier(tokens, file,
                        "Invalid identifier for varName[...]");
                    // '['
                    write_symbol("[", tokens, file,
                        "Missing '[' in term");
                    // expression
                    write_xml_tree(tokens, file, "expression");
                    // ']'
                    write_symbol("]", tokens, file,
                        "Missing ']' in term");
                } else if &tokens[1] == "(" || &tokens[1] == "." {
                    // subroutineCall
                    write_xml_tree(tokens, file, "subroutineCall");
                } else {
                    write_identifier(tokens, file, 
                        "Invalid or missing identifier in term");
                }
            } else if &tokens[0] == "(" {
                // '('
                write_symbol("(", tokens, file, "");
                // expression
                write_xml_tree(tokens, file, "expression");
                // ')'
            } else if is_unaryOp(&tokens[0]) {
                // unaryOp
                write_symbol("next", tokens, file, "");
            } else {
                panic!("Invalid term");
            }
            write_to_file(file, "</term>".to_string());
            return;
        } else if parent == "subroutineCall" {
            if &tokens[1] == "." {
                // (className | varName)
                token = get_token(tokens);
                write_keyword(&token, file, &kw_set);
                // '.'
                write_symbol(".", tokens, file,
                    "Missing '.' in term");
                // subroutineName
                write_identifier(tokens, file,
                    "Invalid or missing identifier in term");
                // '('
                write_symbol("(", tokens, file,
                    "Missing '(' in term");
                // expressionList
                write_xml_tree(tokens, file, "expressionList");
                // ')'
                write_symbol(")", tokens, file,
                    "Missing ')' in term");
                return;
            } else if &tokens[1] == "(" {
                // subroutineName
                token = get_token(tokens);
                write_keyword(&token, file, &kw_set);
                // '('
                write_symbol("(", tokens, file,
                    "Missing '(' in term");
                // expressionList
                write_xml_tree(tokens, file, "expressionList");
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
                write_xml_tree(tokens, file, "expression");
                // (',' expression)*
                loop {
                    if tokens[0] != "," { break; }
                    // ','
                    write_symbol(",", tokens, file, "");
                    // expression
                    write_xml_tree(tokens, file, "expression");
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

        // xml
        write_to_file(out_file, "<class>".to_string());
        write_xml_tree(&mut tokens, out_file, "class");
        write_to_file(out_file, "</class>".to_string());

        println!("Wrote {} to {}", in_path, out_path);
    }
}