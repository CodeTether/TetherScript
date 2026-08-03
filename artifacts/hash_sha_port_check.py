"""Line-for-line Python port of src/hash/*.rs, used to validate the Rust logic
without a compiler.

This is a verification harness, not shipped code. Every function below mirrors
the corresponding Rust function statement for statement so that a discrepancy in
constants, rotation amounts, or padding shows up as a digest mismatch against
hashlib.
"""

import hashlib
import hmac

M32 = (1 << 32) - 1
M64 = (1 << 64) - 1


def rotl(x, n, mask=M32, width=32):
    return ((x << n) | (x >> (width - n))) & mask


def rotr(x, n, mask=M64, width=64):
    return ((x >> n) | (x << (width - n))) & mask


# --- src/hash/pad.rs ---
def bit_len(byte_len):
    return byte_len * 8


# --- src/hash/pad_block.rs ---
def padded64(data):
    bits = bit_len(len(data))
    assert bits < (1 << 64)
    out = bytearray(data)
    out.append(0x80)
    while len(out) % 64 != 56:
        out.append(0)
    out += bits.to_bytes(8, "big")
    return bytes(out)


def padded128(data):
    bits = bit_len(len(data))
    out = bytearray(data)
    out.append(0x80)
    while len(out) % 128 != 112:
        out.append(0)
    out += bits.to_bytes(16, "big")
    return bytes(out)


# --- src/hash/sha1_block.rs ---
def sha1_schedule(chunk):
    w = [0] * 80
    for i in range(16):
        w[i] = int.from_bytes(chunk[4 * i:4 * i + 4], "big")
    for i in range(16, 80):
        w[i] = rotl(w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16], 1)
    return w


def sha1_round(i, b, c, d):
    if i <= 19:
        return ((b & c) | ((~b & M32) & d), 0x5A827999)
    if i <= 39:
        return (b ^ c ^ d, 0x6ED9EBA1)
    if i <= 59:
        return ((b & c) | (b & d) | (c & d), 0x8F1BBCDC)
    return (b ^ c ^ d, 0xCA62C1D6)


def sha1_compress(h, chunk):
    w = sha1_schedule(chunk)
    a, b, c, d, e = h
    for i, word in enumerate(w):
        f, k = sha1_round(i, b, c, d)
        temp = (rotl(a, 5) + f + e + k + word) & M32
        e, d, c, b, a = d, c, rotl(b, 30), a, temp
    for idx, value in enumerate([a, b, c, d, e]):
        h[idx] = (h[idx] + value) & M32


# --- src/hash/sha1.rs ---
SHA1_IV = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0]


def sha1(data):
    h = list(SHA1_IV)
    padded = padded64(data)
    for off in range(0, len(padded), 64):
        sha1_compress(h, padded[off:off + 64])
    return b"".join(x.to_bytes(4, "big") for x in h)


# --- src/hash/sha512_consts.rs (transcribed from the Rust table) ---
K512 = [
    0x428A2F98D728AE22, 0x7137449123EF65CD,
    0xB5C0FBCFEC4D3B2F, 0xE9B5DBA58189DBBC,
    0x3956C25BF348B538, 0x59F111F1B605D019,
    0x923F82A4AF194F9B, 0xAB1C5ED5DA6D8118,
    0xD807AA98A3030242, 0x12835B0145706FBE,
    0x243185BE4EE4B28C, 0x550C7DC3D5FFB4E2,
    0x72BE5D74F27B896F, 0x80DEB1FE3B1696B1,
    0x9BDC06A725C71235, 0xC19BF174CF692694,
    0xE49B69C19EF14AD2, 0xEFBE4786384F25E3,
    0x0FC19DC68B8CD5B5, 0x240CA1CC77AC9C65,
    0x2DE92C6F592B0275, 0x4A7484AA6EA6E483,
    0x5CB0A9DCBD41FBD4, 0x76F988DA831153B5,
    0x983E5152EE66DFAB, 0xA831C66D2DB43210,
    0xB00327C898FB213F, 0xBF597FC7BEEF0EE4,
    0xC6E00BF33DA88FC2, 0xD5A79147930AA725,
    0x06CA6351E003826F, 0x142929670A0E6E70,
    0x27B70A8546D22FFC, 0x2E1B21385C26C926,
    0x4D2C6DFC5AC42AED, 0x53380D139D95B3DF,
    0x650A73548BAF63DE, 0x766A0ABB3C77B2A8,
    0x81C2C92E47EDAEE6, 0x92722C851482353B,
    0xA2BFE8A14CF10364, 0xA81A664BBC423001,
    0xC24B8B70D0F89791, 0xC76C51A30654BE30,
    0xD192E819D6EF5218, 0xD69906245565A910,
    0xF40E35855771202A, 0x106AA07032BBD1B8,
    0x19A4C116B8D2D0C8, 0x1E376C085141AB53,
    0x2748774CDF8EEB99, 0x34B0BCB5E19B48A8,
    0x391C0CB3C5C95A63, 0x4ED8AA4AE3418ACB,
    0x5B9CCA4F7763E373, 0x682E6FF3D6B2B8A3,
    0x748F82EE5DEFB2FC, 0x78A5636F43172F60,
    0x84C87814A1F0AB72, 0x8CC702081A6439EC,
    0x90BEFFFA23631E28, 0xA4506CEBDE82BDE9,
    0xBEF9A3F7B2C67915, 0xC67178F2E372532B,
    0xCA273ECEEA26619C, 0xD186B8C721C0C207,
    0xEADA7DD6CDE0EB1E, 0xF57D4F7FEE6ED178,
    0x06F067AA72176FBA, 0x0A637DC5A2C898A6,
    0x113F9804BEF90DAE, 0x1B710B35131C471B,
    0x28DB77F523047D84, 0x32CAAB7B40C72493,
    0x3C9EBE0A15C9BEBC, 0x431D67C49C100D4C,
    0x4CC5D4BECB3E42B6, 0x597F299CFC657E2A,
    0x5FCB6FAB3AD6FAEC, 0x6C44198C4A475817,
]

IV512 = [
    0x6A09E667F3BCC908, 0xBB67AE8584CAA73B,
    0x3C6EF372FE94F82B, 0xA54FF53A5F1D36F1,
    0x510E527FADE682D1, 0x9B05688C2B3E6C1F,
    0x1F83D9ABFB41BD6B, 0x5BE0CD19137E2179,
]

IV384 = [
    0xCBBB9D5DC1059ED8, 0x629A292A367CD507,
    0x9159015A3070DD17, 0x152FECD8F70E5939,
    0x67332667FFC00B31, 0x8EB44A8768581511,
    0xDB0C2E0D64F98FA7, 0x47B5481DBEFA4FA4,
]


# --- src/hash/sha512_block.rs ---
def sha512_schedule(chunk):
    w = [0] * 80
    for i in range(16):
        w[i] = int.from_bytes(chunk[8 * i:8 * i + 8], "big")
    for i in range(16, 80):
        a = w[i - 15]
        b = w[i - 2]
        s0 = rotr(a, 1) ^ rotr(a, 8) ^ (a >> 7)
        s1 = rotr(b, 19) ^ rotr(b, 61) ^ (b >> 6)
        w[i] = (w[i - 16] + s0 + w[i - 7] + s1) & M64
    return w


def sha512_compress(h, chunk):
    w = sha512_schedule(chunk)
    a, b, c, d, e, f, g, hh = h
    for i, word in enumerate(w):
        s1 = rotr(e, 14) ^ rotr(e, 18) ^ rotr(e, 41)
        ch = (e & f) ^ ((~e & M64) & g)
        t1 = (hh + s1 + ch + K512[i] + word) & M64
        s0 = rotr(a, 28) ^ rotr(a, 34) ^ rotr(a, 39)
        t2 = (s0 + ((a & b) ^ (a & c) ^ (b & c))) & M64
        hh, g, f = g, f, e
        e = (d + t1) & M64
        d, c, b = c, b, a
        a = (t1 + t2) & M64
    for idx, value in enumerate([a, b, c, d, e, f, g, hh]):
        h[idx] = (h[idx] + value) & M64


# --- src/hash/sha512_core.rs ---
def sha512_digest(iv, data):
    h = list(iv)
    padded = padded128(data)
    for off in range(0, len(padded), 128):
        sha512_compress(h, padded[off:off + 128])
    return b"".join(x.to_bytes(8, "big") for x in h)


def sha512(data):
    return sha512_digest(IV512, data)


def sha384(data):
    return sha512_digest(IV384, data)[:48]


# --- src/hash/hmac_sha1.rs / hmac_sha512.rs ---
def hmac_generic(hash_fn, block, digest_len, key, message):
    buf = bytearray(block)
    if len(key) > block:
        buf[:digest_len] = hash_fn(key)
    else:
        buf[:len(key)] = key
    inner = bytes(byte ^ 0x36 for byte in buf) + message
    outer = bytes(byte ^ 0x5C for byte in buf) + hash_fn(inner)
    return hash_fn(outer)


def hmac_sha1(key, message):
    return hmac_generic(sha1, 64, 20, key, message)


def hmac_sha512(key, message):
    return hmac_generic(sha512, 128, 64, key, message)


def main():
    failures = 0
    for length in list(range(0, 400)) + [1000, 1_000_000]:
        data = b"a" * length
        for mine, name in ((sha1, "sha1"), (sha512, "sha512"), (sha384, "sha384")):
            want = hashlib.new(name, data).digest()
            if mine(data) != want:
                print(f"MISMATCH {name} at length {length}")
                failures += 1
    vectors = [
        b"",
        b"abc",
        b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
        (b"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmn"
         b"hijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu"),
        bytes(range(256)),
    ]
    for data in vectors:
        for mine, name in ((sha1, "sha1"), (sha512, "sha512"), (sha384, "sha384")):
            if mine(data) != hashlib.new(name, data).digest():
                print(f"MISMATCH {name} on vector len {len(data)}")
                failures += 1
    for key_len in list(range(0, 200)) + [255, 1000]:
        key = b"\xaa" * key_len
        for msg in (b"", b"boundary", b"x" * 300):
            if hmac_sha1(key, msg) != hmac.new(key, msg, hashlib.sha1).digest():
                print(f"MISMATCH hmac_sha1 key_len {key_len}")
                failures += 1
            if hmac_sha512(key, msg) != hmac.new(key, msg, hashlib.sha512).digest():
                print(f"MISMATCH hmac_sha512 key_len {key_len}")
                failures += 1
    print("failures:", failures)


if __name__ == "__main__":
    main()
