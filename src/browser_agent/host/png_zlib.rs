//! Zlib stream encoding with uncompressed DEFLATE blocks.

pub(super) fn stored(input: &[u8]) -> Vec<u8> {
    let mut output = vec![0x78, 0x01];
    if input.is_empty() {
        block(&mut output, &[], true);
    }
    let chunks = input.chunks(u16::MAX as usize);
    let count = chunks.len();
    for (index, chunk) in chunks.enumerate() {
        block(&mut output, chunk, index + 1 == count);
    }
    output.extend_from_slice(&adler32(input).to_be_bytes());
    output
}

fn block(output: &mut Vec<u8>, data: &[u8], final_block: bool) {
    output.push(u8::from(final_block));
    let length = data.len() as u16;
    output.extend_from_slice(&length.to_le_bytes());
    output.extend_from_slice(&(!length).to_le_bytes());
    output.extend_from_slice(data);
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for byte in data {
        a = (a + u32::from(*byte)) % 65_521;
        b = (b + a) % 65_521;
    }
    (b << 16) | a
}
