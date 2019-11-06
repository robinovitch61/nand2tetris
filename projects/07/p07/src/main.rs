// VM Translator for the Nand2Tetris Hack Computer
// Author: Leo Robinovitch

use std::fs::File;
use std::path::Path;
use std::io::BufReader;
use std::io::prelude::*;
use std::env;
use std::collections::HashMap;

use regex::Regex;
#[macro_use]
extern crate lazy_static;

// create an enum type called CommandType
// that implements the Debug, etc. traits
//     (printing with {:?} tells type)
#[derive(PartialEq, Eq, Debug)]
enum CommandType {
    C_ARITHMETIC,
    C_PUSH,
    C_POP,
    C_LABEL,
    C_GOTO,
    C_IF,
    C_FUNCTION,
    C_RETURN,
    C_CALL
}

/// Returns a String of the file contents at path
/// Note: path is referenced from the root directory of the project
/// 
/// # Arguments
/// 
/// * `path` - A std::path::Path that contains the input file path
/// * `extension` - required extension for file
fn get_file_contents(path: &Path, extension: &str) -> String { // takes reference to str, "read only"

    assert_eq!(path.extension().unwrap(), extension);

    let file = File::open(path).expect("Failed to open file");
    let mut buf_reader = BufReader::new(file); // buffer reader for file
    let mut contents = String::new(); // mutable string variable for contents
    buf_reader.read_to_string(&mut contents) // mutable reference to contents
        .expect("Error reading to string");
    contents // returns contents
}

#[test]
fn test_get_file_contents() {
    // no test implemented
}


/// Create and return writable file based on path
/// 
/// # Arguments
/// 
/// * `path`
fn create_file(path: &Path) -> File {
    File::create(&path).unwrap()
}

#[test]
fn test_create_file () {
    // not implemented
}


/// Write line to file
/// 
/// # Arguments
/// 
/// * `file` - writable file
/// * `line` - line to write to file
fn write_to_file(mut file: &File, line: String) {
    // Write the `LOREM_IPSUM` string to `file`, returns `io::Result<()>`
    file.write_all(format!("{}\n", line).as_bytes()).expect("Failed to write line to file!");
}


/// Parse command line arguments and return input file
/// contents and output file to write to
fn parse_args() -> (String, File, String, String) {
    // get user args
    let args: Vec<String> = env::args().collect();

    // check user args
    if args.len() < 2 {
        println!("Missing required argument!");
        println!("Usage: cargo run FILENAME");
        panic!();
    };

    let in_path_str = &args[1];
    let in_path = Path::new(in_path_str);
    let out_path_str = args[1].replace(".vm", ".asm");
    let out_path = Path::new(&out_path_str);

    let file_contents = get_file_contents(in_path, "vm");
    let output_file = create_file(out_path);

    (file_contents, output_file, in_path_str.to_string(), out_path_str.to_string())
}

#[test]
fn test_parse_args() {
    // not implemented
}


/// Returns a cleaned string slice after removing comments and white space
/// 
/// # Arguments
/// 
/// * `line` - the current line
fn remove_comments(line: &str) -> &str {
    
    // find the index where comments begin on the line
    let idx_comment = match line.find("//") {
        Some(idx) => idx,
        _ => line.len()
    };

    // return a reference to the reduced str with no start/end whitespace
    // note that memory contents are the same, just pointer and/or len changed
    &line[0..idx_comment].trim()
}

#[test]
fn test_stripped_line() {
    assert_eq!("", remove_comments(""));
    assert_eq!("", remove_comments("    "));
    assert_eq!("", remove_comments("//   "));
    assert_eq!("nand2tetris", remove_comments("nand2tetris   // is so cool"));
    assert_eq!("nand2tetris is so cool", remove_comments("nand2tetris is so cool // eh?"));
}


/// Checks if VM code line is one of the 9 possible
/// C_ARITHMETIC commands
/// 
/// # Arguments
/// 
/// * 
fn is_arithmetic(line: &str) -> bool {
    lazy_static! { // lazy_static ensures compilation only happens once
        static ref RE : Regex = Regex::new(
                r"^(add|sub|neg|eq|gt|lt|and|or|not)$"
            ).unwrap();
    };

    RE.captures(line).is_some()
}

#[test]
fn test_is_arithmetic() {
    assert_eq!(true, is_arithmetic("add"));
    assert_eq!(true, is_arithmetic("sub"));
    assert_eq!(true, is_arithmetic("neg"));
    assert_eq!(true, is_arithmetic("eq"));
    assert_eq!(true, is_arithmetic("gt"));
    assert_eq!(true, is_arithmetic("lt"));
    assert_eq!(true, is_arithmetic("and"));
    assert_eq!(true, is_arithmetic("or"));
    assert_eq!(true, is_arithmetic("not"));
    assert_eq!(false, is_arithmetic("butt"));
    assert_eq!(false, is_arithmetic("add sub"));
}

/// ***** TO DO
/// Gets command type for VM code line
/// 
/// # Arguments
/// 
/// * `line` - string slice that holds the current line
fn get_command_type(line: &str) -> CommandType {
    if is_arithmetic(line) {
        CommandType::C_ARITHMETIC
    } else if &line[0..4] == "push" {
        CommandType::C_PUSH
    } else if &line[0..3] == "pop" {
        CommandType::C_POP
    } else { // this is temporary
        panic!();
    }
}

#[test]
fn test_get_command_type() {
    assert_eq!(CommandType::C_PUSH, get_command_type("push constant 1"));
    // ADD MORE TESTS
}


/// ***** TO DO
/// Writes assembly code for push and pop commands
/// 
/// # Arguments
/// 
/// * `command_type` (C_PUSH or C_POP)
/// * `segment` (constant, local, argument, etc.)
/// * `index`
fn write_push_pop(file: &File, line: &str, command_type: CommandType, segment: &str, index: i32) {
    match segment { // TO DO extend
        "constant" => {
            match command_type {
                CommandType::C_POP => {
                    let asm_code = format!("// {line}\n\
                            @SP\n\
                            AM=M-1\n\
                            D=M\n\
                            @{index}\n\
                            M=D", line=line, index=index);
                    write_to_file(file, asm_code)
                },
                CommandType::C_PUSH => { // correct
                    let asm_code = format!("// {line}\n\
                            @{index}\n\
                            D=A\n\
                            @SP\n\
                            A=M\n\
                            M=D\n\
                            @SP\n\
                            M=M+1", line=line, index=index);
                    write_to_file(file, asm_code)
                },
                _ => panic!("Non C_PUSH or C_POP type found in write_push_pop!")
            }
        },
        "other_stuff" => (),
        _ => ()
    }
}

#[test]
fn test_write_push_pop() {
    // not implemented
}


/// ***** TO DO
/// Writes assembly code for push and pop commands
/// 
/// # Arguments
/// 
/// * `command_type` (C_PUSH or C_POP)
/// * `segment` (constant, local, argument, etc.)
/// * `index`
fn write_arithmetic(file: &File, line: &str, cmp_count: i32) -> i32 {

    let mut new_cmp_count = cmp_count;

    match line {
        "add" => { // correct
            let asm_code = format!("// {line}\n\
                @SP\n\
                AM=M-1\n\
                D=M\n\
                @SP\n\
                A=M-1\n\
                M=M+D", line=line);
            write_to_file(file, asm_code)
        },
        "sub" => {
            let asm_code = format!("// {line}\n\
                @SP\n\
                AM=M-1\n\
                D=M\n\
                @SP\n\
                A=M-1\n\
                M=M-D", line=line);
            write_to_file(file, asm_code)
        },
        "neg" => {
            let asm_code = format!("// {line}\n\
                @SP\n\
                A=M-1\n\
                M=-M", line=line);
            write_to_file(file, asm_code)
        },
        "eq" => {
            let asm_code = format!("// {line}\n\
                @SP\n\
                AM=M-1\n\
                D=M\n\
                @SP\n\
                A=M-1\n\
                D=M-D\n\
                @EQUAL{cmp_count}\n\
                D;JEQ\n\
                @SP\n\
                A=M-1\n\
                M=0\n\
                @CONTINUE{cmp_count}\n\
                0;JMP\n\
                (EQUAL{cmp_count})\n\
                    @SP\n\
                    A=M-1\n\
                    M=-1\n\
                (CONTINUE{cmp_count})", line=line, cmp_count=cmp_count);
            new_cmp_count += 1;
            write_to_file(file, asm_code)
        },
        "gt" => {
            let asm_code = format!("// {line}\n\
                @SP\n\
                AM=M-1\n\
                D=M\n\
                @SP\n\
                A=M-1\n\
                D=M-D\n\
                @GT{cmp_count}\n\
                D;JGT\n\
                @SP\n\
                A=M-1\n\
                M=0\n\
                @CONTINUE{cmp_count}\n\
                0;JMP\n\
                (GT{cmp_count})\n\
                    @SP\n\
                    A=M-1\n\
                    M=-1\n\
                (CONTINUE{cmp_count})", line=line, cmp_count=cmp_count);
            new_cmp_count += 1;
            write_to_file(file, asm_code)
        },
        "lt" => {
            let asm_code = format!("// {line}\n\
                @SP\n\
                AM=M-1\n\
                D=M\n\
                @SP\n\
                A=M-1\n\
                D=M-D\n\
                @LT{cmp_count}\n\
                D;JLT\n\
                @SP\n\
                A=M-1\n\
                M=0\n\
                @CONTINUE{cmp_count}\n\
                0;JMP\n\
                (LT{cmp_count})\n\
                    @SP\n\
                    A=M-1\n\
                    M=-1\n\
                (CONTINUE{cmp_count})", line=line, cmp_count=cmp_count);
            new_cmp_count += 1;
            write_to_file(file, asm_code)
        },
        "and" => {
            let asm_code = format!("// {line}\n\
                @SP\n\
                M=M-1\n\
                A=M\n\
                D=M\n\
                @SP\n\
                A=M-1\n\
                M=D&M", line=line);
            write_to_file(file, asm_code)
        },
        "or" => {
            let asm_code = format!("// {line}\n\
                @SP\n\
                M=M-1\n\
                A=M\n\
                D=M\n\
                @SP\n\
                A=M-1\n\
                M=D|M", line=line);
            write_to_file(file, asm_code)
        },
        "not" => {
            let asm_code = format!("// {line}\n\
                @SP\n\
                A=M-1
                M=!M", line=line);
            write_to_file(file, asm_code)
        },
        _ => {}
    }
    new_cmp_count
}

#[test]
fn test_write_arithmetic() {
    // not implemented
}


/// Get segment and index from push/pop command
/// 
/// # Arguments
/// 
/// * `line` - push/pop command
fn parse_push_pop(line: &str) -> (&str, i32) {
    lazy_static! { // lazy_static ensures compilation only happens once
        static ref RE : Regex = Regex::new(
                r"^(push|pop)\s*(\w*)\s*(\d*)" // TO DO may need refining
            ).unwrap();
    };

    let capture = RE.captures(line)
        .expect("Invalid push/pop command!");

    (capture.get(2).unwrap().as_str(),
    capture.get(3).unwrap().as_str().parse::<i32>().unwrap())
}

// #[test]
// fn test_get_symbol_a_command() {
//     assert_eq!("test", get_symbol_a_command("@test"));
//     assert_eq!("Test_:123$", get_symbol_a_command("@Test_:123$"));
//     let result = std::panic::catch_unwind(|| get_symbol_a_command("@Test_:123$%")); // % is invalid
//     assert!(result.is_err());
//     let result = std::panic::catch_unwind(|| get_symbol_a_command("@test test"));
//     assert!(result.is_err());
//     let result = std::panic::catch_unwind(|| get_symbol_a_command("@1test"));
//     assert!(result.is_err());
// }


/// ********************************
/// ************* MAIN *************
/// ********************************
fn main () {

    let (file_contents, output_file, in_path, out_path) = parse_args();
    println!("{}", file_contents);

    let mut cmp_count = 0;
    for line in file_contents.lines() {
        let clean_line = remove_comments(line);
        if clean_line == "" { continue };
        println!("Clean line: {}", clean_line);

        let command_type = get_command_type(line);
        println!("Command type: {:?}", command_type);

        match command_type {
            CommandType::C_PUSH => {
                let (segment, index) = parse_push_pop(clean_line);
                write_push_pop(&output_file, line, command_type, segment, index);
            },
            CommandType::C_POP => {
                let (segment, index) = parse_push_pop(clean_line);
                write_push_pop(&output_file, line, command_type, segment, index);
            },
            CommandType::C_ARITHMETIC => {
                cmp_count = write_arithmetic(&output_file, line, cmp_count);
            },
            _ => ()
        }

    }

    println!("\nTranslated {:?}\n        -> {:?}\n", in_path, out_path);

}