//! Rust LESS 编译器的 CLI 二进制文件

use clap::{Arg, Command};
use rust_less::Compiler;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

fn default_source_map_url(map_path: &str) -> String {
    Path::new(map_path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| map_path.to_string())
}

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
            Arg::new("source-map-file")
                .long("source-map-file")
                .value_name("FILE")
                .help("指定 source map 输出文件路径（默认: <output>.map）"),
        )
        .arg(
            Arg::new("source-map-url")
                .long("source-map-url")
                .value_name("URL")
                .help("指定写入 CSS 注释的 sourceMappingURL（默认使用 map 文件名）"),
        )
        .arg(
            Arg::new("source-map-root")
                .long("source-map-root")
                .value_name("ROOT")
                .help("设置 source map 的 sourceRoot 字段"),
        )
        .arg(
            Arg::new("source-map-lessjs-compat")
                .long("source-map-lessjs-compat")
                .action(clap::ArgAction::SetTrue)
                .help("启用 less.js 兼容 source map 输出（names 为空，sourceRoot 改写到 sources）"),
        )
        .arg(
            Arg::new("include-path")
                .long("include-path")
                .value_name("PATH")
                .action(clap::ArgAction::Append)
                .help("添加导入搜索路径"),
        )
        .get_matches();

    let input_file = matches.get_one::<String>("input").cloned();
    let output_file = matches.get_one::<String>("output").cloned();

    // Read input from stdin only when no file path is provided.
    let input_content = if input_file.is_none() {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        String::new()
    };

    let source_map_enabled = matches.get_flag("source-map");
    let source_map_file = if source_map_enabled {
        matches
            .get_one::<String>("source-map-file")
            .cloned()
            .or_else(|| output_file.as_ref().map(|out| format!("{}.map", out)))
    } else {
        None
    };
    let source_map_url = if source_map_enabled {
        matches
            .get_one::<String>("source-map-url")
            .cloned()
            .or_else(|| {
                source_map_file
                    .as_ref()
                    .map(|path| default_source_map_url(path))
            })
    } else {
        None
    };

    // Compile LESS to CSS
    let mut compiler = if matches.get_flag("compress") {
        Compiler::compressed()
    } else {
        Compiler::new()
    };

    if source_map_enabled {
        compiler = compiler.with_source_map(true);
        if let Some(source_root) = matches.get_one::<String>("source-map-root") {
            compiler.set_source_map_source_root(Some(source_root.clone()));
        }
        compiler.set_source_map_lessjs_compat(matches.get_flag("source-map-lessjs-compat"));
        if let Some(output) = &output_file {
            compiler.set_source_map_file(Some(output.clone()));
        }
    }

    // Add include paths
    if let Some(paths) = matches.get_many::<String>("include-path") {
        for path in paths {
            compiler.add_include_path(path);
        }
    }

    // Compile from file or stdin
    let css_output = if let Some(input_path) = &input_file {
        compiler.compile_file(input_path)?
    } else {
        compiler.compile(&input_content)?
    };

    // Write output
    if let Some(output) = &output_file {
        let mut final_css = css_output;

        // Write source map sidecar and append sourceMappingURL when possible.
        if source_map_enabled {
            if let Some(sm_json) = compiler.generate_source_map() {
                if let Some(sm_file) = &source_map_file {
                    fs::write(sm_file, sm_json)?;
                    let mapping_url = source_map_url
                        .clone()
                        .unwrap_or_else(|| default_source_map_url(sm_file));
                    if !final_css.ends_with('\n') {
                        final_css.push('\n');
                    }
                    final_css.push_str(&format!("/*# sourceMappingURL={} */\n", mapping_url));
                    eprintln!("Source map written to {}", sm_file);
                } else {
                    eprintln!(
                        "source map generation enabled but no output file was resolved; skipping map file write"
                    );
                }
            }
        }

        fs::write(output, final_css)?;
        eprintln!("CSS written to {}", output);
    } else {
        print!("{}", css_output);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::default_source_map_url;

    #[test]
    fn test_default_source_map_url_uses_file_name() {
        let url = default_source_map_url("/tmp/dist/styles.css.map");
        assert_eq!(url, "styles.css.map");
    }

    #[test]
    fn test_default_source_map_url_plain_path() {
        let url = default_source_map_url("styles.css.map");
        assert_eq!(url, "styles.css.map");
    }
}
