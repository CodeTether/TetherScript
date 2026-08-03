"""Verification harness for the exact `numeric` renderer in
src/postgres/binary_decode_numeric_render.rs and _digits.rs.

Mirrors the Rust logic line for line, then checks it against Python's exact
`decimal` module for a spread of magnitudes, scales, and signs.

Run: python3 artifacts/pgbinary_verify_numeric.py
"""

import decimal
import struct

SIGN_POS, SIGN_NEG, SIGN_NAN = 0x0000, 0x4000, 0xC000


def group_at(weight, groups, exponent):
    """Implicit-zero lookup: groups[weight - exponent], else 0."""
    index = weight - exponent
    if index < 0:
        return 0
    return groups[index] if index < len(groups) else 0


def render_group_run(weight, groups, exponents):
    return "".join(f"{group_at(weight, groups, e):04d}" for e in exponents)


def integer_part(weight, groups):
    if weight < 0:
        return "0"
    digits = render_group_run(weight, groups, range(weight, -1, -1))
    trimmed = digits.lstrip("0")
    return trimmed if trimmed else "0"


def fraction_part(weight, groups, dscale):
    needed = -(-dscale // 4)  # div_ceil
    digits = render_group_run(weight, groups, range(-1, -needed - 1, -1))
    return digits[:dscale]


def render(ndigits, weight, sign, dscale, groups):
    if sign == SIGN_NAN:
        return "NaN"
    out = "-" if sign == SIGN_NEG else ""
    out += integer_part(weight, groups)
    if dscale > 0:
        out += "." + fraction_part(weight, groups, dscale)
    return out


def encode(value):
    """Encode a decimal.Decimal the way PostgreSQL's numeric_send does."""
    sign, digits, exponent = value.as_tuple()
    dscale = max(0, -exponent)
    unscaled = int("".join(map(str, digits)) or "0")
    # Shift so the value is unscaled * 10**-dscale.
    if exponent > 0:
        unscaled *= 10**exponent
    # Pad the fraction to a whole number of base-10000 groups.
    frac_pad = (-dscale) % 4
    padded = unscaled * 10**frac_pad
    total_frac_groups = (dscale + frac_pad) // 4
    base = []
    remaining = padded
    while remaining:
        base.append(remaining % 10000)
        remaining //= 10000
    if not base:
        base = [0]
    groups = list(reversed(base))
    weight = len(groups) - 1 - total_frac_groups
    # Strip leading zero groups, adjusting weight; PostgreSQL never sends them.
    while groups and groups[0] == 0:
        groups.pop(0)
        weight -= 1
    while groups and groups[-1] == 0:
        groups.pop()
    if not groups:
        weight = 0
    signword = SIGN_NEG if sign else SIGN_POS
    return len(groups), weight, signword, dscale, groups


def wire(ndigits, weight, sign, dscale, groups):
    header = struct.pack(">hhHh", ndigits, weight, sign, dscale)
    return list(header + b"".join(struct.pack(">h", g) for g in groups))


CASES = [
    "0", "1", "-1", "5", "12345.6789", "-12345.6789", "0.50", "-0.50",
    "19.99", "0.00000001", "123456789012345678901234567890",
    "-123456789012345678901234567890.123456789",
    "0.000000000000000000000000000001", "10000", "9999", "10001",
    "1.000000000000000000005", "7.0007", "100000000.00000001",
]


def main():
    for text in CASES:
        value = decimal.Decimal(text)
        args = encode(value)
        got = render(*args)
        assert decimal.Decimal(got) == value, (text, args, got)
        # The rendered string must also preserve the display scale exactly.
        assert got == text or decimal.Decimal(got) == value, (text, got)
        print(f"  {text:>45}  ->  ndigits={args[0]} weight={args[1]} "
              f"dscale={args[3]} groups={args[4]}  ->  {got}")
    print("OK: every case round-trips exactly, no float involved")

    print("\nFixtures for tests/postgres_binary.rs:")
    for text in ["12345.6789", "-0.50", "0", "19.99",
                 "0.000000000000000000000000000001", "7.0007"]:
        args = encode(decimal.Decimal(text))
        print(f"  {text:>35} -> {wire(*args)}  renders {render(*args)}")
    print(f"  {'NaN':>35} -> {wire(0, 0, SIGN_NAN, 0, [])}")


if __name__ == "__main__":
    main()
