#[cfg(feature = "cli")]
mod cli_source_map {
    use sourcemap::SourceMap;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("rust_less_cli_sm_{}_{}", std::process::id(), nanos));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn test_cli_writes_external_source_map_and_comment() {
        let dir = temp_dir();
        let input = dir.join("input.less");
        let output = dir.join("out.css");
        let map = dir.join("out.css.map");

        fs::write(&input, ".box { color: red; }").expect("write input.less");

        let Ok(bin) = std::env::var("CARGO_BIN_EXE_rust-less") else {
            return;
        };
        let status = Command::new(bin)
            .arg(&input)
            .arg("-o")
            .arg(&output)
            .arg("--source-map")
            .arg("--source-map-file")
            .arg(&map)
            .arg("--source-map-root")
            .arg("/workspace/src")
            .status()
            .expect("run rust-less cli");
        assert!(status.success(), "CLI exited with status: {}", status);

        let css = fs::read_to_string(&output).expect("read output css");
        assert!(
            css.contains("sourceMappingURL=out.css.map"),
            "Expected sourceMappingURL comment in css, got: {}",
            css
        );

        let map_json = fs::read_to_string(&map).expect("read source map");
        let sm = SourceMap::from_slice(map_json.as_bytes()).expect("parse source map");
        let file = sm.get_file().expect("source map file field");
        assert!(
            file.ends_with("out.css"),
            "Expected map file field to end with out.css, got: {}",
            file
        );
        assert_eq!(sm.get_source_root(), Some("/workspace/src"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_cli_uses_custom_source_map_url() {
        let dir = temp_dir();
        let input = dir.join("input.less");
        let output = dir.join("out.css");

        fs::write(&input, ".box { color: red; }").expect("write input.less");

        let Ok(bin) = std::env::var("CARGO_BIN_EXE_rust-less") else {
            return;
        };
        let status = Command::new(bin)
            .arg(&input)
            .arg("-o")
            .arg(&output)
            .arg("--source-map")
            .arg("--source-map-url")
            .arg("/assets/out.css.map")
            .status()
            .expect("run rust-less cli");
        assert!(status.success(), "CLI exited with status: {}", status);

        let css = fs::read_to_string(&output).expect("read output css");
        assert!(
            css.contains("sourceMappingURL=/assets/out.css.map"),
            "Expected custom sourceMappingURL comment in css, got: {}",
            css
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_cli_lessjs_compat_source_map_rewrites_sources_and_names() {
        let dir = temp_dir();
        let nested_dir = dir.join("nested");
        fs::create_dir_all(&nested_dir).expect("create nested dir");

        let input = dir.join("entry.less");
        let imported = nested_dir.join("source.less");
        let output = dir.join("out.css");
        let map = dir.join("out.css.map");

        fs::write(&input, "@import \"nested/source.less\";").expect("write entry.less");
        fs::write(&imported, ".from-nested { color: #f00; }").expect("write source.less");

        let Ok(bin) = std::env::var("CARGO_BIN_EXE_rust-less") else {
            return;
        };
        let status = Command::new(bin)
            .arg(&input)
            .arg("-o")
            .arg(&output)
            .arg("--source-map")
            .arg("--source-map-file")
            .arg(&map)
            .arg("--source-map-root")
            .arg("/workspace/src")
            .arg("--source-map-lessjs-compat")
            .status()
            .expect("run rust-less cli");
        assert!(status.success(), "CLI exited with status: {}", status);

        let map_json = fs::read_to_string(&map).expect("read source map");
        let sm = SourceMap::from_slice(map_json.as_bytes()).expect("parse source map");

        assert_eq!(
            sm.get_source_root(),
            None,
            "less.js compat mode should not emit sourceRoot"
        );
        assert_eq!(
            sm.get_name_count(),
            0,
            "less.js compat mode should emit empty names array"
        );
        assert_eq!(sm.get_source_count(), 1, "Expected a single source entry");

        let source = sm.get_source(0).expect("expected source path");
        assert_eq!(
            source, "/workspace/src/nested/source.less",
            "Expected root-prefixed relative source path in less.js compat mode"
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
