use std::{fs, path::PathBuf};
use ospl_common::inst::optimized::Inst;
use ospl_vm::VM;
use std::io::Read;
use clap::{Parser, Subcommand, ValueEnum};

use crate::graph::resolv::RecursionInfo;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Build the project in the current directory
    #[command(alias = "b")]
    Build,

    /// Rebuild and run the project in the current directory
    #[command(alias = "sr")]
    ScratchRun,

    /// Run a bytecode file
    #[command(alias = "e")]
    Exec {
        at: PathBuf,
    },

    New {
        name: String,

        #[arg(short = 'k', default_value_t = ProjTyp::Library)]
        kind: ProjTyp
    },

    #[command(alias = "dis")]
    Disassemble {
        file: PathBuf
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
pub mod graph;
pub mod util;
pub mod init;

const BUILD_FILE: &str = "build/dist.ospb";
const BUILD_FOLDER: &str = "build/";

pub fn ensure_build_folder() {
    let pb = PathBuf::from(BUILD_FOLDER);
    let _ = std::fs::remove_dir_all(&pb);

    std::fs::DirBuilder::new()
        .create(&pb)
        .expect("failed to create the build/ folder. Delete the folder and try again.");
}

fn main() {
    tracing_subscriber::fmt::init();

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
            init::cmd_new(name, kind.to_string());
        },
        Cmd::Disassemble { file } => cmd_disassemble(file)
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
    ospl_vm::debug::setup_debug_panic_handler();
    vm.run_all(&insts);
}

fn cmd_build(_o: PathBuf) {
    let root_pkg = load_package_yml_at("package.yml")
        .expect("you're not even in an OSPL project, there's no package.yml");

    let entry_name = root_pkg.entry.clone();
    ensure_build_folder();

    // high-level
    let mut gg = graph::resolv::HighGraph::default();
    let pi = RecursionInfo::default();

    graph::resolv::resolve_pkg(root_pkg, &mut gg, pi.clone());

    gg.main = graph::resolv2::get_package_module_with_name(&pi.pkg, &entry_name, &gg.module_index);
    LogState!(&"");
    Log!(Setting, "entry point to {}", gg.main);

    // low-level
    let low = graph::resolv2::lower(gg);

    // compile
    let out = graph::build::compile(&low);

    // write out
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(_o)
        .expect("failed to open output file");

    postcard::to_io(&out, &mut f)
        .expect("failed to write dist.ospb");
}

/* ---------------------------------- */

pub fn load_package_yml_at<P: AsRef<std::path::Path>>(path: P) -> Result<graph::decl::PackageSetup, String> {
    let src = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    yaml_serde::from_str(&src).map_err(|e| e.to_string())
}

fn cmd_disassemble<P: AsRef<std::path::Path>>(path: P) {
    let f = std::fs::read(path)
        .expect("failed to read file for disasm");

    let p = postcard::from_bytes::<Vec<Inst>>(&f).expect("failed to disassemble file");

    for thing in p {
        println!("{thing}");
    }
}
