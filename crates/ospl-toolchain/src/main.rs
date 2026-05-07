use std::fs;
use ospl_compiler::Compiler;
use tracing::info;

pub mod package;

fn main() {
    tracing_subscriber::fmt::init();

    let s = fs::read_to_string("package.yml").expect("package.yml not found in package folder");
    let cfg: package::PackageSetup = yaml_serde::from_str(&s).expect("failed to read package.yml");

    let pkgs = cfg.gensrc();
    let mut comp = Compiler::new(pkgs);
    let mut root = Vec::new();

    let b = &raw const *comp.get_module("binary")
        .expect("binary package not declared");

    comp.compile_block(unsafe {&(*b).ast}, &mut root).expect("failed to compile");

    info!("compilation successful!");

    let mut v = ospl_vm::VM::new();
    v.run_all(&root);
}