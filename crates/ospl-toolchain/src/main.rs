use std::{fs, path::PathBuf};
use ospl_common::inst::optimized::Inst;
use ospl_compiler::Compiler;
use ospl_vm::VM;
use tracing::info;
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
    Build {
        #[arg(short)]
        output: Option<PathBuf>,
    },

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
        Cmd::ScratchRun => {
            let pb = PathBuf::from(DEFAULT_BUILD_LOCATION);
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