
#[cfg(test)]
mod tests {
    use rust_less::Compiler;

    #[test]
    fn test_hsl_functions() {
        let mut compiler = Compiler::new();
        // Red is hue 0/360, saturation 100%, lightness 50%
        let css = compiler.compile(".test { color: hsl(0, 100%, 50%); }").unwrap();
        assert!(css.contains("#ff0000") || css.contains("#f00"), "hsl(0, 100%, 50%) should be red, got: {}", css);

        // Green is hue 120, saturation 100%, lightness 50%
        let css = compiler.compile(".test { color: hsl(120, 100%, 50%); }").unwrap();
        assert!(css.contains("#00ff00") || css.contains("#0f0"), "hsl(120, 100%, 50%) should be green, got: {}", css);
        
        // HSLA
        let css = compiler.compile(".test { color: hsla(240, 100%, 50%, 0.5); }").unwrap();
        assert!(css.contains("rgba(0, 0, 255, 0.5)"), "hsla(240, 100%, 50%, 0.5) failed, got: {}", css);
    }

    #[test]
    fn test_saturation_functions() {
        let mut compiler = Compiler::new();
        // desaturate red by 100% should be gray
        // hsl(0, 100%, 50%) -> desaturate 100% -> hsl(0, 0%, 50%) -> #808080
        let css = compiler.compile(".test { color: desaturate(#ff0000, 100%); }").unwrap();
        assert!(css.contains("#808080"), "desaturate(#ff0000, 100%) failed, got: {}", css);

        // saturate gray
        // hsl(0, 0%, 50%) -> saturate 100% -> hsl(0, 100%, 50%) -> #ff0000 (hue is lost in gray, strictly speaking, but let's see implementation)
        // Usually implementation preserves hue if possible or defaults to 0 (red)
    }

    #[test]
    fn test_fade_functions() {
        let mut compiler = Compiler::new();
        let css = compiler.compile(".test { color: fade(#ff0000, 50%); }").unwrap();
        assert!(css.contains("rgba(255, 0, 0, 0.5)"), "fade failed, got: {}", css);

        let css = compiler.compile(".test { color: fadeout(#ff0000, 10%); }").unwrap();
        // #ff0000 is alpha 1.0 -> fadeout 10% -> 0.9
        assert!(css.contains("rgba(255, 0, 0, 0.9)"), "fadeout failed, got: {}", css);
        
        let css = compiler.compile(".test { color: fadein(rgba(255, 0, 0, 0.5), 10%); }").unwrap();
        // alpha 0.5 -> fadein 10% -> 0.6
        assert!(css.contains("rgba(255, 0, 0, 0.6)"), "fadein failed, got: {}", css);
    }

    #[test]
    fn test_spin_function() {
        let mut compiler = Compiler::new();
        // Red (0) spin 180 -> Cyan (180) #00ffff
        let css = compiler.compile(".test { color: spin(#ff0000, 180); }").unwrap();
        assert!(css.contains("#00ffff") || css.contains("#0ff"), "spin failed, got: {}", css);
    }
    
    #[test]
    fn test_mix_function() {
        let mut compiler = Compiler::new();
        // Mix red and blue equally -> purple
        let css = compiler.compile(".test { color: mix(#ff0000, #0000ff); }").unwrap();
        // Red: 255, 0, 0
        // Blue: 0, 0, 255
        // Mix: 128, 0, 128 -> #800080
        assert!(css.contains("#800080"), "mix failed, got: {}", css);
    }
}
