use std::{fs, path::PathBuf};
use ospl_common::inst::optimized::Inst;
use ospl_compiler::Compiler;
use ospl_vm::VM;
use tracing_subscriber::fmt::MakeWriter;
use std::io::{Write, Read};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Build the project in the current directory
    Build,

    /// Rebuild and run the project in the current directory
    ScratchRun,

    /// Run a bytecode file
    Exec {
        at: PathBuf,
    },

    New {
        name: String,

        #[arg(short = 'k', default_value_t = ProjTyp::Library)]
        kind: ProjTyp
    },
}

#[derive(Default, Copy, Clone, Debug, ValueEnum)]
enum ProjTyp {
    Binary,

    #[default]
    Library,
}

impl ToString for ProjTyp {
    fn to_string(&self) -> String {
        return match self {
            Self::Binary => "binary".to_string(),
            Self::Library => "library".to_string(),
        }
    }
}

pub mod log;
pub mod package;
pub mod util;
pub mod c_extension;

const BUILD_FILE: &str = "build/dist.ospb";
const C_EXT_FOLDER: &str = "build/c/";
const BUILD_FOLDER: &str = "build/";

fn ensure_build_folder() {
    let pb = PathBuf::from(BUILD_FOLDER);
    std::fs::remove_dir_all(&pb)
        .expect("Failed to remove the build/ folder. Delete the folder and try again.");

    std::fs::DirBuilder::new()
        .create(&pb)
        .expect("failed to create the build/ folder. Delete the folder and try again.");

    std::fs::DirBuilder::new()
        .create(PathBuf::from(C_EXT_FOLDER))
        .expect("failed to create the build/c/ folder. Delete the build/ folder and try again.");
}

fn main() {
    tracing_subscriber::fmt::init();

    ensure_build_folder();

    let cli = Cli::parse();
    match cli.command {
        Cmd::Build => {
            cmd_build(PathBuf::from(BUILD_FILE));
        }
        Cmd::Exec { at } => cmd_exec(at),
        Cmd::ScratchRun => {
            let pb = PathBuf::from(BUILD_FILE);
            // if !pb.exists() {
                // cmd_build(pb.clone());
            // }
            cmd_build(pb.clone());
            cmd_exec(pb);
        },
        Cmd::New { name, kind } => {
            cmd_new(name, kind.to_string());
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

fn cmd_new(name: String, kind: String) {
    let mut path = std::env::current_dir().expect("failed to get cwd");
    path.push(&name);
    fs::DirBuilder::new()
        .recursive(true)
        .create(&path)
        .expect("failed to create new package folder");

    path.push("package.yml");
    let mut f = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .expect("failed to open package.yml");

    writeln!(&mut f, include_str!("default_config_fstring"), name, kind)
        .expect("failed to write default package.yml");
}

fn cmd_build(output: PathBuf) {
    let s = fs::read_to_string("package.yml").expect("package.yml not found in package folder");
    let cfg: package::PackageSetup = yaml_serde::from_str(&s).expect("failed to read package.yml");

    let pkgs = cfg.gensrc();
    let mut comp = Compiler::new(pkgs);
    let mut root = Vec::new();

    let b = &raw const *comp.get_module("binary")
        .expect("binary package not declared");

    if let Err(e) = comp.compile_block(unsafe {&(*b).code}, &mut root) {
        panic!("failed to compile\n\n{e:#?}");
    }

    Log!(Finished, "build/dist.ospb is ready");

    let f = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(output)
        .expect("failed to open output file");

    postcard::to_io(&root, &mut f.make_writer())
        .expect("failed to save compiled program");
}