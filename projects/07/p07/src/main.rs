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


/// ***** TO DO
/// Gets command type for VM code line
/// 
/// # Arguments
/// 
/// * `line` - string slice that holds the current line
fn get_command_type(line: &str) -> CommandType {
    // TO DO
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
fn write_push_pop(command_type: CommandType, segment: &str, index: i32) {
    // TO DO
}

#[test]
fn test_write_push_pop() {
    // not implemented
}


fn main () {
    let (file_contents, output_file, in_path, out_path) = parse_args();

    println!("\nTranslated {:?}\n        -> {:?}\n", in_path, out_path);

}