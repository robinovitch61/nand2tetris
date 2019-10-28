
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::env;

use regex::Regex;
#[macro_use]
extern crate lazy_static;


/// Constructor/initializer
/// 
/// Returns a String of the file contents at path
/// Note: path is referenced from the root directory of the project
/// 
/// # Arguments
/// 
/// * `path` - A string slice that holds the file path
fn get_file_contents(path: &str) -> String { // takes reference to str, "read only"
    let file = File::open(&path).expect("Failed to open file");
    let mut buf_reader = BufReader::new(file); // buffer reader for file
    let mut contents = String::new(); // mutable string variable for contents
    buf_reader.read_to_string(&mut contents) // mutable reference to contents
        .expect("Error reading to string");
    contents // returns contents
}

#[test]
fn test_get_file_contents() {
    assert_eq!("testing\ntesting2", get_file_contents("test.txt"));
}


/// Returns a parsed line after removing comments and white space
/// 
/// # Arguments
/// 
/// * `line` - A mutable reference that holds the current line
fn parse_line(line: &str) -> &str {
    
    // find the index where comments begin on the line
    let idx_comment = match line.find("//") {
        Some(idx) => idx,
        _ => line.len()
    };

    // return a reference to the reduced str
    // note that memory contents are the same, just pointer and/or len changed
    &line[0..idx_comment].trim()
}

#[test]
fn test_parse_line() {
    assert_eq!("", parse_line(""));
    assert_eq!("", parse_line("    "));
    assert_eq!("", parse_line("//   "));
    assert_eq!("nand2tetris", parse_line("nand2tetris   // is so cool"));
    assert_eq!("nand2tetris is so cool", parse_line("nand2tetris is so cool // eh?"));
}


/// commandType
/// 
/// Returns a string slice that indicates command type
/// 
/// # Arguments
/// 
/// * `line` - A string slice that holds the current line
fn get_command_type(line: &str) -> &str {
    if line.contains('@') {
        return "A_COMMAND";
    } else if line.contains('(') && line.contains(')') {
        return "L_COMMAND";
    }
    "C_COMMAND"
}

#[test]
fn test_get_command_type() {
    assert_eq!("A_COMMAND", get_command_type("@test"));
    assert_eq!("L_COMMAND", get_command_type("(test)"));
    assert_eq!("C_COMMAND", get_command_type("A=1;JEQ"));
    assert_eq!("C_COMMAND", get_command_type("A=M"));
    assert_eq!("C_COMMAND", get_command_type("0;JMP"));
}


fn get_symbol_a_command(command: &str) -> &str {
    lazy_static! { // lazy_static ensures compilation only happens once
        static ref RE : Regex = Regex::new(
                r"^@([^\d][a-zA-Z0-9_\.$:]*)\s*(\S*)"
            ).unwrap();
    };

    let capture = RE.captures(command)
        .expect("Invalid symbol");

    // ensure the A_Command has no garbage after it 
    assert_eq!("", capture.get(2).unwrap().as_str(), "Invalid A Command: {}", command);

    capture.get(1).unwrap().as_str()
}

#[test]
fn test_get_symbol_a_command() {
    assert_eq!("test", get_symbol_a_command("@test"));
    assert_eq!("Test_:123$", get_symbol_a_command("@Test_:123$"));
    let result = std::panic::catch_unwind(|| get_symbol_a_command("@Test_:123$%")); // % is invalid
    assert!(result.is_err());
    let result = std::panic::catch_unwind(|| get_symbol_a_command("@test test"));
    assert!(result.is_err());
    let result = std::panic::catch_unwind(|| get_symbol_a_command("@1test"));
    assert!(result.is_err());
}


fn get_symbol_l_command(command: &str) -> &str {
    lazy_static! { // lazy_static ensures compilation only happens once
        static ref RE : Regex = Regex::new(
                r"^\(([^\d][a-zA-Z0-9_\.$:]*)\)\s*(\S*)"
            ).unwrap();
    };

    let capture = RE.captures(command)
        .expect("Invalid symbol");

    // ensure the L_Command has no garbage after it 
    assert_eq!("", capture.get(2).unwrap().as_str(), "Invalid L Command: {}", command);

    capture.get(1).unwrap().as_str()
}

#[test]
fn test_get_symbol_l_command() {
    assert_eq!("test", get_symbol_l_command("(test)"));
    assert_eq!("Test_:123$", get_symbol_l_command("(Test_:123$)"));
    let result = std::panic::catch_unwind(|| get_symbol_l_command("(Test_:123$%)")); // % is invalid
    assert!(result.is_err());
    let result = std::panic::catch_unwind(|| get_symbol_l_command("(test) test"));
    assert!(result.is_err());
    let result = std::panic::catch_unwind(|| get_symbol_l_command("(1test)"));
    assert!(result.is_err());
}


fn main() {

    // get user args
    let args: Vec<String> = env::args().collect();

    // check user args
    if args.len() < 2 {
        println!("Missing arguments!");
        println!("Usage: run FILENAME");
        panic!();
    }

    // assemble it
    let file_contents = get_file_contents(&args[1]);

    for (ii, line) in file_contents.lines().enumerate() {

        println!("Line {}:\n{}", ii, line);
        
        // parse line
        let parsed_line = parse_line(&line);
        println!("Parsed:\n{}", parsed_line);
        
        // get command type of line
        let command_type = get_command_type(parsed_line);
        println!("Type:\n{}", command_type);

        // get symbol if A or L command
        if command_type == "A_COMMAND" {
            let symbol = get_symbol_a_command(parsed_line);
            println!("A Symbol:\n{}", symbol)
        } else if command_type == "L_COMMAND" {
            let symbol = get_symbol_l_command(parsed_line);
            println!("L Symbol:\n{}", symbol)
        }

        // // get dest, cmp, and jmp bits if C command
        // if command_type == "C_COMMAND" {
        //     (dest, cmp, jmp) = get_bits(parsed_line);
        // }

        println!("\n")
    }
}

















// fn parse_line_mod(line: &str) -> String {
    
//     // find the index where comments begin on the line
//     let idx_comment = match line.find("//") {
//         Some(idx) => idx,
//         _ => line.len()
//     };
//     println!("{}", idx_comment);

//     // truncate the line after the comment chars
//     line[0..idx_comment].to_string()
//     // String::from("dummy")
// }