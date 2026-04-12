use std::{
    env, fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const BASIC_LAYOUT_SOURCE: &str =
    include_str!("../crates/dogl-language/tests/fixtures/layout_basic_chain.dogl");
const BOTTOM_LAYOUT_SOURCE: &str =
    include_str!("../crates/dogl-language/tests/fixtures/bottom_layout_block.dogl");

#[test]
fn export_bpmn_cli_writes_sibling_bpmn_file() {
    let path = write_temp_dogl("export_bpmn_cli_basic.dogl", BOTTOM_LAYOUT_SOURCE);
    let bpmn_path = path.with_extension("bpmn");

    let output = Command::new(env!("CARGO_BIN_EXE_dogl"))
        .arg("export-bpmn")
        .arg(&path)
        .output()
        .expect("run dogl export-bpmn command");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let content = fs::read_to_string(&bpmn_path).expect("read bpmn output");
    assert!(content.contains("<bpmn:definitions"));
    assert!(content.contains("<bpmndi:BPMNDiagram"));
}

#[test]
fn export_bpmn_cli_fails_when_layout_is_missing() {
    let path = write_temp_dogl("export_bpmn_cli_missing_layout.dogl", BASIC_LAYOUT_SOURCE);
    let bpmn_path = path.with_extension("bpmn");

    let output = Command::new(env!("CARGO_BIN_EXE_dogl"))
        .arg("export-bpmn")
        .arg(&path)
        .output()
        .expect("run dogl export-bpmn command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("requires an existing layout"));
    assert!(!bpmn_path.exists());
}

#[test]
fn build_dir_cli_updates_layout_and_writes_bpmn_for_all_dogl_files() {
    let dir = write_temp_dir("build_dir_cli");
    let root_dogl = dir.join("root.dogl");
    let nested_dir = dir.join("nested");
    fs::create_dir_all(&nested_dir).expect("create nested temp dir");
    let nested_dogl = nested_dir.join("child.dogl");

    fs::write(&root_dogl, BASIC_LAYOUT_SOURCE).expect("write root dogl");
    fs::write(&nested_dogl, BASIC_LAYOUT_SOURCE).expect("write nested dogl");

    let output = Command::new(env!("CARGO_BIN_EXE_dogl"))
        .arg("build-dir")
        .arg(&dir)
        .output()
        .expect("run dogl build-dir command");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let root_content = fs::read_to_string(&root_dogl).expect("read root dogl");
    let nested_content = fs::read_to_string(&nested_dogl).expect("read nested dogl");
    assert!(root_content.contains("layout"));
    assert!(nested_content.contains("layout"));

    let root_bpmn = fs::read_to_string(root_dogl.with_extension("bpmn")).expect("read root bpmn");
    let nested_bpmn =
        fs::read_to_string(nested_dogl.with_extension("bpmn")).expect("read nested bpmn");
    assert!(root_bpmn.contains("<bpmn:definitions"));
    assert!(nested_bpmn.contains("<bpmn:definitions"));
}

#[test]
fn build_dir_cli_fails_when_directory_has_no_dogl_files() {
    let dir = write_temp_dir("build_dir_cli_empty");

    let output = Command::new(env!("CARGO_BIN_EXE_dogl"))
        .arg("build-dir")
        .arg(&dir)
        .output()
        .expect("run dogl build-dir command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no `.dogl` files found"));
}

fn write_temp_dogl(name: &str, content: &str) -> PathBuf {
    let dir = write_temp_dir("dogl_cli_files");
    let mut path = dir;
    path.push(name);
    fs::write(&path, content).expect("write temp dogl");
    path
}

fn write_temp_dir(prefix: &str) -> PathBuf {
    let mut path = env::temp_dir();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    path.push(format!("dogl_cli_{unique}_{prefix}"));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}
