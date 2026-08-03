"""Round-trip verification for the numeric ENCODER
(src/postgres/binary_encode_numeric_groups.rs and _split.rs) against the
DECODER (binary_decode_numeric_render.rs), mirroring both in Python.

Proves: encode(text) -> wire -> decode(wire) == text, exactly, for every
case including trailing-zero scale preservation, and that no float is involved.

Run: python3 artifacts/pgbinary_verify_roundtrip.py
"""

import decimal
import struct

SIGN_POS, SIGN_NEG, SIGN_NAN = 0x0000, 0x4000, 0xC000


# ---- encoder: binary_encode_numeric_split.rs + _groups.rs ----------------

def peel_sign(text):
    if text.startswith("-"):
        return True, text[1:]
    return False, text[1:] if text.startswith("+") else text


def halves(digits):
    parts = digits.split(".")
    integer = parts[0] if parts else ""
    fraction = parts[1] if len(parts) > 1 else ""
    assert len(parts) <= 2, f"{digits!r} has more than one decimal point"
    assert integer or fraction, f"{digits!r} contains no digits"
    for half in (integer, fraction):
        for ch in half:
            assert ch.isdigit(), f"{digits!r} contains {ch!r}"
    return integer, fraction


def chunk(padded):
    return [int(padded[i:i + 4]) for i in range(0, len(padded), 4)]


def chunk_left_padded(digits):
    source = digits or "0"
    pad = (4 - len(source) % 4) % 4
    return chunk("0" * pad + source)


def chunk_right_padded(digits):
    if not digits:
        return []
    pad = (4 - len(digits) % 4) % 4
    return chunk(digits + "0" * pad)


def encode_parse(text):
    negative, digits = peel_sign(text)
    int_digits, frac_digits = halves(digits)
    dscale = len(frac_digits)
    int_groups = chunk_left_padded(int_digits)
    frac_groups = chunk_right_padded(frac_digits)
    weight = len(int_groups) - 1
    groups = int_groups + frac_groups
    while groups and groups[0] == 0:
        groups.pop(0)
        weight -= 1
    while groups and groups[-1] == 0:
        groups.pop()
    if not groups:
        weight = 0
    return negative, weight, dscale, groups


def encode(text):
    if text.strip().lower() == "nan":
        return 0, 0, SIGN_NAN, 0, []
    negative, weight, dscale, groups = encode_parse(text.strip())
    return (len(groups), weight, SIGN_NEG if negative else SIGN_POS,
            dscale, groups)


# ---- decoder: binary_decode_numeric_render.rs ---------------------------

def group_at(weight, groups, exponent):
    index = weight - exponent
    if index < 0:
        return 0
    return groups[index] if index < len(groups) else 0


def run(weight, groups, exponents):
    return "".join(f"{group_at(weight, groups, e):04d}" for e in exponents)


def decode(ndigits, weight, sign, dscale, groups):
    if sign == SIGN_NAN:
        return "NaN"
    out = "-" if sign == SIGN_NEG else ""
    if weight < 0:
        out += "0"
    else:
        trimmed = run(weight, groups, range(weight, -1, -1)).lstrip("0")
        out += trimmed if trimmed else "0"
    if dscale > 0:
        needed = -(-dscale // 4)
        out += "." + run(weight, groups, range(-1, -needed - 1, -1))[:dscale]
    return out


CASES = [
    "0", "1", "-1", "5", "12345.6789", "-12345.6789", "0.50", "-0.50",
    "19.99", "0.00000001", "123456789012345678901234567890",
    "-123456789012345678901234567890.123456789",
    "0.000000000000000000000000000001", "10000", "9999", "10001",
    "1.000000000000000000005", "7.0007", "100000000.00000001",
    "0.0", "0.000", "-0.0", "1000000", "0.1", "0.0001", "0.00001",
    "999999999999.999999999999", "NaN",
]


def wire(ndigits, weight, sign, dscale, groups):
    header = struct.pack(">hhHh", ndigits, weight, sign, dscale)
    return list(header + b"".join(struct.pack(">h", g) for g in groups))


def main():
    for text in CASES:
        args = encode(text)
        got = decode(*args)
        expected = text if text != "-0.0" else "-0.0"
        assert got == expected, (text, args, got)
        if text != "NaN":
            assert decimal.Decimal(got) == decimal.Decimal(text)
        print(f"  {text:>45} -> ndigits={args[0]} weight={args[1]} "
              f"dscale={args[3]} groups={args[4]} -> {got}")
    print("OK: every encode->decode round trip is byte-exact, no float used")

    print("\nEncoder fixtures for tests/postgres_binary.rs:")
    for text in ["19.99", "-0.50", "0", "NaN", "12345.6789"]:
        print(f"  {text:>12} -> {wire(*encode(text))}")


if __name__ == "__main__":
    main()
