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

    // CREATE package.kdl
    Log!(Creating, "package.kdl");

    let mut yaml_path = path.clone();
    yaml_path.push("package.kdl");

    let mut f = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&yaml_path)
        .expect("failed to open package.kdl");

    writeln!(&mut f, include_str!("default_config.kdl"), name=name, kind=kind)
        .expect("failed to write default package.kdl");

    // CREATE main.ospl
    Log!(Creating, "main.ospl");
    let mut main_path = path.clone();
    main_path.push("main.ospl");

    let mut f = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&main_path)
        .expect("failed to open main.ospl");

    writeln!(&mut f, "def x = 1;")
        .expect("failed to write default main.ospl");

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
    gitignore_path.push(".gitignore");

    let mut f = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&gitignore_path)
        .expect("failed to open .gitignore");

    writeln!(&mut f, include_str!("default.gitignore"))
        .expect("failed to write default .gitignore");
}