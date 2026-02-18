//! Rust LESS 编译器的 CLI 二进制文件

use clap::{Arg, Command};
use rust_less::Compiler;
use std::fs;
use std::io::{self, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("rust-less")
        .version("0.2.4")
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
        .arg(
            Arg::new("source-map")
                .long("source-map")
                .action(clap::ArgAction::SetTrue)
                .help("生成 source map 文件"),
        )
        .arg(
            Arg::new("include-path")
                .long("include-path")
                .value_name("PATH")
                .action(clap::ArgAction::Append)
                .help("添加导入搜索路径"),
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

    let source_map_enabled = matches.get_flag("source-map");

    // Compile LESS to CSS
    let mut compiler = if matches.get_flag("compress") {
        Compiler::compressed()
    } else {
        Compiler::new()
    };

    if source_map_enabled {
        compiler = compiler.with_source_map(true);
    }

    // Add include paths
    if let Some(paths) = matches.get_many::<String>("include-path") {
        for path in paths {
            compiler.add_include_path(path);
        }
    }

    // Compile from file or stdin
    let css_output = if let Some(input_file) = matches.get_one::<String>("input") {
        compiler.compile_file(input_file)?
    } else {
        compiler.compile(&input_content)?
    };

    // Write output
    if let Some(output_file) = matches.get_one::<String>("output") {
        fs::write(output_file, &css_output)?;

        // Write source map if enabled
        if source_map_enabled {
            if let Some(sm_json) = compiler.generate_source_map() {
                let sm_file = format!("{}.map", output_file);
                fs::write(&sm_file, sm_json)?;
                eprintln!("Source map written to {}", sm_file);
            }
        }

        eprintln!("CSS written to {}", output_file);
    } else {
        print!("{}", css_output);
    }

    Ok(())
}
