//! Help text for `tetherscript run`.

pub(crate) fn print() {
    println!("tetherscript run -- Run a TetherScript program");
    println!();
    println!("USAGE:");
    println!("    tetherscript run [options] [file.tether|package-directory] [--] [args...]");
    println!("    With no target, the nearest tetherscript.json package is used.");
    println!();
    println!("OPTIONS:");
    for line in OPTIONS {
        println!("    {line}");
    }
    println!();
    println!("EXAMPLES:");
    for line in EXAMPLES {
        println!("    {line}");
    }
}

const OPTIONS: &[&str] = &[
    "--vm                    Use bytecode VM (default)",
    "--interp, --tree-walk    Use tree-walking interpreter for debugging",
    "--step-budget <n>       Set max execution steps (default: unlimited)",
    "--access-mode <mode>    restricted (default) or full",
    "--grant-fs <dir>        Grant filesystem capability scoped to <dir>",
    "--grant-provider <url>  Grant LLM provider capability",
    "--grant-provider-key <k> API key for the provider",
    "--grant-provider-vault <id> Load provider grant from Vault KV v2",
    "--grant-rpc <url>       Grant JSON-RPC capability (http://host:port)",
    "-h, --help              Print this help message",
];

const EXAMPLES: &[&str] = &[
    "tetherscript run hello.tether",
    "tetherscript run --access-mode full examples/agent_tui.tether",
    "tetherscript run --step-budget 100000 fib.tether",
    "tetherscript run --grant-fs . policy.tether",
    "tetherscript run --grant-provider http://localhost:11434 chat.tether",
    "tetherscript run --grant-provider-vault openai chat.tether",
    "tetherscript run --grant-rpc http://127.0.0.1:36627 agent.tether",
    "tetherscript run examples/cli_args.tether -- --name Riley",
];
