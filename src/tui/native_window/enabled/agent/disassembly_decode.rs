//! x86-64 text-section decoding and machine-byte formatting.

use iced_x86::{Decoder, DecoderOptions, Formatter, IntelFormatter};

pub(super) struct Row {
    pub address: u64,
    pub offset: u64,
    pub bytes: String,
    pub assembly: String,
}

pub(super) fn decode(data: &[u8], base: u64) -> Vec<Row> {
    let mut decoder = Decoder::with_ip(64, data, base, DecoderOptions::NONE);
    let mut formatter = IntelFormatter::new();
    let mut rows = Vec::new();
    while decoder.can_decode() && rows.len() < 2048 {
        let instruction = decoder.decode();
        let offset = instruction.ip().saturating_sub(base);
        let start = offset as usize;
        let end = start.saturating_add(instruction.len()).min(data.len());
        let mut assembly = String::new();
        formatter.format(&instruction, &mut assembly);
        rows.push(Row {
            address: instruction.ip(),
            offset,
            bytes: hex(&data[start..end]),
            assembly,
        });
    }
    rows
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}
