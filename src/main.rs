extern crate regex;

use regex::Regex;
use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, Read};
use std::process::exit;
use std::str::FromStr;

fn main() {
    let args: Vec<String> = args().skip(1).collect();
    if args.len() == 0 {
        println!("No input file");
        exit(-1);
    }
    let file = File::open(&args.get(0).unwrap());
    match file {
        Ok(f) => {
            println!("{} opened.", args.get(0).unwrap());
            let pair: Vec<&str> = args.get(0).unwrap().split(".").collect();
            let filename = format!("{}.hack", pair.get(0).unwrap());
            let str_bin = parse(f);
            let output = File::create(&filename);
            match output {
                Ok(mut o) => match o.write_all(str_bin.as_bytes()) {
                    Ok(_k) => {
                        println!("{} written.", filename);
                    }
                    Err(e) => {
                        println!("{}", e);
                    }
                },
                Err(e) => {
                    println!("{}", e);
                }
            }
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}

fn parse(f: File) -> String {
    let mut dict: HashMap<&str, u16> = [
        ("R0", 0),
        ("R1", 1),
        ("R2", 2),
        ("R3", 3),
        ("R4", 4),
        ("R5", 5),
        ("R6", 6),
        ("R7", 7),
        ("R8", 8),
        ("R9", 9),
        ("R10", 10),
        ("R11", 11),
        ("R12", 12),
        ("R13", 13),
        ("R14", 14),
        ("R15", 15),
        ("SP", 0),
        ("LCL", 1),
        ("ARG", 2),
        ("THIS", 3),
        ("THAT", 4),
        ("SCREEN", 16384),
        ("KBD", 24576),
    ]
    .iter()
    .cloned()
    .collect();

    let comp_dict: HashMap<&str, &str> = [
        ("0", "0101010"),
        ("1", "0111111"),
        ("-1", "0111010"),
        ("D", "0001100"),
        ("A", "0110000"),
        ("!D", "0001101"),
        ("!A", "0110001"),
        ("-D", "0001111"),
        ("-A", "0110011"),
        ("D+1", "0011111"),
        ("1+D", "0011111"),
        ("A+1", "0110111"),
        ("1+A", "0110111"),
        ("D-1", "0001110"),
        ("A-1", "0110010"),
        ("D+A", "0000010"),
        ("A+D", "0000010"),
        ("D-A", "0010011"),
        ("A-D", "0000111"),
        ("D&A", "0000000"),
        ("A&D", "0000000"),
        ("D|A", "0010101"),
        ("A|D", "0010101"),
        ("M", "1110000"),
        ("!M", "1110001"),
        ("-M", "1110011"),
        ("M+1", "1110111"),
        ("1+M", "1110111"),
        ("M-1", "1110010"),
        ("D+M", "1000010"),
        ("M+D", "1000010"),
        ("D-M", "1010011"),
        ("M-D", "1000111"),
        ("D&M", "1000000"),
        ("M&D", "1000000"),
        ("D|M", "1010101"),
        ("M|D", "1010101"),
    ]
    .iter()
    .cloned()
    .collect();

    let dest_dict: HashMap<&str, &str> = [
        ("M", "001"),
        ("D", "010"),
        ("MD", "011"),
        ("A", "100"),
        ("AM", "101"),
        ("AD", "110"),
        ("AMD", "111"),
    ]
    .iter()
    .cloned()
    .collect();

    let jump_dict: HashMap<&str, &str> = [
        ("JGT", "001"),
        ("JEQ", "010"),
        ("JGE", "011"),
        ("JLT", "100"),
        ("JNE", "101"),
        ("JLE", "110"),
        ("JMP", "111"),
    ]
    .iter()
    .cloned()
    .collect();

    let mut reader: BufReader<File> = BufReader::new(f);
    let mut all = String::new();
    reader.read_to_string(&mut all).unwrap();
    let lines: Vec<&str> = all
        .split('\n')
        .map(|line| {
            let arr: Vec<&str> = line.split("//").collect();
            arr[0].trim()
        })
        .filter(|line| line.len() > 0)
        .collect();

    let reg_a = Regex::new(r"^@(.+)$").unwrap();
    let reg_b = Regex::new(r"^\((.+)\)$").unwrap();
    let reg_d = Regex::new(r"^\d+$").unwrap();

    let mut current: u16 = 0;
    // deal with (xxx)
    let lines_without_labels: Vec<&str> = lines
        .into_iter()
        .map(|line| {
            if reg_b.is_match(line) {
                let sym = reg_b.captures(line).unwrap().get(1).unwrap().as_str();
                dict.insert(sym, current);
                ""
            } else {
                current = current + 1;
                line
            }
        })
        .filter(|line| line.len() > 0)
        .collect();

    let mut alloc_number: u16 = 16;
    // deal with @xxx
    let lines_with_only_numbers: Vec<String> = lines_without_labels
        .into_iter()
        .map(|line| {
            let result: String;
            if reg_a.is_match(&line) {
                let sym = reg_a.captures(&line).unwrap().get(1).unwrap().as_str();
                if reg_d.is_match(sym) {
                    // xxx is digit
                    result = line.to_string();
                } else {
                    if !dict.contains_key(sym) {
                        dict.insert(sym, alloc_number);
                        alloc_number = alloc_number + 1;
                    }
                    result = format!("@{}", dict.get(sym).unwrap());
                }
            } else {
                result = line.to_string();
            }
            result
        })
        .collect();

    let lines_in_binary: Vec<String> = lines_with_only_numbers
        .into_iter()
        .map(|line| {
            let result: String;
            if reg_a.is_match(&line) {
                // A command
                let str_num = reg_a.captures(&line).unwrap().get(1).unwrap().as_str();
                let num: u16 = u16::from_str(str_num).unwrap();
                result = format!("{:0>16b}", num);
            } else {
                // C command
                let no_space = line.replace(" ", "");
                let head: &str = "111"; // 3 bits
                let comp: &str; // 7 bits
                let dest: &str; // 3 bits
                let jump: &str; // 3 bits
                let no_semicolon: Vec<&str> = no_space.split(";").collect();
                let no_equal_sign: Vec<&str> = no_semicolon.get(0).unwrap().split("=").collect();
                match no_equal_sign.get(1) {
                    Some(k) => {
                        // with dest
                        dest = dest_dict.get(no_equal_sign.get(0).unwrap()).unwrap();
                        comp = comp_dict.get(k).unwrap();
                    }
                    None => {
                        // without dest
                        comp = comp_dict.get(no_equal_sign.get(0).unwrap()).unwrap();
                        dest = "000";
                    }
                }
                match no_semicolon.get(1) {
                    Some(k) => {
                        // with jump
                        jump = jump_dict.get(k).unwrap();
                    }
                    None => {
                        // without jump
                        jump = "000";
                    }
                }
                result = format!("{}{}{}{}", head, comp, dest, jump);
            }
            result
        })
        .collect();

    lines_in_binary.join("\n")
}
