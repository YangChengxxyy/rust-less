# rust-less

一个用 Rust 编写的 LESS 到 CSS 解析器和转换器库。

## 功能特性

- LESS 解析
- CSS 转换
- 支持嵌套选择器
- 支持变量
- 支持媒体查询

## 安装

将下面的内容添加到你的 `Cargo.toml` 文件中：

```toml
[dependencies]
rust_less = "0.1.0"
```

## 使用方法

### 作为库使用

```rust
use rust_less::parse_less;

fn main() {
    let less_code = r#"
        @primary-color: #333;
        
        .container {
            @width: 80%;
            width: @width;
            background-color: @primary-color;
            
            .header {
                color: blue;
            }
        }
    "#;
    
    match parse_less(less_code) {
        Ok(css) => println!("转换后的 CSS: \n{}", css),
        Err(e) => eprintln!("转换出错: {}", e),
    }
}
```

### 作为命令行工具使用

```bash
# 安装命令行工具
cargo install rust-less --features cli

# 转换 LESS 文件为 CSS
rust-less style.less  # 将生成 style.css
```

## API 文档

### `parse_less(source: &str) -> Result<String, String>`

解析 LESS 字符串并转换为 CSS。

### `parse_less_file(file_path: &str) -> Result<String, String>`

从文件中解析 LESS 并转换为 CSS。

## 许可证

MIT