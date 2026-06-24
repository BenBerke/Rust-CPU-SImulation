use std::fs;
use crate::cpu::Core;
use crate::opcodes::{OP_ADD, OP_HALT};

pub fn load_bootloader(mut cpu: &Core, bootloader_path: String){
    let mut bootloader_code = String::new();

    // Turn boot_loader.
    match fs::read_to_string(bootloader_path) {
        Ok(contents) => { bootloader_code = contents; }
        Err(e) => { println!("Error: {}", e); }
    }

    let mut chars = bootloader_code.chars().peekable();
    let mut write_address = 0;

    // Load the bootloader. Lexer
    let mut compiled_bytes: Vec<u8> = Vec::new();

    while let Some(&c) = chars.peek() {
        if c.is_whitespace(){ chars.next(); }
        else if c.is_ascii_digit(){
            let mut num_str = String::new();

            while let Some(&c) = chars.peek() {
                if !c.is_ascii_alphabetic() { chars.next(); }
                num_str.push(chars.next().unwrap());
            }

            let number_val: u8 = num_str.parse().unwrap();
            compiled_bytes.push(number_val);
        }
    }
}

