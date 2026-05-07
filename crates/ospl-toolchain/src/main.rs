use std::{fs, path::PathBuf};
use ospl_common::inst::optimized::Inst;
use ospl_compiler::Compiler;
use ospl_vm::VM;
use tracing::info;
use tracing_subscriber::fmt::MakeWriter;
use std::io::Read;
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Build the project in the current directory
    Build {
        #[arg(short)]
        output: Option<PathBuf>,
    },

    /// Run the project in the current directory
    Run,

    /// Run a bytecode file
    Exec {
        at: PathBuf,
    },
}

pub mod package;

const DEFAULT_BUILD_LOCATION: &str = "dist.ospb";

fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    match cli.command {
        Cmd::Build { output } => {
            let outfile = match output {
                Some(outfile) => outfile,
                None => PathBuf::from(DEFAULT_BUILD_LOCATION)
            };
            cmd_build(outfile);
        }
        Cmd::Exec { at } => cmd_exec(at),
        Cmd::Run => {
            let pb = PathBuf::from(DEFAULT_BUILD_LOCATION);
            if !pb.exists() {
                cmd_build(pb.clone());
            }

            cmd_exec(pb);
        }
    };
}

fn cmd_exec(at: PathBuf) {
    let mut f = fs::OpenOptions::new()
        .create(false)
        .read(true)
        .open(at)
        .expect("failed to open bytecode file");

    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes).expect("failed to read file");

    let insts: Vec<Inst> = postcard::from_bytes(bytes.as_slice())
        .expect("failed to deserialize (is this program for an older OSPL version?)");

    let mut vm = VM::new();
    vm.run_all(&insts);
}

fn cmd_build(output: PathBuf) {
    let s = fs::read_to_string("package.yml").expect("package.yml not found in package folder");
    let cfg: package::PackageSetup = yaml_serde::from_str(&s).expect("failed to read package.yml");

    let pkgs = cfg.gensrc();
    let mut comp = Compiler::new(pkgs);
    let mut root = Vec::new();

    let b = &raw const *comp.get_module("binary")
        .expect("binary package not declared");

    comp.compile_block(unsafe {&(*b).ast}, &mut root).expect("failed to compile");

    info!("compilation successful!");

    let f = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(output)
        .expect("failed to open output file");

    postcard::to_io(&root, &mut f.make_writer())
        .expect("failed to save compiled program");
}