//! Rust LESS 编译器的 CLI 二进制文件

use clap::{Arg, Command};
use rust_less::{compile, Compiler};
use std::fs;
use std::io::{self, Read};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("rust-less")
        .version("0.2.0")
        .author("Yang Cheng")
        .about("一个用 Rust 编写的 LESS 到 CSS 编译器")
        .arg(
            Arg::new("input")
                .help("输入 LESS 文件")
                .value_name("FILE")
                .index(1),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("输出 CSS 文件"),
        )
        .arg(
            Arg::new("compress")
                .short('c')
                .long("compress")
                .action(clap::ArgAction::SetTrue)
                .help("压缩输出的 CSS"),
        )
        .get_matches();

    // Read input
    let input_content = if let Some(input_file) = matches.get_one::<String>("input") {
        fs::read_to_string(input_file)?
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    // Compile LESS to CSS
    let css_output = if matches.get_flag("compress") {
        let mut compiler = Compiler::compressed();
        compiler.compile(&input_content)?
    } else {
        compile(&input_content)?
    };

    // Write output
    if let Some(output_file) = matches.get_one::<String>("output") {
        fs::write(output_file, css_output)?;
        println!("CSS 已写入 {}", output_file);
    } else {
        print!("{}", css_output);
    }

    Ok(())
}
