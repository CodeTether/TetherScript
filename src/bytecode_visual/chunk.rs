//! Chunk-level visualization.

use std::fmt::Write;

use crate::bytecode::Chunk;

use super::instr;

pub(super) fn render(chunk: &Chunk) -> String {
    let mut out = String::from("bytecode visualizer\n");
    render_chunk(&mut out, "main", chunk, 0);
    out
}

fn render_chunk(out: &mut String, name: &str, chunk: &Chunk, depth: usize) {
    let pad = indent(depth);
    writeln!(out, "{pad}chunk {name}").unwrap();
    render_values(out, "const", &chunk.consts, depth + 1);
    render_names(out, &chunk.names, depth + 1);
    render_code(out, chunk, depth + 1);
    for (index, proto) in chunk.protos.iter().enumerate() {
        let title = proto.name.as_deref().unwrap_or("<anon>");
        let args = proto.params.join(", ");
        let name = format!("p{index:03} fn {title}({args})");
        render_chunk(out, &name, &proto.chunk, depth + 1);
    }
}

fn render_values(out: &mut String, label: &str, values: &[crate::value::Value], depth: usize) {
    let pad = indent(depth);
    writeln!(out, "{pad}{label}s ({})", values.len()).unwrap();
    for (index, value) in values.iter().enumerate() {
        writeln!(out, "{pad}  {label}{index:03} = {value}").unwrap();
    }
}

fn render_names(out: &mut String, names: &[String], depth: usize) {
    let pad = indent(depth);
    writeln!(out, "{pad}names ({})", names.len()).unwrap();
    for (index, name) in names.iter().enumerate() {
        writeln!(out, "{pad}  n{index:03} = {name}").unwrap();
    }
}

fn render_code(out: &mut String, chunk: &Chunk, depth: usize) {
    let pad = indent(depth);
    writeln!(out, "{pad}code ({})", chunk.code.len()).unwrap();
    for (pc, instruction) in chunk.code.iter().enumerate() {
        writeln!(out, "{pad}  {}", instr::render(pc, instruction)).unwrap();
    }
}

fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}
