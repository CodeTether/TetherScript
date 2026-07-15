//! Help text for the inspect subcommand.

pub(crate) fn print() {
    println!("tetherscript inspect -- Inspect TetherScript source code");
    println!();
    println!("USAGE:");
    println!("    tetherscript inspect <mode> <file.tether>");
    println!();
    println!("MODES:");
    println!("    --tokens       Dump lexer tokens");
    println!("    --ast          Dump abstract syntax tree");
    println!("    --ir           Dump verified backend-independent Tether IR");
    println!("    --bytecode     Dump compiled bytecode");
    println!("    --bytecode-visual  Render annotated bytecode");
    println!();
    println!("EXAMPLES:");
    println!("    tetherscript inspect --tokens hello.tether");
    println!("    tetherscript inspect --ast hello.tether");
    println!("    tetherscript inspect --ir ir_arithmetic.tether");
    println!("    tetherscript inspect --bytecode hello.tether");
    println!("    tetherscript inspect --bytecode-visual hello.tether");
}
