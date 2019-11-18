// VM Translator for the Nand2Tetris Hack Computer
// Author: Leo Robinovitch

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::io::BufReader;
use std::io::prelude::*;
use std::env;

use regex::Regex;
#[macro_use]
extern crate lazy_static;

// create an enum type called CommandType
// that implements the Debug, etc. traits
//     (printing with {:?} tells type)
#[derive(PartialEq, Eq, Debug)]
enum CommandType {
    CArithmetic,
    CPush,
    CPop,
    CLabel,
    CGoTo,
    CIfGoTo,
    CFunction,
    CReturn,
    CCall
}

#[derive(PartialEq, Eq, Debug)]
enum SegType {
    SConstant,
    SLocal,
    SArgument,
    SThis,
    SThat,
    STemp,
    SPointer,
    SStatic
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


/// Create and return writable file based on path
/// 
/// # Arguments
/// 
/// * `path`
fn create_file(path: &Path) -> fs::File {
    fs::File::create(&path).unwrap()
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

/// Get .vm extension filepaths in a directory. If .vm file specified, wraps
/// input filename in vector.
/// 
/// # Arguments
/// 
/// * vector of pathbufs in dir with .vm extensions
fn get_vm_filepaths(dir: &str) -> Vec<PathBuf>{

    match PathBuf::from(dir).extension() {
        Some(ext) => {
            if ext == "vm" {
                return vec![PathBuf::from(dir)];
            } else {
                panic!("Cannot translate non-.vm file");
            }
        }
        _ => {
            let paths = fs::read_dir(dir).unwrap();

            let mut vm_paths: Vec<PathBuf> = Vec::new();
            for direntry in paths {
                println!("{:?}", direntry);
                let path = direntry.unwrap().path();
                if path.extension().unwrap() == "vm" {
                    println!("{:?}", path);
                    vm_paths.push(path);
                }
            }
            vm_paths
        }
    }
}


/// Parse command line arguments and return input file
/// contents and output file to write to
fn parse_args() -> (Vec<String>, Vec<String>, fs::File, String) {
    // get user args
    let args: Vec<String> = env::args().collect();

    // check user args
    if args.len() < 2 {
        println!("\nMissing required argument");
        println!("Usage: cargo run FILENAME\n");
        panic!();
    };

    // .vm file contents
    let vm_filepaths = get_vm_filepaths(&args[1]);
    let mut file_contents: Vec<String> = Vec::new();
    let mut input_files: Vec<String> = Vec::new();
    for filepath in &vm_filepaths {
        file_contents.push(get_file_contents(&filepath, "vm"));
        input_files.push(filepath.as_path().to_str().unwrap().to_string());
    }

    // output file
    let out_path_str = match vm_filepaths.len() {
        0 => {
            panic!("No .vm files to translate found")
        },
        1 => {
            let path = &vm_filepaths[0];
            path.to_str().unwrap().replace(".vm", ".asm")
        },
        _ => {
            let mut out_path_str = String::from(&args[1]);
            out_path_str.pop();
            out_path_str.push_str(&".asm");
            out_path_str
        }
    };
    println!("HERE: {:?}", out_path_str);
    let out_path = Path::new(&out_path_str);
    let output_file = create_file(out_path);
    println!("file: {:?}", output_file);

    (file_contents, input_files, output_file, out_path_str.to_string())
}

/// Returns a cleaned string slice after removing comments and white space
/// 
/// # Arguments
/// 
/// * `line` - the raw line
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
    assert_eq!("nand2tetris is so cool",
        remove_comments("nand2tetris is so cool // eh?"));
}


/// Checks if VM code line is one of the 9 possible
/// CArithmetic commands
/// 
/// # Arguments
/// 
/// * `line` - string slice that holds possible arithmetic command
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


/// Gets command type for VM code line
/// 
/// # Arguments
/// 
/// * `line` - string slice that holds the current line
fn get_command_type(line: &str) -> CommandType {
    if is_arithmetic(line) {
        CommandType::CArithmetic
    } else if &line[0..3] == "pop" {
        CommandType::CPop
    } else if &line[0..4] == "push" {
        CommandType::CPush
    } else if &line[0..4] == "goto"{
        CommandType::CGoTo
    } else if &line[0..4] == "call"{
        CommandType::CCall
    } else if &line[0..5] == "label" {
        CommandType::CLabel
    } else if &line[0..6] == "return"{
        CommandType::CReturn
    } else if &line[0..7] == "if-goto"{
        CommandType::CIfGoTo
    } else if &line[0..8] == "function"{
        CommandType::CFunction
    } else {
        panic!("Line not a valid command!");
    }
}

#[test]
fn test_get_command_type() {
    assert_eq!(CommandType::CPush, get_command_type("push constant 1"));
    assert_eq!(CommandType::CPop, get_command_type("pop constant 1"));
    assert_eq!(CommandType::CArithmetic, get_command_type("eq"));
}


/// Get segment and index from push/pop command
/// 
/// # Arguments
/// 
/// * `line` - push/pop command
fn parse_push_pop(line: &str) -> (SegType, i32) {
    lazy_static! { // lazy_static ensures compilation only happens once
        static ref RE : Regex = Regex::new(
                r"^(push|pop)\s*(\w*)\s*(\d*)" // TO DO may need refining
            ).unwrap();
    };

    let capture = RE.captures(line)
        .expect("Invalid push/pop command!");

    let segment = match capture.get(2).unwrap().as_str() {
        "constant" => SegType::SConstant,
        "local" => SegType::SLocal,
        "argument" => SegType::SArgument,
        "this" => SegType::SThis,
        "that" => SegType::SThat,
        "temp" => SegType::STemp,
        "pointer" => SegType::SPointer,
        "static" => SegType::SStatic,
        _ => panic!("Invalid segment")
    };

    let index = capture.get(3).unwrap().as_str().parse::<i32>().unwrap();
    (segment, index)
}

#[test]
fn test_parse_push_pop() {
    assert_eq!((SegType::SStatic, 1), parse_push_pop("push static 1"));
    assert_eq!((SegType::SLocal, 10), parse_push_pop("pop local 10"));
    let result = std::panic::catch_unwind(|| parse_push_pop("eq")); // % is invalid
    assert!(result.is_err());
}


/// Get file name from path
/// 
/// # Arguments
/// 
/// * `path` - path
fn get_file_name(path: &str) -> String {
    match path.rfind('/') {
        Some(idx_slash) => {
            match path.rfind('.') {
                Some(idx_dot) => path[idx_slash+1..idx_dot].to_string(),
                _ => path.to_string()
            }
        },
        _ => {
            match path.rfind('.') {
                Some(idx_dot) => path[..idx_dot].to_string(),
                _ => path.to_string()
            }
        }
    }
}

#[test]
fn test_get_file_name() {
    assert_eq!("test", get_file_name("/asdfasdf/asdfasdf/test.vm"));
    assert_eq!("test", get_file_name("/test.vm"));
    assert_eq!("test", get_file_name("test.vm"));
    assert_eq!("test", get_file_name("test"));
}


/// Write assembly code for push commands
/// 
/// # Arguments
/// 
/// * `file` - output file
/// * `input_filename` - input filename, used for static vars
/// * `line` - push command
/// * `segment` - memory segment
/// * `index`
fn write_push(file: &fs::File, input_filename: &str, line: &str, segment: SegType, index: i32) {
    match segment {
        SegType::SConstant => {
            // in constant segment, index is treated as value to push on to stack
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
        SegType::SLocal => {
            // in local segment, push puts index in to local memory on to stack
            let asm_code = format!("// {line}\n\
                @{index}\n\
                D=A\n\
                @LCL\n\
                A=M\n\
                A=A+D\n\
                D=M\n\
                @SP\n\
                A=M\n\
                M=D\n\
                @SP\n\
                M=M+1", line=line, index=index);
            write_to_file(file, asm_code)
        },
        SegType::SArgument => {
            // in argument segment, push puts index in to argument memory on to stack
            let asm_code = format!("// {line}\n\
                @{index}\n\
                D=A\n\
                @ARG\n\
                A=M\n\
                A=A+D\n\
                D=M\n\
                @SP\n\
                A=M\n\
                M=D\n\
                @SP\n\
                M=M+1", line=line, index=index);
            write_to_file(file, asm_code)
        },
        SegType::SThis => {
            // in this segment, push puts index in to this memory on to stack
            let asm_code = format!("// {line}\n\
                @{index}\n\
                D=A\n\
                @THIS\n\
                A=M\n\
                A=A+D\n\
                D=M\n\
                @SP\n\
                A=M\n\
                M=D\n\
                @SP\n\
                M=M+1", line=line, index=index);
            write_to_file(file, asm_code)
        },
        SegType::SThat => {
            // in that segment, push puts index in to that memory on to stack
            let asm_code = format!("// {line}\n\
                @{index}\n\
                D=A\n\
                @THAT\n\
                A=M\n\
                A=A+D\n\
                D=M\n\
                @SP\n\
                A=M\n\
                M=D\n\
                @SP\n\
                M=M+1", line=line, index=index);
            write_to_file(file, asm_code)
        },
        SegType::STemp => {
            // in temp segment, push puts index in to temp memory on to stack
            let asm_code = format!("// {line}\n\
                @{index}\n\
                D=A\n\
                @5\n\
                A=A+D\n\
                D=M\n\
                @SP\n\
                A=M\n\
                M=D\n\
                @SP\n\
                M=M+1", line=line, index=index);
            write_to_file(file, asm_code)
        },
        SegType::SPointer => {
            // in pointer segment, push puts index in to pointer memory on to stack
            let asm_code = format!("// {line}\n\
                @{index}\n\
                D=A\n\
                @3\n\
                A=A+D\n\
                D=M\n\
                @SP\n\
                A=M\n\
                M=D\n\
                @SP\n\
                M=M+1", line=line, index=index);
            write_to_file(file, asm_code)
        },
        SegType::SStatic => {
            // in static segment, push puts index in to static memory on to stack
            let asm_code = format!("// {line}\n\
                @{input_filename}.{index}\n\
                D=M\n\
                @SP\n\
                A=M\n\
                M=D\n\
                @SP\n\
                M=M+1", line=line, input_filename=input_filename, index=index);
            write_to_file(file, asm_code)
        }
    }
}


// Write assembly code for pop commands
/// 
/// # Arguments
/// 
/// * `file` - output file
/// * `input_filename` - input filename, used for static vars
/// * `line` - pop command
/// * `segment` - memory segment
/// * `index`
fn write_pop(file: &fs::File, input_filename: &str, line: &str, segment: SegType, index: i32) {
    match segment {
        SegType::SConstant => {
            // in constant segment, pop should not be implemented
            panic!("pop constant command is invalid");
        },
        SegType::SLocal => {
            // in local segment, pop puts top of stack at index in to local memory
            let asm_code = format!("// {line}\n\
                @{index}\n\
                D=A\n\
                @LCL\n\
                A=M\n\
                D=D+A\n\
                @R13\n\
                M=D\n\
                @SP\n\
                AM=M-1\n\
                D=M\n\
                @R13\n\
                A=M\n\
                M=D", line=line, index=index);
            write_to_file(file, asm_code)
        },
        SegType::SArgument => {
            // in argument segment, pop puts top of stack at index in to argument memory
            let asm_code = format!("// {line}\n\
                @{index}\n\
                D=A\n\
                @ARG\n\
                A=M\n\
                D=D+A\n\
                @R13\n\
                M=D\n\
                @SP\n\
                AM=M-1\n\
                D=M\n\
                @R13\n\
                A=M\n\
                M=D", line=line, index=index);
            write_to_file(file, asm_code)
        },
        SegType::SThis => {
            // in this segment, pop puts top of stack at index in to this memory
            let asm_code = format!("// {line}\n\
                @{index}\n\
                D=A\n\
                @THIS\n\
                A=M\n\
                D=D+A\n\
                @R13\n\
                M=D\n\
                @SP\n\
                AM=M-1\n\
                D=M\n\
                @R13\n\
                A=M\n\
                M=D", line=line, index=index);
            write_to_file(file, asm_code)
        },
        SegType::SThat => {
            // in that segment, pop puts top of stack at index in to that memory
            let asm_code = format!("// {line}\n\
                @{index}\n\
                D=A\n\
                @THAT\n\
                A=M\n\
                D=D+A\n\
                @R13\n\
                M=D\n\
                @SP\n\
                AM=M-1\n\
                D=M\n\
                @R13\n\
                A=M\n\
                M=D", line=line, index=index);
            write_to_file(file, asm_code)
        },
        SegType::STemp => {
            // in temp segment, pop puts top of stack at index in to temp memory
            let asm_code = format!("// {line}\n\
                @{index}\n\
                D=A\n\
                @5\n\
                D=D+A\n\
                @R13\n\
                M=D\n\
                @SP\n\
                AM=M-1\n\
                D=M\n\
                @R13\n\
                A=M\n\
                M=D", line=line, index=index);
            write_to_file(file, asm_code)
        },
        SegType::SPointer => {
            // in pointer segment, pop puts top of stack at index in to pointer memory
            let asm_code = format!("// {line}\n\
                @{index}\n\
                D=A\n\
                @3\n\
                D=D+A\n\
                @R13\n\
                M=D\n\
                @SP\n\
                AM=M-1\n\
                D=M\n\
                @R13\n\
                A=M\n\
                M=D", line=line, index=index);
            write_to_file(file, asm_code)
        },
        SegType::SStatic => {
            // in static segment, pop puts top of stack at index in to static memory
            let asm_code = format!("// {line}\n\
                @SP\n\
                AM=M-1\n\
                D=M\n\
                @{input_filename}.{index}\n\
                M=D", line=line, input_filename=input_filename, index=index);
            write_to_file(file, asm_code)
        }   
    }
}


/// Writes assembly code for arithmetic commands to output file
/// 
/// # Arguments
/// 
/// * `file` - output file
/// * `line` - input arithmetic command ("add", "sub", "eq", etc.)
/// * `cmp_count` - count of previous comparison operations ("eq", "lt", and "gt").
///     Used to mark jump and continue locations in these operations such that each
///     label is unique.
fn write_arithmetic(file: &fs::File, line: &str, cmp_count: i32) {

    match line {
        "add" => {
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
                A=M-1\n\
                M=!M", line=line);
            write_to_file(file, asm_code)
        },
        _ => {}
    }
}


/// Writes assembly code for label commands to output file
/// TODO: check if need to prepend filename or function?
/// 
/// # Arguments
/// 
/// * `file` - output file
/// * `line` - input label command
fn write_label(file: &fs::File, line: &str) {
    lazy_static! { // lazy_static ensures compilation only happens once
        static ref RE : Regex = Regex::new(
                r"^label ([a-zA-Z0-9._:]+)$"
            ).unwrap();
    };

    let capture = RE.captures(line)
        .expect("Invalid label command!");

    let label = capture.get(1).unwrap().as_str();
    let asm_code = format!("// {line}\n\
        ({label})", line=line, label=label);
    write_to_file(file, asm_code);
}


/// Writes assembly code for unconditional goto commands to output file
/// 
/// # Arguments
/// 
/// * `file` - output file
/// * `line` - input unconditional goto command
fn write_goto(file: &fs::File, line: &str) {
    lazy_static! { // lazy_static ensures compilation only happens once
        static ref RE : Regex = Regex::new(
                r"^goto ([a-zA-Z0-9._:]+)$"
            ).unwrap();
    };

    let capture = RE.captures(line)
        .expect("Invalid goto command!");

    let label = capture.get(1).unwrap().as_str();
    let asm_code = format!("// {line}\n\
        @{label}\n\
        0;JMP", line=line, label=label);
    write_to_file(file, asm_code);
}


/// Writes assembly code for conditional goto commands to output file
/// 
/// # Arguments
/// 
/// * `file` - output file
/// * `line` - input unconditional goto command
fn write_ifgoto(file: &fs::File, line: &str) {
    lazy_static! { // lazy_static ensures compilation only happens once
        static ref RE : Regex = Regex::new(
                r"^if-goto ([a-zA-Z0-9._:]+)$"
            ).unwrap();
    };

    println!("{}", line);
    let capture = RE.captures(line)
        .expect("Invalid if-goto command!");

    let label = capture.get(1).unwrap().as_str();
    let asm_code = format!("// {line}\n\
        @SP\n\
        AM=M-1\n\
        D=M\n\
        @{label}\n\
        D;JNE", line=line, label=label);
    write_to_file(file, asm_code);
}


/// Writes assembly code for function declarations to output file
/// 
/// # Arguments
/// 
/// * `file` - output file
/// * `line` - input unconditional goto command
fn write_function(file: &fs::File, line: &str) {
    lazy_static! { // lazy_static ensures compilation only happens once
        static ref RE : Regex = Regex::new(
                r"^function ([a-zA-Z0-9._:]+) ([0-9]+)$"
            ).unwrap();
    };

    println!("{}", line);
    let capture = RE.captures(line)
        .expect("Invalid function command!");

    let func_name = capture.get(1).unwrap().as_str();
    let n_args = capture.get(2).unwrap().as_str().parse::<i32>().unwrap();
    let mut asm_code = format!("// {line}\n\
        ({func_name})", line=line, func_name=func_name);
    let push_0_asy = "\n@SP\nM=0\nA=A+1";
    for _ in 0..n_args {
        asm_code.push_str(push_0_asy);
    }
    write_to_file(file, asm_code);
}

// TODO: TEST THIS HERE^!

/// Writes assembly code for function declarations to output file
/// 
/// # Arguments
/// 
/// * `file` - output file
/// * `line` - input unconditional goto command
fn write_call(file: &fs::File, line: &str) {

}


/// Writes assembly code for function declarations to output file
/// 
/// # Arguments
/// 
/// * `file` - output file
/// * `line` - input unconditional goto command
fn write_return(file: &fs::File, line: &str) {

}


/// ********************************
/// ************* MAIN *************
/// ********************************
fn main () {

    let (file_contents, in_paths, output_file, out_path) = parse_args();
    let mut it_file_contents = file_contents.iter();
    let mut it_in_paths = in_paths.iter();
    while let (Some(contents), Some(in_path)) = (it_file_contents.next(), it_in_paths.next()) {
        let in_file_name = get_file_name(&in_path);

        let mut cmp_count = 0;
        for line in contents.lines() {
            let clean_line = remove_comments(line);
            if clean_line == "" { continue };
            println!("Clean line: {}", clean_line);

            let command_type = get_command_type(line);
            println!("Command type: {:?}", command_type);

            match command_type {
                CommandType::CPush => {
                    let (segment, index) = parse_push_pop(clean_line);
                    write_push(&output_file, &in_file_name, line, segment, index);
                },
                CommandType::CPop => {
                    let (segment, index) = parse_push_pop(clean_line);
                    write_pop(&output_file, &in_file_name, line, segment, index);
                },
                CommandType::CArithmetic => {
                    write_arithmetic(&output_file, clean_line, cmp_count);
                    cmp_count += 1;
                },
                CommandType::CLabel => {
                    write_label(&output_file, clean_line);
                },
                CommandType::CGoTo => {
                    write_goto(&output_file, clean_line);
                },
                CommandType::CIfGoTo => {
                    write_ifgoto(&output_file, clean_line);
                },
                CommandType::CFunction => {
                    write_function(&output_file, clean_line);
                },
                CommandType::CCall => {
                    write_call(&output_file, clean_line);
                },
                CommandType::CReturn => {
                    write_return(&output_file, clean_line);
                },
                _ => ()
            }

        }
        println!("\nTranslated {:?}\n        -> {:?}\n", in_path, out_path);
    }
}