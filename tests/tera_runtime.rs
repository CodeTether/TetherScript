use tetherscript::compiler::Compiler;
use tetherscript::interp::Interpreter;
use tetherscript::lexer::Lexer;
use tetherscript::parser::Parser;
use tetherscript::vm::VM;

const SOURCE: &str = r#"
fn main() {
    let context = map()
    context.name = "<Riley>"
    context.items = ["one", "two"]
    let template = "\{% for item in items %\}\{\{ item | upper \}\} \{% endfor %\}\{\{ name \}\}"
    assert(tera_render(template, context).unwrap() == "ONE TWO &lt;Riley&gt;", "escaped render failed")
    assert(tera_render(template, context, false).unwrap() == "ONE TWO <Riley>", "raw render failed")
    assert(tera_render("\{\{", context).is_err(), "invalid template should fail")
    let option_error = tera_render(template, context, 1).err()
    assert(option_error.contains("autoescape must be bool"), "option error missing detail")
    let invalid = map()
    invalid.callback = main
    assert(tera_render("", invalid).err().contains("context.callback"), "path missing")
}
"#;

fn program() -> tetherscript::ast::Program {
    let tokens = Lexer::new(SOURCE).tokenize().unwrap();
    Parser::new(tokens).parse_program().unwrap()
}

#[test]
fn renders_in_reference_interpreter() {
    Interpreter::new().run(&program()).unwrap();
}

#[test]
fn renders_in_bytecode_vm() {
    let chunk = Compiler::compile_program(&program());
    VM::new().run(chunk).unwrap();
}
