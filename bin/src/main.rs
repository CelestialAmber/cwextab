use cwextab::*;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn test_decode(data: &[u8], funcs: Vec<String>) {
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

fn read_all_lines_from_file(path: &str) -> Vec<String> {
    let file = File::open(path).expect(&format!("Failed to open file \"{}\"", path));
    let reader = BufReader::new(file);
    reader.lines().map(|line| line.expect("Could not parse line")).collect()
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let mut table_bytes: Vec<u8> = vec![];
        let mut func_names: Vec<String> = vec![];

        let lines = read_all_lines_from_file(&args[1]);

        //println!("Lines: {}", num_lines);

        //Parse the table in the given text file
        for line in lines {
            let cur_line: String = line;
            let parts: Vec<&str> = cur_line.trim().split(' ').collect();

            let data_size: u32 =
            if parts[0].starts_with(".4byte") {
                4
            } else if parts[0].starts_with(".2byte") {
                2
            } else {
                println!("Error: Invalid line in table, must start with .4byte");
                return;
            };

            let value: String = parts[1].to_string();

            let mut line_val: u32 = 0; //Value for current line (16/32 bit)

            //Hex value
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

            let bytes: &[u8] =
            if data_size == 4 {
                &line_val.to_be_bytes()
            } else {
                let u16_val: u16 = line_val as u16;
                &u16_val.to_be_bytes()
            };
            table_bytes.extend_from_slice(bytes);
        }

        test_decode(&table_bytes, func_names);
    } else {
        println!("Usage: cwextab-bin <file>");
    }
}
