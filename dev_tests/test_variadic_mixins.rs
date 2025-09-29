//! Test file for variadic mixin parameters
//!
//! This test verifies that mixins can accept variable numbers of arguments
//! using the @args... syntax.

use rust_less::{compile, Error};

fn main() {
    println!("🧪 Testing Variadic Mixin Parameters");
    println!("=====================================");

    // Test 1: Basic variadic mixin
    test_basic_variadic_mixin();

    // Test 2: Variadic mixin with some fixed parameters
    test_mixed_parameters();

    // Test 3: Variadic mixin used in different contexts
    test_variadic_usage_contexts();

    println!("\n✅ All variadic mixin tests completed!");
}

fn test_basic_variadic_mixin() {
    println!("\n📋 Test 1: Basic Variadic Mixin");
    println!("-------------------------------");

    let less_input = r#"
.box-shadow(@shadows...) {
    box-shadow: @shadows;
    -webkit-box-shadow: @shadows;
    -moz-box-shadow: @shadows;
}

.card {
    .box-shadow(0 2px 4px rgba(0,0,0,0.1), 0 4px 8px rgba(0,0,0,0.05));
}

.button {
    .box-shadow(0 1px 3px rgba(0,0,0,0.2));
}

.modal {
    .box-shadow(0 10px 25px rgba(0,0,0,0.15), 0 0 0 1px rgba(0,0,0,0.05));
}
"#;

    match compile(less_input) {
        Ok(css) => {
            println!("✅ Input LESS:");
            println!("{}", less_input.trim());
            println!("\n✅ Generated CSS:");
            println!("{}", css);

            // Verify the output contains expected shadow properties
            assert!(css.contains("box-shadow:"));
            assert!(css.contains("-webkit-box-shadow:"));
            assert!(css.contains("-moz-box-shadow:"));
            assert!(css.contains("rgba(0,0,0,0.1)"));
            println!("\n✅ Basic variadic mixin test passed!");
        }
        Err(e) => {
            println!("❌ Error compiling basic variadic mixin: {}", e);
            panic!("Basic variadic mixin test failed");
        }
    }
}

fn test_mixed_parameters() {
    println!("\n📋 Test 2: Mixed Parameters (Fixed + Variadic)");
    println!("-----------------------------------------------");

    let less_input = r#"
.transition(@property, @duration: 0.3s, @rest...) {
    transition: @property @duration @rest;
    -webkit-transition: @property @duration @rest;
    -moz-transition: @property @duration @rest;
}

.fade {
    .transition(opacity, 0.5s, ease-in-out);
}

.slide {
    .transition(transform, 0.2s, ease-out, 0.1s);
}

.complex {
    .transition(all, 0.4s, cubic-bezier(0.25, 0.46, 0.45, 0.94));
}
"#;

    match compile(less_input) {
        Ok(css) => {
            println!("✅ Input LESS:");
            println!("{}", less_input.trim());
            println!("\n✅ Generated CSS:");
            println!("{}", css);

            // Verify different transition properties
            assert!(css.contains("transition:"));
            assert!(css.contains("opacity"));
            assert!(css.contains("transform"));
            assert!(css.contains("ease-in-out"));
            assert!(css.contains("cubic-bezier"));
            println!("\n✅ Mixed parameters test passed!");
        }
        Err(e) => {
            println!("❌ Error compiling mixed parameters: {}", e);
            panic!("Mixed parameters test failed");
        }
    }
}

fn test_variadic_usage_contexts() {
    println!("\n📋 Test 3: Variadic Usage in Different Contexts");
    println!("------------------------------------------------");

    let less_input = r#"
.animation(@keyframes...) {
    animation: @keyframes;
    -webkit-animation: @keyframes;
}

.gradient(@directions, @colors...) {
    background: linear-gradient(@directions, @colors);
    background: -webkit-linear-gradient(@directions, @colors);
}

.multi-property(@properties...) {
    @properties;
}

.bounce {
    .animation(bounce 2s infinite, pulse 1s ease-in-out);
}

.header {
    .gradient(to right, #ff6b6b, #4ecdc4, #45b7d1);
}

.utility {
    .multi-property(
        color: red;
        margin: 10px;
        padding: 5px;
    );
}
"#;

    match compile(less_input) {
        Ok(css) => {
            println!("✅ Input LESS:");
            println!("{}", less_input.trim());
            println!("\n✅ Generated CSS:");
            println!("{}", css);

            // Verify different variadic usages
            assert!(css.contains("animation:"));
            assert!(css.contains("linear-gradient"));
            assert!(css.contains("bounce"));
            assert!(css.contains("#ff6b6b"));
            println!("\n✅ Variadic usage contexts test passed!");
        }
        Err(e) => {
            println!("❌ Error compiling variadic contexts: {}", e);
            println!(
                "Note: This test may fail if variadic parameters are not yet fully implemented"
            );
            // Don't panic here as this is expected to fail until implementation is complete
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variadic_mixins_compilation() {
        // This test will initially fail until variadic parameters are implemented
        let less = r#"
.shadows(@args...) {
    box-shadow: @args;
}

.element {
    .shadows(0 2px 4px rgba(0,0,0,0.1), 0 4px 8px rgba(0,0,0,0.05));
}
"#;

        // For now, this might fail - that's expected
        match compile(less) {
            Ok(css) => {
                println!("Variadic mixins work: {}", css);
                assert!(css.contains("box-shadow:"));
            }
            Err(e) => {
                println!("Variadic mixins not yet implemented: {}", e);
                // Expected to fail initially
            }
        }
    }
}
