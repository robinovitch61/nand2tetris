
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::env;
use std::collections::HashMap;

use regex::Regex;
#[macro_use]
extern crate lazy_static;

// create an enum type called CommandType
// that implements the Debug trait
//     (printing with {:?} tells type)
#[derive(PartialEq, Eq, Debug)]
enum CommandType {
    A,
    L,
    C
}

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


/// Returns a raw line after removing comments and white space
/// 
/// # Arguments
/// 
/// * `line` - A mutable reference that holds the current line
fn remove_comments(line: &str) -> &str {
    
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
fn test_raw_line() {
    assert_eq!("", remove_comments(""));
    assert_eq!("", remove_comments("    "));
    assert_eq!("", remove_comments("//   "));
    assert_eq!("nand2tetris", remove_comments("nand2tetris   // is so cool"));
    assert_eq!("nand2tetris is so cool", remove_comments("nand2tetris is so cool // eh?"));
}


/// commandType
/// 
/// Returns a string slice that indicates command type
/// 
/// # Arguments
/// 
/// * `line` - A string slice that holds the current line
fn get_command_type(line: &str) -> CommandType {
    if line.contains('@') {
        return CommandType::A;
    } else if line.contains('(') && line.contains(')') {
        return CommandType::L;
    }
    CommandType::C
}

#[test]
fn test_get_command_type() {
    assert_eq!(CommandType::A, get_command_type("@test"));
    assert_eq!(CommandType::L, get_command_type("(test)"));
    assert_eq!(CommandType::C, get_command_type("A=1;JEQ"));
    assert_eq!(CommandType::C, get_command_type("A=M"));
    assert_eq!(CommandType::C, get_command_type("0;JMP"));
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


fn get_c_command_mnemonics(command: &str) -> (&str, &str, &str) {
    lazy_static! { // lazy_static ensures compilation only happens once
        static ref RE : Regex = Regex::new(
                r"^([MDA]?[MD]?[D]?)\s*=?\s*([01\-DA!M][1DA+\-&|M]?[1ADM]?\s*);?\s*(J?[GELNM]?[TQEP]?)"
            ).unwrap();
    };

    let capture = RE.captures(command)
        .expect("Invalid symbol");

    (
        capture.get(1).unwrap().as_str(), // dest
        capture.get(2).unwrap().as_str(), // comp
        capture.get(3).unwrap().as_str(), // jump
    )
}

#[test]
fn test_get_c_command_mnemonics() {
    // dest, comp, jump  --- dest=comp;jump
    assert_eq!(("M", "D", "JMP"), get_c_command_mnemonics("M=D;JMP"));
    assert_eq!(("", "D", "JGT"), get_c_command_mnemonics("D;JGT"));
    assert_eq!(("", "D", ""), get_c_command_mnemonics("D"));
    assert_eq!(("M", "D", ""), get_c_command_mnemonics("M=D"));
    assert_eq!(("M", "D+M", ""), get_c_command_mnemonics("M=D+M"));
    assert_eq!(("M", "M+1", ""), get_c_command_mnemonics("M=M+1"));
    assert_eq!(("", "0", "JMP"), get_c_command_mnemonics("0;JMP"));
}

// fn convert_to_bits(raw_line: &str) -> &str {

//     let symbol = get_symbol(raw_line);

//     // get dest, comp, and jump mnemonics if C command
//     if symbol == "" { // proxy for "if C command"
//         let (dest, comp, jump) = get_c_command_mnemonics(raw_line);
//         println!("dest mnemonic: {}", dest);
//         println!("comp mnemonic: {}", comp);
//         println!("jump mnemonic: {}", jump);
//         let bits = mnemonics_to_bits(dest, comp, jump);

//     } else {
//         let bits = symbol_to_bits(symbol);
//     }

//     println!("\n");

//     "BITS"
// }

fn main() {

    // define initial hashmap of builtins
    let mut symbol_table: HashMap<&str, String> = HashMap::new();
    symbol_table.insert("SP", "0".to_string());
    symbol_table.insert("LCL", "1".to_string());
    symbol_table.insert("ARG", "2".to_string());
    symbol_table.insert("THIS", "3".to_string());
    symbol_table.insert("THAT", "4".to_string());
    symbol_table.insert("R0", "0".to_string());
    symbol_table.insert("R1", "1".to_string());
    symbol_table.insert("R2", "2".to_string());
    symbol_table.insert("R3", "3".to_string());
    symbol_table.insert("R4", "4".to_string());
    symbol_table.insert("R5", "5".to_string());
    symbol_table.insert("R6", "6".to_string());
    symbol_table.insert("R7", "7".to_string());
    symbol_table.insert("R8", "8".to_string());
    symbol_table.insert("R9", "9".to_string());
    symbol_table.insert("R10", "10".to_string());
    symbol_table.insert("R11", "11".to_string());
    symbol_table.insert("R12", "12".to_string());
    symbol_table.insert("R13", "13".to_string());
    symbol_table.insert("R14", "14".to_string());
    symbol_table.insert("R15", "15".to_string());
    symbol_table.insert("SCREEN", "16384".to_string());
    symbol_table.insert("KBD", "24576".to_string());

    // get user args
    let args: Vec<String> = env::args().collect();

    // check user args
    if args.len() < 2 {
        println!("Missing arguments!");
        println!("Usage: run FILENAME");
        panic!();
    }

    // get file contents
    let file_contents = get_file_contents(&args[1]);

    // first pass: add L symbols to symbol table
    let mut line_count = -1;
    for line in file_contents.lines() {

        // strip comments
        let raw_line = remove_comments(&line);
        if raw_line == "" { continue };

        line_count += 1;
        
        // get command type of line
        let command_type = get_command_type(raw_line);

        if let CommandType::L = command_type { // this is a condensed match
            let symbol = get_symbol_l_command(raw_line);

            match symbol_table.get(&symbol) {
                Some(_) => { continue },
                _ => { // if symbol not already in table
                    symbol_table.insert(symbol, line_count.to_string());
                    line_count -= 1; // don't count label symbol as a line
                }
            }
        }
    }

    for (key, value) in &symbol_table {
        println!("{}: {}", key, value);
    }

    // second pass: replace symbols with numbers, convert to bits
    let mut var_count = 16;
    for (ii, line) in file_contents.lines().enumerate() {

        println!("Line {}: '{}'", ii, line);
        
        // strip comments
        let raw_line = remove_comments(&line);
        println!("Parsed: '{}'", raw_line);
        if raw_line == "" { continue };
        
        // get command type of line
        let command_type = get_command_type(raw_line);
        println!("Type: '{:?}'", command_type);

        let symbol = match command_type {
            CommandType::A => {
                let symbol = get_symbol_a_command(raw_line);

                // put var number in symbol table if not already
                match symbol_table.get(&symbol) {
                    Some(_) => {}, // do nothing if already in table
                    _ => { // if symbol not already in table
                        symbol_table.insert(&symbol, var_count.to_string());
                        var_count += 1; // increment variables declared
                    }
                }
                symbol
            }
            CommandType::L => {
                // get_symbol_l_command(raw_line)
                continue // no bits for L commands
            }
            CommandType::C => {
                ""
            }
        };

        // replace symbol with number in line if necessary
        let parsed_line = match symbol {
            "" => raw_line.to_string(),
            _ => raw_line.replace(symbol, symbol_table.get(&symbol).unwrap())
        };

        println!("Parsed line: {}", parsed_line);
        
        // create bit line based on parsed
        let bits = match command_type {
            CommandType::A => {
                get_a_bits(parsed_line)
            }
            CommandType::C => {
                get_c_bits(parsed_line)
            }
            CommandType::L => {
                panic!();
            }
        };

        // write bits to new file
    }
}

















// fn raw_line_mod(line: &str) -> String {
    
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



        // let symbol = if command_type == CommandType::A {
        //     // get symbol
        //     let symbol = get_symbol_a_command(raw_line);

        //     // put var number in symbol table if not already
        //     match symbol_table.get(&symbol) {
        //         Some(_) => {}, // do nothing if already in table
        //         _ => { // if symbol not already in table
        //             symbol_table.insert(&symbol, var_count.to_string());
        //             var_count += 1; // increment variables declared
        //         }
        //     }
        //     symbol
        // } else if command_type == CommandType::L {
        //     // get symbol
        //     get_symbol_l_command(raw_line)
        // } else {
        //     panic!()
        // };