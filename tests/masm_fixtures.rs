//! MASM fixture tests — validates generated .asm output against golden files.
//! Optionally assembles with ml64.exe if available on PATH.

use std::path::PathBuf;
use std::process::Command;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn asm_bib_exe() -> PathBuf {
    let mut p = project_root();
    p.push("target");
    let release = p.join("release").join("asm-bib.exe");
    if release.exists() {
        return release;
    }
    p.join("debug").join("asm-bib.exe")
}

fn run_asm_bib(pasm_file: &str, format: &str) -> (String, String, i32) {
    let fixture_dir = project_root().join("tests").join("fixtures");
    let input = fixture_dir.join(pasm_file);

    let output = Command::new(asm_bib_exe())
        .arg(input.to_str().unwrap())
        .arg(format)
        .output()
        .expect("failed to execute asm-bib");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);

    (stdout, stderr, code)
}

fn load_expected(expected_file: &str) -> String {
    let path = project_root().join("tests").join("fixtures").join(expected_file);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read expected file {:?}: {}", path, e))
}

fn normalize(s: &str) -> String {
    s.lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn assert_fixture(pasm: &str, expected_file: &str) {
    let (stdout, stderr, code) = run_asm_bib(pasm, "--masm");

    assert_eq!(code, 0, "asm-bib failed for {}: {}", pasm, stderr);

    let expected = load_expected(expected_file);
    let got = normalize(&stdout);
    let exp = normalize(&expected);

    if got != exp {
        eprintln!("=== FIXTURE MISMATCH: {} ===", pasm);
        let got_lines: Vec<&str> = got.lines().collect();
        let exp_lines: Vec<&str> = exp.lines().collect();
        let max = got_lines.len().max(exp_lines.len());
        for i in 0..max {
            let g = got_lines.get(i).unwrap_or(&"<missing>");
            let e = exp_lines.get(i).unwrap_or(&"<missing>");
            if g != e {
                eprintln!("  line {}: ", i + 1);
                eprintln!("    got:      {:?}", g);
                eprintln!("    expected: {:?}", e);
            }
        }
        panic!("fixture mismatch for {}", pasm);
    }
}

fn try_assemble_ml64(pasm: &str) {
    let fixture_dir = project_root().join("tests").join("fixtures");
    let input = fixture_dir.join(pasm);
    let asm_out = fixture_dir.join(format!("{}.test.asm", pasm.replace(".pasm", "")));
    let obj_out = fixture_dir.join(format!("{}.test.obj", pasm.replace(".pasm", "")));

    let output = Command::new(asm_bib_exe())
        .arg(input.to_str().unwrap())
        .arg("--masm")
        .arg("-o")
        .arg(asm_out.to_str().unwrap())
        .output()
        .expect("failed to execute asm-bib");

    assert!(output.status.success(), "asm-bib -o failed for {}", pasm);

    let ml64 = Command::new("ml64.exe")
        .arg("/c")
        .arg("/nologo")
        .arg(format!("/Fo{}", obj_out.to_str().unwrap()))
        .arg(asm_out.to_str().unwrap())
        .output();

    match ml64 {
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            let stdout = String::from_utf8_lossy(&result.stdout);
            if result.status.success() {
                eprintln!("[ML64 OK] {} assembled successfully", pasm);
            } else {
                eprintln!("[ML64 FAIL] {}: {}{}", pasm, stdout, stderr);
            }
            let _ = std::fs::remove_file(&asm_out);
            let _ = std::fs::remove_file(&obj_out);
        }
        Err(_) => {
            eprintln!("[ML64 SKIP] ml64.exe not found on PATH — skipping assembly test for {}", pasm);
            let _ = std::fs::remove_file(&asm_out);
        }
    }
}

// ─── Fixture tests ──────────────────────────────────────────────────────

#[test]
fn fixture_arithmetic() {
    assert_fixture("arithmetic.pasm", "arithmetic.expected.asm");
}

#[test]
fn fixture_hello() {
    assert_fixture("hello.pasm", "hello.expected.asm");
}

#[test]
fn fixture_memory_labels() {
    assert_fixture("memory_labels.pasm", "memory_labels.expected.asm");
}

#[test]
fn fixture_control_flow() {
    assert_fixture("control_flow.pasm", "control_flow.expected.asm");
}

#[test]
fn fixture_win32() {
    assert_fixture("win32.pasm", "win32.expected.asm");
}

#[test]
fn fixture_rep_string() {
    assert_fixture("rep_string.pasm", "rep_string.expected.asm");
}

#[test]
fn fixture_floats() {
    assert_fixture("floats.pasm", "floats.expected.asm");
}

// ─── ML64 assembly tests (optional) ────────────────────────────────────

#[test]
fn ml64_arithmetic() {
    try_assemble_ml64("arithmetic.pasm");
}

#[test]
fn ml64_hello() {
    try_assemble_ml64("hello.pasm");
}

#[test]
fn ml64_control_flow() {
    try_assemble_ml64("control_flow.pasm");
}

#[test]
fn ml64_rep_string() {
    try_assemble_ml64("rep_string.pasm");
}
