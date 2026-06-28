//! Example lines for top-level CLI help.

pub(crate) const EXAMPLES: &[&str] = &[
    "tetherscript run hello.tether",
    "tetherscript run --interp fib.tether",
    "tetherscript run --access-mode full examples/agent_tui.tether",
    "tetherscript run --grant-fs . policy.tether",
    "tetherscript run --grant-provider http://localhost:11434 chat.tether",
    "tetherscript run --grant-provider-vault openai chat.tether",
    "tetherscript run --grant-provider https://api.cerebras.ai glm_chat.tether",
    "tetherscript run --grant-rpc http://127.0.0.1:36627 agent.tether",
    "tetherscript inspect --tokens hello.tether",
    "tetherscript inspect --ast hello.tether",
    "tetherscript inspect --bytecode hello.tether",
    "tetherscript render examples/browser.html examples/browser.css",
    "tetherscript raster examples/browser.html out.ppm examples/browser.css",
    "tetherscript js app.js",
    "tetherscript git",
    "tetherscript repl",
    "tetherscript lsp",
];
