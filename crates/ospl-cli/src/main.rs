use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CLI {
    #[command(subcommand)]
    cmd: Command
}

#[derive(Subcommand, Clone)]
pub enum Command {
    ScratchRun {
        filepath: String
    },
    TestLoop,
}

fn main() {
    let cli = CLI::parse();
    match cli.cmd {
        Command::TestLoop => {
            ospl_vm::tests::loops::t_loops();
        },
        Command::ScratchRun { filepath } => {
            println!("parsing...");
            use ospl_parser::Parser;
            let input = std::fs::read_to_string(filepath).expect("file not found");
            let mut parser = Parser::new(&input);
            let file = parser.parse_file();

            // println!("file = {:?}", &file);

            println!("compiling...");
            use ospl_compiler::Compiler;
            let mut compiler = Compiler::new();
            let mut root = Vec::new();
            compiler.compile_stmt_list(&file.statements, &mut root).unwrap();

            println!("translating...");
            use ospl_vm::inst::translator;
            let translated = translator::vm_instructions_to_optimized(root);

            // println!("test: {:?}", &translated);

            println!("executing...");
            use ospl_vm::VM;
            let mut vm = VM::new();
            vm.run_all(&translated);

            println!("execution ended");
        },
    }
}
