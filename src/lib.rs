//! TetherScript library surface for embedding the language in host Rust projects.

pub mod ast;
pub mod bytecode;
pub mod capability;
pub mod compiler;
pub mod fs_cap;
pub mod http;
pub mod interp;
pub mod json;
pub mod lexer;
pub mod output;
pub mod parser;
pub mod plugin;
pub mod provider_cap;
pub mod rpc_cap;
pub mod smtp;
pub mod system;
pub mod token;
pub mod tls;
pub mod value;
pub mod vm;