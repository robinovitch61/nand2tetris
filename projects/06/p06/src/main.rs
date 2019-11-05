
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
    A,
    L,
    C
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


/// Returns a str after removing comments and white space
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


/// Returns CommandType of input line
/// 
/// # Arguments
/// 
/// * `line` - a string slice that holds the current line
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


/// Get symbol represented in A command, e.g. `@SYMBOL`
/// 
/// # Arguments
/// 
/// * `command` - a string slice that holds the A command with the symbol
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


/// Checks if the A command contains a symbol (returns true) or 
/// refers to a number directly (returns false)
/// 
/// # Arguments
/// 
/// * `stripped_line` - input line free of comments and whitespace
fn a_command_contains_symbol(stripped_line: &str) -> bool {
    lazy_static! { // lazy_static ensures compilation only happens once
        static ref RE : Regex = Regex::new(
                r"^@([^\d][a-zA-Z0-9_\.$:]*)\s*(\S*)"
            ).unwrap();
    };

    match RE.captures(stripped_line) {
        Some(_) => true,
        _ => false
    }
}

#[test]
fn test_a_command_contains_symbol() {
    assert_eq!(true, a_command_contains_symbol("@test"));
    assert_eq!(false, a_command_contains_symbol("@1"));
}


/// Get symbol represented in L command, e.g. `(SYMBOL)`
/// 
/// # Arguments
/// 
/// * `command` - a string slice that holds the L command with the symbol
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


/// Get (dest, comp, jump) mnemonics contained in complete command
/// 
/// # Arguments
/// 
/// * `command` - a string slice that holds the C command with the mnemonics
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


/// Get 16 bit text representation of A command
/// 
/// # Arguments
/// 
/// * `command` - a string slice that holds the A command
fn get_a_bits(command: &str) -> String {
    match command.replace("@", "").parse::<i32>() {
        Ok(num) => format!("{:#018b}", num).to_string().replace("0b", ""),
        Err(e) => {
            panic!("Problem parsing number {:?} to string from line {:?}", e, command)
        },
    }
}

#[test]
fn test_get_a_bits() {
    assert_eq!("0000000000000010", get_a_bits("@2"));
    assert_eq!("0000000000000001", get_a_bits("@1"));
}


/// Get 16 bit text representation of C command
/// 
/// # Arguments
/// 
/// * `command` - a string slice that holds the C command
/// * `comp_map` - hashmap of comp mnemonics to bits
/// * `dest_map` - hashmap of dest mnemonics to bits
/// * `jump_map` - hashmap of jump mnemonics to bits
fn get_c_bits(
    command: &str,
    comp_map: &HashMap<&str, String>,
    dest_map: &HashMap<&str, String>,
    jump_map: &HashMap<&str, String>) -> String {
    
    let (dest, comp, jump) = get_c_command_mnemonics(command);
    let comp_bits = comp_map.get(&comp).expect("No mapping found for comp mnemonic");
    let dest_bits = dest_map.get(&dest).expect("No mapping found for dest mnemonic");
    let jump_bits = jump_map.get(&jump).expect("No mapping found for jump mnemonic");

    let bits = "111".to_string();
    bits + comp_bits + dest_bits + jump_bits
}

#[test]
fn test_get_c_bits() {
    // not implemented
}


/// Create and return writable file based on path
/// 
/// # Arguments
/// 
/// * `path`
fn create_file(path: &Path) -> File {
    File::create(&path).unwrap()
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
    let out_path_str = args[1].replace(".asm", ".hack");
    let out_path = Path::new(&out_path_str);

    let file_contents = get_file_contents(in_path, "asm");
    let output_file = create_file(out_path);

    (file_contents, output_file, in_path_str.to_string(), out_path_str.to_string())
}

#[test]
fn test_parse_args() {
    // not implemented
}


fn main() {

    // define C command hashmaps
    let mut dest_map: HashMap<&str, String> = HashMap::new();
    dest_map.insert("",    "000".to_string());
    dest_map.insert("M",   "001".to_string());
    dest_map.insert("D",   "010".to_string());
    dest_map.insert("MD",  "011".to_string());
    dest_map.insert("A",   "100".to_string());
    dest_map.insert("AM",  "101".to_string());
    dest_map.insert("AD",  "110".to_string());
    dest_map.insert("AMD", "111".to_string());

    let mut comp_map: HashMap<&str, String> = HashMap::new();
    comp_map.insert("0",   "0101010".to_string());
    comp_map.insert("1",   "0111111".to_string());
    comp_map.insert("-1",  "0111010".to_string());
    comp_map.insert("D",   "0001100".to_string());
    comp_map.insert("A",   "0110000".to_string());
    comp_map.insert("M",   "1110000".to_string());
    comp_map.insert("!D",  "0001101".to_string());
    comp_map.insert("!A",  "0110001".to_string());
    comp_map.insert("!M",  "1110001".to_string());
    comp_map.insert("-D",  "0001111".to_string());
    comp_map.insert("-A",  "0110011".to_string());
    comp_map.insert("-M",  "1110011".to_string());
    comp_map.insert("D+1", "0011111".to_string());
    comp_map.insert("A+1", "0110111".to_string());
    comp_map.insert("M+1", "1110111".to_string());
    comp_map.insert("D-1", "0001110".to_string());
    comp_map.insert("A-1", "0110010".to_string());
    comp_map.insert("M-1", "1110010".to_string());
    comp_map.insert("D+A", "0000010".to_string());
    comp_map.insert("D+M", "1000010".to_string());
    comp_map.insert("D-A", "0010011".to_string());
    comp_map.insert("D-M", "1010011".to_string());
    comp_map.insert("A-D", "0000111".to_string());
    comp_map.insert("M-D", "1000111".to_string());
    comp_map.insert("D&A", "0000000".to_string());
    comp_map.insert("D&M", "1000000".to_string());
    comp_map.insert("D|A", "0010101".to_string());
    comp_map.insert("D|M", "1010101".to_string());

    let mut jump_map: HashMap<&str, String> = HashMap::new();
    jump_map.insert("",    "000".to_string());
    jump_map.insert("JGT", "001".to_string());
    jump_map.insert("JEQ", "010".to_string());
    jump_map.insert("JGE", "011".to_string());
    jump_map.insert("JLT", "100".to_string());
    jump_map.insert("JNE", "101".to_string());
    jump_map.insert("JLE", "110".to_string());
    jump_map.insert("JMP", "111".to_string());

    // define initial symbol map of builtins TODO: &str instead of String
    let mut symbol_map: HashMap<&str, String> = HashMap::new();
    symbol_map.insert("SP", "0".to_string());
    symbol_map.insert("LCL", "1".to_string());
    symbol_map.insert("ARG", "2".to_string());
    symbol_map.insert("THIS", "3".to_string());
    symbol_map.insert("THAT", "4".to_string());
    symbol_map.insert("R0", "0".to_string());
    symbol_map.insert("R1", "1".to_string());
    symbol_map.insert("R2", "2".to_string());
    symbol_map.insert("R3", "3".to_string());
    symbol_map.insert("R4", "4".to_string());
    symbol_map.insert("R5", "5".to_string());
    symbol_map.insert("R6", "6".to_string());
    symbol_map.insert("R7", "7".to_string());
    symbol_map.insert("R8", "8".to_string());
    symbol_map.insert("R9", "9".to_string());
    symbol_map.insert("R10", "10".to_string());
    symbol_map.insert("R11", "11".to_string());
    symbol_map.insert("R12", "12".to_string());
    symbol_map.insert("R13", "13".to_string());
    symbol_map.insert("R14", "14".to_string());
    symbol_map.insert("R15", "15".to_string());
    symbol_map.insert("SCREEN", "16384".to_string());
    symbol_map.insert("KBD", "24576".to_string());

    let (file_contents, output_file, in_path, out_path) = parse_args();

    // first pass: add L symbols to symbol table
    let mut line_count = -1;
    for line in file_contents.lines() {

        // strip comments
        let stripped_line = remove_comments(&line);
        if stripped_line == "" { continue };

        line_count += 1;
        
        // get command type of line
        let command_type = get_command_type(stripped_line);

        if let CommandType::L = command_type { // this is a condensed match
            let symbol = get_symbol_l_command(stripped_line);

            match symbol_map.get(&symbol) {
                Some(_) => { continue },
                _ => { // if symbol not already in table
                    symbol_map.insert(symbol, line_count.to_string());
                    line_count -= 1; // don't count label symbol as a line
                }
            }
        }
    }

    // second pass: replace symbols with numbers, convert to bits
    let mut var_count = 16;
    for line in file_contents.lines() {

        let stripped_line = remove_comments(&line);
        if stripped_line == "" { continue };
        
        let command_type = get_command_type(stripped_line);

        let parsed_line = match command_type {
            CommandType::A => {
                if a_command_contains_symbol(stripped_line) {
                    let symbol = get_symbol_a_command(stripped_line);

                    // put var number in symbol table if not already
                    match symbol_map.get(&symbol) {
                        Some(_) => {}, // do nothing if already in table
                        _ => { // if symbol not already in table
                            symbol_map.insert(&symbol, var_count.to_string());
                            var_count += 1; // increment variables declared
                        }
                    }
                    
                    // replace symbol with number in line if necessary
                    match symbol {
                        "" => stripped_line.to_string(),
                        _ => stripped_line.replace(symbol, symbol_map.get(&symbol).unwrap())
                    }
                }
                else {
                    stripped_line.to_string()
                }
            }
            CommandType::L => {
                continue // no bits for L commands
            }
            CommandType::C => {
                stripped_line.to_string()
            }
        };

        // create bit line based on parsed
        let bits = match command_type {
            CommandType::A => {
                get_a_bits(&parsed_line)
            }
            CommandType::C => {
                get_c_bits(&parsed_line, &comp_map, &dest_map, &jump_map)
            }
            CommandType::L => {
                panic!();
            }
        };

        write_to_file(&output_file, bits);
    }

    println!("\nAssembled {:?} to {:?}\n", in_path, out_path);
}
