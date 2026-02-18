use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_less::Compiler;

const SIMPLE_LESS: &str = r#"
@primary: #333;
@font-size: 14px;

.header {
    color: @primary;
    font-size: @font-size;
}
"#;

const NESTING_LESS: &str = r#"
@color: #333;
.nav {
    background: white;
    ul {
        list-style: none;
        li {
            display: inline-block;
            a {
                color: @color;
                &:hover {
                    color: darken(@color, 20%);
                }
            }
        }
    }
}
"#;

const MIXIN_LESS: &str = r#"
.border-radius(@radius: 5px) {
    -webkit-border-radius: @radius;
    -moz-border-radius: @radius;
    border-radius: @radius;
}
.box-shadow(@x: 0, @y: 0, @blur: 1px, @color: #000) {
    -webkit-box-shadow: @x @y @blur @color;
    -moz-box-shadow: @x @y @blur @color;
    box-shadow: @x @y @blur @color;
}
.card {
    .border-radius(10px);
    .box-shadow(2px, 2px, 5px, rgba(0,0,0,0.3));
    padding: 20px;
}
.button {
    .border-radius(3px);
    .box-shadow(1px, 1px, 3px);
    display: inline-block;
}
"#;

const COMPLEX_LESS: &str = r#"
@primary: #4a90d9;
@secondary: #e74c3c;
@font-stack: 'Helvetica Neue', Helvetica, Arial, sans-serif;
@base-size: 16px;
@border-radius: 4px;

.clearfix() {
    &::after {
        content: "";
        display: table;
        clear: both;
    }
}
.text-ellipsis() {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.btn-variant(@bg, @color: #fff) {
    background-color: @bg;
    color: @color;
    border: 1px solid darken(@bg, 10%);
    &:hover {
        background-color: darken(@bg, 10%);
    }
}

body {
    font-family: @font-stack;
    font-size: @base-size;
    color: #333;
}
.container {
    .clearfix();
    max-width: 1200px;
    margin: 0 auto;
}
.header {
    background: @primary;
    color: #fff;
    padding: 20px;
    h1 {
        font-size: @base-size * 2;
        .text-ellipsis();
    }
}
.btn-primary { .btn-variant(@primary); }
.btn-danger { .btn-variant(@secondary); }
.sidebar {
    float: left;
    width: 25%;
    .nav-item {
        padding: 10px;
        border-bottom: 1px solid #eee;
        a {
            color: @primary;
            text-decoration: none;
            &:hover { color: darken(@primary, 20%); }
        }
    }
}
.content {
    float: left;
    width: 75%;
    padding: 20px;
    @media (max-width: 768px) {
        width: 100%;
        float: none;
    }
}
"#;

fn bench_simple(c: &mut Criterion) {
    c.bench_function("compile_simple", |b| {
        b.iter(|| {
            let mut compiler = Compiler::new();
            compiler.compile(black_box(SIMPLE_LESS)).unwrap()
        })
    });
}

fn bench_nesting(c: &mut Criterion) {
    c.bench_function("compile_nesting", |b| {
        b.iter(|| {
            let mut compiler = Compiler::new();
            compiler.compile(black_box(NESTING_LESS)).unwrap()
        })
    });
}

fn bench_mixins(c: &mut Criterion) {
    c.bench_function("compile_mixins", |b| {
        b.iter(|| {
            let mut compiler = Compiler::new();
            compiler.compile(black_box(MIXIN_LESS)).unwrap()
        })
    });
}

fn bench_complex(c: &mut Criterion) {
    c.bench_function("compile_complex", |b| {
        b.iter(|| {
            let mut compiler = Compiler::new();
            compiler.compile(black_box(COMPLEX_LESS)).unwrap()
        })
    });
}

fn bench_compressed(c: &mut Criterion) {
    c.bench_function("compile_compressed", |b| {
        b.iter(|| {
            let mut compiler = Compiler::compressed();
            compiler.compile(black_box(COMPLEX_LESS)).unwrap()
        })
    });
}

criterion_group!(
    benches,
    bench_simple,
    bench_nesting,
    bench_mixins,
    bench_complex,
    bench_compressed,
);
criterion_main!(benches);
