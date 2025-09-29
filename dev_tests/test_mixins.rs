use rust_less::{compile, compile_with_options, CompileOptions};

fn main() {
    println!("🧪 Testing Mixin Functionality");
    println!("================================");

    // Test 1: Basic mixin definition and call
    test_basic_mixin();

    // Test 2: Mixin with parameters
    test_mixin_with_parameters();

    // Test 3: Mixin with default parameters
    test_mixin_with_defaults();

    // Test 4: Nested mixin calls
    test_nested_mixins();

    // Test 5: Mixin error handling
    test_mixin_errors();

    println!("\n✅ Mixin tests completed!");
}

fn test_basic_mixin() {
    println!("\n🔧 Test 1: Basic Mixin Definition and Call");

    let less_code = r#"
.border-radius {
    border-radius: 5px;
    -webkit-border-radius: 5px;
    -moz-border-radius: 5px;
}

.button {
    padding: 10px;
    .border-radius;
    background: #007cba;
}

.card {
    margin: 20px;
    .border-radius;
}
"#;

    match compile(less_code) {
        Ok(css) => {
            println!("✅ Basic mixin compilation successful!");
            println!("Generated CSS:");
            println!("{}", css);

            // Verify that the mixin was expanded
            assert!(css.contains("border-radius: 5px"));
            assert!(css.contains("-webkit-border-radius: 5px"));
            assert!(css.contains(".button"));
            assert!(css.contains(".card"));
        }
        Err(e) => {
            println!("❌ Basic mixin compilation failed: {}", e);
        }
    }
}

fn test_mixin_with_parameters() {
    println!("\n🔧 Test 2: Mixin with Parameters");

    let less_code = r#"
.border-radius(@radius) {
    border-radius: @radius;
    -webkit-border-radius: @radius;
    -moz-border-radius: @radius;
}

.button {
    .border-radius(10px);
    background: #007cba;
}

.small-button {
    .border-radius(3px);
    background: #28a745;
}
"#;

    match compile(less_code) {
        Ok(css) => {
            println!("✅ Parameterized mixin compilation successful!");
            println!("Generated CSS:");
            println!("{}", css);

            // Verify that parameters were substituted correctly
            assert!(css.contains("border-radius: 10px"));
            assert!(css.contains("border-radius: 3px"));
        }
        Err(e) => {
            println!("❌ Parameterized mixin compilation failed: {}", e);
        }
    }
}

fn test_mixin_with_defaults() {
    println!("\n🔧 Test 3: Mixin with Default Parameters");

    let less_code = r#"
.box-shadow(@x: 0, @y: 2px, @blur: 4px, @color: rgba(0,0,0,0.1)) {
    box-shadow: @x @y @blur @color;
    -webkit-box-shadow: @x @y @blur @color;
}

.card {
    .box-shadow(); // Use all defaults
    background: white;
}

.elevated-card {
    .box-shadow(2px, 4px, 8px); // Override some defaults
    background: white;
}

.custom-shadow {
    .box-shadow(1px, 1px, 2px, #333); // Override all
    background: white;
}
"#;

    match compile(less_code) {
        Ok(css) => {
            println!("✅ Default parameter mixin compilation successful!");
            println!("Generated CSS:");
            println!("{}", css);

            // Verify default values were used
            assert!(css.contains("box-shadow: 0 2px 4px"));
            assert!(css.contains("box-shadow: 2px 4px 8px"));
            assert!(css.contains("box-shadow: 1px 1px 2px #333"));
        }
        Err(e) => {
            println!("❌ Default parameter mixin compilation failed: {}", e);
        }
    }
}

fn test_nested_mixins() {
    println!("\n🔧 Test 4: Nested Mixin Calls");

    let less_code = r#"
.border-radius(@radius: 5px) {
    border-radius: @radius;
    -webkit-border-radius: @radius;
}

.box-shadow(@blur: 2px) {
    box-shadow: 0 2px @blur rgba(0,0,0,0.1);
}

.card-style {
    .border-radius(8px);
    .box-shadow(4px);
    padding: 20px;
}

.product-card {
    .card-style;
    background: white;
    border: 1px solid #ddd;
}
"#;

    match compile(less_code) {
        Ok(css) => {
            println!("✅ Nested mixin compilation successful!");
            println!("Generated CSS:");
            println!("{}", css);

            // Verify nested expansion worked
            assert!(css.contains("border-radius: 8px"));
            assert!(css.contains("box-shadow: 0 2px 4px"));
            assert!(css.contains("padding: 20px"));
        }
        Err(e) => {
            println!("❌ Nested mixin compilation failed: {}", e);
        }
    }
}

fn test_mixin_errors() {
    println!("\n🔧 Test 5: Mixin Error Handling");

    // Test undefined mixin
    let undefined_mixin = r#"
.button {
    .undefined-mixin();
    background: red;
}
"#;

    match compile(undefined_mixin) {
        Ok(_) => {
            println!("❌ Should have failed for undefined mixin");
        }
        Err(e) => {
            println!("✅ Correctly caught undefined mixin error: {}", e);
        }
    }

    // Test wrong number of arguments
    let wrong_args = r#"
.border-radius(@radius) {
    border-radius: @radius;
}

.button {
    .border-radius(5px, 10px); // Too many arguments
    background: blue;
}
"#;

    match compile(wrong_args) {
        Ok(_) => {
            println!("❌ Should have failed for wrong argument count");
        }
        Err(e) => {
            println!("✅ Correctly caught argument count error: {}", e);
        }
    }
}
