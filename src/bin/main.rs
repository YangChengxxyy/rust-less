use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use rust_less::parse_less_file;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "-h" | "--help" => {
            print_help();
        },
        "-v" | "--version" => {
            println!("rust-less v{}", env!("CARGO_PKG_VERSION"));
        },
        input_file => {
            if !input_file.ends_with(".less") {
                eprintln!("Error: Input file must have a .less extension");
                return;
            }

            let output_file = input_file.replace(".less", ".css");
            println!("Converting {} to {}", input_file, output_file);

            match parse_less_file(input_file) {
                Ok(css) => {
                    match fs::File::create(&output_file) {
                        Ok(mut file) => {
                            if let Err(e) = file.write_all(css.as_bytes()) {
                                eprintln!("Error writing to output file: {}", e);
                            } else {
                                println!("Successfully converted to {}", output_file);
                            }
                        },
                        Err(e) => eprintln!("Error creating output file: {}", e),
                    }
                },
                Err(e) => eprintln!("Error parsing LESS file: {}", e),
            }
        }
    }
}

fn print_help() {
    println!("rust-less - A LESS to CSS compiler written in Rust");
    println!("Usage: rust-less [options] <file.less>");
    println!("");
    println!("Options:");
    println!("  -h, --help     Display this help message");
    println!("  -v, --version  Display version information");
    println!("");
    println!("Examples:");
    println!("  rust-less style.less     # Converts style.less to style.css");
}