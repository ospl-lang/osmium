use std::{fs, path::PathBuf};
use ospl_common::inst::optimized::Inst;
use ospl_toolchain::load_package_cfg;
use ospl_vm::VM;
use std::io::Read;
use clap::{Parser, Subcommand, ValueEnum};
use ospl_toolchain::{graph::resolv1::RecursionInfo, util::print_diag};

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

    #[command(alias = "r")]
    Run,

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

// This is my very cool comment
impl ToString for ProjTyp {
    fn to_string(&self) -> String {
        return match self {
            Self::Binary => "binary".to_string(),
            Self::Library => "library".to_string(),
        }
    }
}

pub fn ensure_build_folder() {
    let pb = PathBuf::from(ospl_toolchain::BUILD_FOLDER);
    let _ = std::fs::remove_file(pb.with_file_name("dist.ospb"));

    std::fs::DirBuilder::new()
        .recursive(true)
        .create(&pb)
        .expect("failed to create the build/ folder. Delete the folder and try again.");
}

fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    match cli.command {
        Cmd::Build => {
            cmd_build(PathBuf::from(ospl_toolchain::BUILD_FILE));
        },
        Cmd::Run => {
            let pb = PathBuf::from(ospl_toolchain::BUILD_FILE);
            cmd_exec(pb);
        },
        Cmd::Exec { at } => cmd_exec(at),
        Cmd::ScratchRun => {
            // build
            let bf = PathBuf::from(ospl_toolchain::BUILD_FILE);
            cmd_build(bf.clone());

            // cd into the build folder
            let pb2 = PathBuf::from(ospl_toolchain::BUILD_FOLDER);
            std::env::set_current_dir(pb2).expect("failed to cd into the build folder");
            cmd_exec(PathBuf::from("dist.ospb"));
        },
        Cmd::New { name, kind } => {
            ospl_toolchain::init::cmd_new(name, kind.to_string());
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

fn cmd_build(out_path: PathBuf) {
    let root_pkg = load_package_cfg("package.kdl").finalize();

    let entry_name = root_pkg.entry.clone();
    ensure_build_folder();

    // high-level
    let mut gg = ospl_toolchain::graph::resolv1::HighGraph::default();
    let pi = RecursionInfo::default();

    if let Err(e) = ospl_toolchain::graph::resolv1::resolve_pkg(root_pkg, &mut gg, pi.clone()) {
        panic!("Resolver error: {e:?}")
    }

    gg.main = ospl_toolchain::graph::resolv2::get_package_module_with_name(&pi.pkg, &entry_name, &gg.module_index);
    ospl_toolchain::LogState!(&"");

    // low-level
    let low = ospl_toolchain::graph::resolv2::lower(gg);

    // compile
    let out = ospl_toolchain::graph::build::genmods(&low);
    let out = ospl_toolchain::graph::build::buildmain(out);
    let out = match out {
        Ok(x) => x,
        Err(e) => {
            print_diag(e);
            std::process::exit(101);
        }
    };

    // write out
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(out_path)
        .expect("failed to open output file");

    postcard::to_io(&out, &mut f)
        .expect("failed to write dist.ospb");
}

/* ---------------------------------- */

fn cmd_disassemble<P: AsRef<std::path::Path>>(path: P) {
    let f = std::fs::read(path)
        .expect("failed to read file for disasm");

    let p = postcard::from_bytes::<Vec<Inst>>(&f).expect("failed to disassemble file");

    for thing in p {
        println!("{thing}");
    }
}
