use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn test_architecture_boundaries() {
    let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    
    check_no_adapters_import(&src_dir.join("domain"));
    check_no_adapters_import(&src_dir.join("application"));
    check_no_adapters_import(&src_dir.join("validation"));
    check_no_adapters_import(&src_dir.join("resolver"));
    check_no_adapters_import(&src_dir.join("syntax"));
    check_no_adapters_import(&src_dir.join("layout"));
}

fn check_no_adapters_import(dir: &Path) {
    if !dir.exists() {
        return;
    }
    for entry in fs::read_dir(dir).expect("failed to read dir") {
        let entry = entry.expect("failed to read entry");
        let path = entry.path();
        if path.is_dir() {
            check_no_adapters_import(&path);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let content = fs::read_to_string(&path).expect("failed to read file");
            for (i, line) in content.lines().enumerate() {
                if line.contains("crate::adapters") || line.contains("super::adapters") {
                    panic!(
                        "Architecture violation: `{}` imports adapters on line {}: {}",
                        path.display(),
                        i + 1,
                        line.trim()
                    );
                }
            }
        }
    }
}
