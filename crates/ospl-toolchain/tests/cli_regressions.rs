use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_ospt")
}

fn temp_dir(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock went backwards")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("ospl-{name}-{suffix}"));
    fs::create_dir_all(&path).expect("failed to create temp test directory");
    path
}

fn run(args: &[&str], cwd: &Path) -> std::process::Output {
    Command::new(bin())
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("failed to run ospt")
}

fn write_binary_package(dir: &Path, source: &str) {
    fs::write(
        dir.join("package.kdl"),
        r#"name: app
version: 1.0.0
entry: bin

includes:
  bin:
    at: !File "main.ospl"

requires: []
"#,
    )
    .expect("failed to write package.kdl");

    fs::write(dir.join("main.ospl"), source).expect("failed to write main.ospl");
}

#[test]
fn binary_package_can_build_exec_and_scratch_run() {
    let dir = temp_dir("binary-happy-path");
    write_binary_package(&dir, "def x = 1;\n");

    let build = run(&["build"], &dir);
    assert!(
        build.status.success(),
        "build failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
    assert!(dir.join("build/dist.ospb").is_file());

    let exec = run(&["exec", "build/dist.ospb"], &dir);
    assert!(
        exec.status.success(),
        "exec failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&exec.stdout),
        String::from_utf8_lossy(&exec.stderr)
    );

    let scratch = run(&["scratch-run"], &dir);
    assert!(
        scratch.status.success(),
        "scratch-run failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&scratch.stdout),
        String::from_utf8_lossy(&scratch.stderr)
    );
}

#[test]
fn local_module_can_be_required_by_entry_module() {
    let dir = temp_dir("local-module");
    fs::write(
        dir.join("package.kdl"),
        r#"name: app
version: 1.0.0
entry: bin

includes:
  bin:
    at: !File "main.ospl"
    require:
      lib: !Local "lib"
  lib:
    at: !File "lib.ospl"

requires: []
"#,
    )
    .expect("failed to write package.kdl");
    fs::write(
        dir.join("main.ospl"),
        "def y = lib.x;\n",
    )
    .expect("failed to write main.ospl");
    fs::write(dir.join("lib.ospl"), "def x = 42;\n").expect("failed to write lib.ospl");

    let scratch = run(&["scratch-run"], &dir);
    assert!(
        scratch.status.success(),
        "scratch-run failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&scratch.stdout),
        String::from_utf8_lossy(&scratch.stderr)
    );
}

#[test]
fn declared_c_extension_can_be_loaded_and_called() {
    let dir = temp_dir("c-extension");
    fs::write(dir.join("ffi.c"), "int one(void) { return 1; }\n")
        .expect("failed to write ffi.c");
    fs::write(
        dir.join("package.kdl"),
        r#"name: cext
version: 1.0.0
entry: bin

includes:
  bin:
    at: !File "main.ospl"
    extensions:
      mylib: "ffi.c"

requires: []
"#,
    )
    .expect("failed to write package.kdl");
    fs::write(
        dir.join("main.ospl"),
        r#"def one = foreign fn mylib "one" i32 {};
do foreign one();
"#,
    )
    .expect("failed to write main.ospl");

    let scratch = run(&["scratch-run"], &dir);
    assert!(
        scratch.status.success(),
        "scratch-run failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&scratch.stdout),
        String::from_utf8_lossy(&scratch.stderr)
    );
}

#[test]
#[ignore = "bug: new currently panics while creating .gitignore/package.kdl"]
fn new_binary_package_succeeds_and_builds_without_manual_files() {
    let dir = temp_dir("new-binary-scaffold");
    let created = run(&["new", "hello", "-k", "binary"], &dir);
    assert!(
        created.status.success(),
        "new failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&created.stdout),
        String::from_utf8_lossy(&created.stderr)
    );
    assert!(dir.join("hello/.gitignore").is_file());
    assert!(dir.join("hello/main.ospl").is_file());

    let build = run(&["build"], &dir.join("hello"));
    assert!(
        build.status.success(),
        "generated binary package should build\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
}

#[test]
#[ignore = "bug: default_config.kdl does not match the current PackageSetup schema"]
fn default_config_template_can_be_loaded_as_a_package() {
    let template = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/default_config.kdl"),
    )
    .expect("failed to read default config template");
    let rendered = template
        .replacen("{}", "hello", 1)
        .replacen("{}", "bin", 1)
        .replace("{{}}", "{}");

    let dir = temp_dir("default-config-template");
    fs::write(dir.join("package.kdl"), rendered).expect("failed to write rendered package.kdl");
    fs::write(dir.join("main.ospl"), "def x = 1;\n").expect("failed to write main.ospl");

    let build = run(&["build"], &dir);
    assert!(
        build.status.success(),
        "rendered default config should build\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
}

#[test]
#[ignore = "bug: checked-in sample programs use top-level block syntax rejected by parse_file"]
fn checked_in_sample_programs_compile_as_binary_sources() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let samples = repo_root.join("test suite");

    for entry in fs::read_dir(samples).expect("failed to read test suite") {
        let entry = entry.expect("failed to read sample entry");
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("ospl") {
            continue;
        }

        let dir = temp_dir("sample-program");
        write_binary_package(
            &dir,
            &fs::read_to_string(entry.path()).expect("failed to read sample source"),
        );
        let build = run(&["build"], &dir);
        assert!(
            build.status.success(),
            "sample {:?} should compile\nstdout:\n{}\nstderr:\n{}",
            entry.file_name(),
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
#[ignore = "bug: foreign use only accepts declared extension ids, not system library paths"]
fn foreign_use_can_load_system_library_path() {
    let dir = temp_dir("system-ffi");
    write_binary_package(
        &dir,
        r#"def libc = foreign use "libc.so.6";
def puts = foreign fn libc "puts" i32 { ptr };
do foreign puts("hello from ffi");
"#,
    );

    let scratch = run(&["scratch-run"], &dir);
    assert!(
        scratch.status.success(),
        "system FFI library path should load\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&scratch.stdout),
        String::from_utf8_lossy(&scratch.stderr)
    );
}

#[test]
#[ignore = "bug: build clears build/dist.ospb before validating the next build"]
fn failed_build_preserves_last_successful_dist_file() {
    let dir = temp_dir("failed-build-preserves-output");
    write_binary_package(&dir, "def x = 1;\n");

    let first = run(&["build"], &dir);
    assert!(first.status.success());
    assert!(dir.join("build/dist.ospb").is_file());

    fs::write(
        dir.join("package.kdl"),
        r#"name: app
version: 1.0.0
entry: bin

includes:
  bin:
    at: !File "missing.ospl"

requires: []
"#,
    )
    .expect("failed to write invalid package.kdl");

    let second = run(&["build"], &dir);
    assert!(!second.status.success(), "invalid package unexpectedly built");
    assert!(
        dir.join("build/dist.ospb").is_file(),
        "failed build should preserve last successful dist file"
    );
}
