use ospl_compiler::Compiler;
use ospl_parser::{lexer::lexer::Lexer, parse::Parser};
use ospl_vm::VM;
use tracing::{info, info_span};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mut args = std::env::args();

    let fp = args.nth(1).expect("please specify a file path");
    let file = std::fs::read_to_string(&fp).expect("failed to open file");


    let toks = {
        let s = info_span!("lexing", file=fp);
        let _enter = s.enter();
        info!("beginning lexing");

        let mut lex = Lexer::new(&file);
        let toks = lex.all_tokens();

        toks
    };

    let p = {
        let s = info_span!("parsing", file=fp);
        let _enter = s.enter();
        info!("beginning parsing");

        let mut parser = Parser::new(&toks);
        let p = parser.parse_block()
            .unwrap();
            /*.map_err(|e| {
                print_diag(&parser, e);
            }).unwrap();*/

        p
    };

    let root = {
        let s = info_span!("compiling", file=fp);
        let _enter = s.enter();
        info!("beginning compilation");

        let mut compiler = Compiler::new();
        let mut root = Vec::new();
        compiler.compile_block(&p, &mut root)
            .unwrap_or_else(|c| panic!("{:#?}", c));

        println!("{root:#?}");
        root
    };

    info!("RUNNING: {}", &fp);
    let mut vm = VM::new();
    vm.run_all(&root);

    info!("final VM: {:?}", vm);
}
