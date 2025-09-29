//! Simple test for variadic mixin parameters

use rust_less::compile;

fn main() {
    println!("Testing variadic mixin parameters...");

    let less_input = r#"
    .box-shadow(@shadows...) {
        box-shadow: @shadows;
        -webkit-box-shadow: @shadows;
    }

    .animation(@keyframes...) {
        animation: @keyframes;
    }

    .simple {
        .box-shadow(0px 2px 4px black, 0px 4px 8px gray);
    }

    .bounce {
        .animation(bounce 2s infinite, pulse 1s ease-in-out);
    }

    .single {
        .box-shadow(0px 1px 3px red);
    }
    "#;

    match compile(less_input) {
        Ok(css) => {
            println!("Success! Generated CSS:");
            println!("{}", css);
        }
        Err(e) => {
            println!("Error: {}", e);
            println!("This is expected - variadic parameters not yet implemented");
        }
    }
}
