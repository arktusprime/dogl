use std::{
    env, fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const BASIC_LAYOUT_SOURCE: &str = include_str!("../crates/dogl-language/tests/fixtures/layout_basic_chain.dogl");
const INVALID_LAYOUT_SOURCE: &str =
    include_str!("../crates/dogl-language/tests/fixtures/validation_orphan_task.dogl");

#[test]
fn layout_cli_recomputes_layout_and_writes_file() {
    let path = write_temp_dogl("layout_cli_basic.dogl", BASIC_LAYOUT_SOURCE);

    let output = Command::new(env!("CARGO_BIN_EXE_dogl"))
        .arg("layout")
        .arg(&path)
        .output()
        .expect("run dogl layout command");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let content = fs::read_to_string(&path).expect("read output file");
    assert!(content.contains("\nlayout\n"));
    assert!(content.contains("[] Review"));
    assert!(content.contains("{"));
}

#[test]
fn layout_cli_does_not_write_partial_output_on_validation_failure() {
    let path = write_temp_dogl("layout_cli_invalid.dogl", INVALID_LAYOUT_SOURCE);
    let original = fs::read_to_string(&path).expect("read original file");

    let output = Command::new(env!("CARGO_BIN_EXE_dogl"))
        .arg("layout")
        .arg(&path)
        .output()
        .expect("run dogl layout command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("validation diagnostics:"));

    let after = fs::read_to_string(&path).expect("read failed output file");
    assert_eq!(after, original);
}

fn write_temp_dogl(name: &str, content: &str) -> PathBuf {
    let mut path = env::temp_dir();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    path.push(format!("dogl_cli_{unique}_{name}"));
    fs::write(&path, content).expect("write temp dogl");
    path
}
