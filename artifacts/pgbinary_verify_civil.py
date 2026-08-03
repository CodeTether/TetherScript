"""Verification harness for the calendar and epoch arithmetic in
src/postgres/binary_time_civil.rs and binary_time.rs.

Mirrors the Rust integer semantics (truncating division) exactly, then
exhaustively round-trips every day from 1600-01-01 to 2400-01-01 against
Python's datetime. Run: python3 artifacts/pgbinary_verify_civil.py
"""

import datetime


def idiv(a, b):
    """Rust integer division: truncates toward zero (Python's // floors)."""
    q = abs(a) // abs(b)
    return q if (a < 0) == (b < 0) else -q


def civil_from_days(days):
    z = days + 719468
    era = idiv(z if z >= 0 else z - 146096, 146097)
    doe = z - era * 146097
    yoe = idiv(doe - idiv(doe, 1460) + idiv(doe, 36524) - idiv(doe, 146096), 365)
    y = yoe + era * 400
    doy = doe - (365 * yoe + idiv(yoe, 4) - idiv(yoe, 100))
    mp = idiv(5 * doy + 2, 153)
    d = doy - idiv(153 * mp + 2, 5) + 1
    m = mp + 3 if mp < 10 else mp - 9
    return (y + 1 if m <= 2 else y, m, d)


def days_from_civil(year, month, day):
    y = year - 1 if month <= 2 else year
    era = idiv(y if y >= 0 else y - 399, 400)
    yoe = y - era * 400
    m = month
    doy = idiv(153 * (m - 3 if m > 2 else m + 9) + 2, 5) + day - 1
    doe = yoe * 365 + idiv(yoe, 4) - idiv(yoe, 100) + doy
    return era * 146097 + doe - 719468


def main():
    assert civil_from_days(0) == (1970, 1, 1)
    assert civil_from_days(10957) == (2000, 1, 1)
    assert civil_from_days(19737) == (2024, 1, 15)
    assert days_from_civil(1970, 1, 1) == 0
    assert days_from_civil(2024, 1, 15) == 19737
    assert civil_from_days(days_from_civil(2000, 2, 29)) == (2000, 2, 29)

    # Epoch constants.
    assert 10957 * 86400 == 946684800
    epoch = datetime.date(1970, 1, 1)
    start = (datetime.date(1600, 1, 1) - epoch).days
    end = (datetime.date(2400, 1, 1) - epoch).days
    for i in range(start, end):
        y, m, d = civil_from_days(i)
        real = epoch + datetime.timedelta(days=i)
        assert (y, m, d) == (real.year, real.month, real.day), (i, y, m, d, real)
        assert days_from_civil(y, m, d) == i, (i, y, m, d)
    print(f"OK: {end - start} days round-tripped, 1600-01-01..2400-01-01")
    print("OK: 10957 * 86400 == 946684800")


if __name__ == "__main__":
    main()
