// Compiler for the Nand2Tetris Jack Programming Language
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

            let mut vm_paths: Vec<PathBuf> = Vec::new();
            for direntry in paths {
                let path = direntry.unwrap().path();
                if path.extension().unwrap() == extension {
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
        println!("{:?}", filepath);
        file_contents.push(get_file_contents(&filepath, extension));
        input_files.push(filepath.as_path().to_str().unwrap().to_string());
        let mut xml_filepath = filepath;
        xml_filepath.set_extension("xml");
        output_files.push(fs::File::create(&xml_filepath).unwrap());
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
    // println!("mod_comment: {}", mod_comment);

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

    // find the index where comments begin on the line
    let idx_comment = match mod_line.find("//") {
        Some(idx) => idx,
        _ => mod_line.len()
    };
    // println!("end mod_comment: {}", mod_comment);

    // return a reference to the reduced str with no start/end whitespace
    // note that memory contents are the same, just pointer and/or len changed
    (&mod_line[0..idx_comment].trim(), mod_comment)
}


/// Finds next token in line. Returns token and rest of line.
fn find_next(line: &str) -> (&str, usize) {
    lazy_static! { // lazy_static ensures compilation only happens once
        static ref RE : Regex = Regex::new(
                r"(?x)
                (class\b|constructor\b|function\b|method\b|field\b
                |static\b|var\b|int\b|char\b|boolean\b|void\b|true\b|false\b
                |null\b|this\b|let\b|do\b|if\b|else\b|while\b|return\b)"
            ).unwrap();
    };

    let capture = match RE.find(line){
        Some(cap) => cap,
        _ => panic!("No token found")
    };
    let token = &line[capture.start()..capture.end()];
    (token, capture.end())
}

fn tokenize_line(line: &str, tokens: &Vec<&str>) {
    println!("Tokenizing: {}", line);
    let mut rest = line;
    while rest != "" {
        let (token, idx) = find_next(rest);
        rest = &rest[idx..];
        println!("Found token: {}", token);
        // tokens.push(token);
    }
}


/// ********************************
/// ************* MAIN *************
/// ********************************
fn main () {

    let (file_contents, in_paths, output_files, out_paths) = parse_args();

    let mut it_file_contents = file_contents.iter();
    let mut it_in_paths = in_paths.iter();
    let mut it_out_files = output_files.iter();
    let mut it_out_paths = out_paths.iter();
    
    while let (Some(contents), Some(in_path), Some(out_file), Some(out_path))
        = (it_file_contents.next(), it_in_paths.next(), it_out_files.next(), it_out_paths.next()) {

        // tokenize
        let mut is_comment = false;
        let mut tokens: Vec<&str> = Vec::new();
        for line in contents.lines() {
            println!("\nRaw: {}", line);
            println!("Is comment: {}", is_comment);
            let (clean_line, comment) = remove_comments(line, is_comment);
            is_comment = comment;
            println!("Cleaned: {}", clean_line);
            if clean_line == "" { continue };
            tokenize_line(clean_line, &tokens);
        }
    }
}
    //     println!("\nTranslated {:?}\n        -> {:?}\n", in_path, out_path);
    // }