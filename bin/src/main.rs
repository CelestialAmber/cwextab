use cwextab::*;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn test_decode(data: &[u8], funcs: &[&str]) {
    let result = decode_extab(data);
    let data: ExceptionTableData = match result {
        Ok(val) => val,
        Err(e) => {
            panic!(
                concat!("Something went wrong with decoding :<\n", "Error: {}"),
                e.to_string()
            );
        }
    };
    println!("{}", data.pc_actions[0].action_offset);

    //Convert the table struct to a string and print it.
    let result = data.to_string(funcs);
    let text: String = match result {
        Some(val) => val,
        None => {
            panic!("Something went wrong with converting to text :<");
        }
    };

    println!("{}", text);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let mut table_bytes: Vec<u8> = vec![];
        let mut func_names: Vec<String> = vec![];

        let file = File::open(&args[1]).unwrap_or_else(
            |_| panic!("Failed to open file \"{}\"", args[1]));
        let reader = BufReader::new(file);
        let lines = reader.lines();

        //println!("Lines: {}", num_lines);

        //Parse the table in the given text file
        for line in lines {
            let cur_line: String =
                line.expect("Idk why tf expect is needed here, never using Rust again i stg");
            let parts: Vec<&str> = cur_line.trim().split(' ').collect();

            if !parts[0].starts_with(".4byte") {
                println!("Error: Invalid line in table, must start with .4byte");
                return;
            }

            let value: String = parts[1].to_string();

            let mut line_val: u32 = 0; //32 bit value for current line

            //32 bit value
            if let Some(hex_string) = value.strip_prefix("0x") {
                line_val = u32::from_str_radix(hex_string, 16).expect("Failed to parse hex value");
            } else {
                //Otherwise, treat as a function name
                let length: usize = value.len();
                let func_name: String = if value.starts_with('"') && value.ends_with('"') {
                    value[1..length - 1].to_string()
                } else {
                    value.to_string()
                };

                func_names.push(func_name);
            }

            let bytes = line_val.to_be_bytes();
            table_bytes.extend_from_slice(&bytes);
        }

        //Rust's stupid borrow bs has left me no choice ;<
        let str_array: Vec<&str> = func_names.iter().map(String::as_str).collect();

        test_decode(&table_bytes, &str_array);
    } else {
        println!("Usage: cwextab-bin <file>");
    }
}
