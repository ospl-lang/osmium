use std::fs;
use std::io::Write;

use crate::Log;

pub fn cmd_new(name: String, kind: String) {
    Log!(Creating, "new package '{name}'");

    // CREATE FOLDER
    let mut path = std::env::current_dir().expect("failed to get cwd");
    path.push(&name);
    fs::DirBuilder::new()
        .recursive(true)
        .create(&path)
        .expect("failed to create new package folder");

    // CREATE package.yml
    Log!(Creating, "package.yml");

    let mut yaml_path = path.clone();
    yaml_path.push("package.yml");

    let mut f = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&yaml_path)
        .expect("failed to open package.yml");

    writeln!(&mut f, include_str!("default_config.yml"), name, kind)
        .expect("failed to write default package.yml");

    // INIT REPO
    Log!(Invoking, "git init");

    std::process::Command::new("git")
        .arg("init")
        .arg("--quiet")
        .arg(&path)
        .spawn()
        .expect("failed to invoke git")
        .wait()
        .expect("failed to wait for git");

    // CREATE GITIGNORE
    Log!(Creating, ".gitignore");
    let mut gitignore_path = path.clone();
    gitignore_path.push("package.yml");

    let mut f = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&yaml_path)
        .expect("failed to open package.yml");

    writeln!(&mut f, include_str!("default.gitignore"))
        .expect("failed to write default .gitignore");
}