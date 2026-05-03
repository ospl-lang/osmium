use ospl_compiler::Compiler;
use ospl_parser::{lexer::lexer::Lexer, parse::{Parser, diag::print_diag}};
use ospl_vm::VM;

fn main() {
    let mut args = std::env::args();

    let fp = args.nth(1).expect("please specify a file path");
    let file = std::fs::read_to_string(&fp).expect("failed to open file");

    eprintln!("lexing: {}", &fp);
    let mut lex = Lexer::new(&file);
    let toks = lex.all_tokens();

    eprintln!("parsing: {}", &fp);
    let mut parser = Parser::new(&toks);
    let p = parser.parse_block().map_err(|e| {
        print_diag(&parser, e);
        std::process::exit(1);
    }).unwrap();

    eprintln!("building: {}", &fp);
    let mut compiler = Compiler::new();
    let mut root = Vec::new();
    compiler.compile_block(&p, &mut root).expect("failed to compile");

    eprintln!("RUNNING: {}", &fp);
    let mut vm = VM::new();
    vm.run_all(&root);

    println!("{vm:?}");
}
