"""EN 1506:2007 round duct sizes (Mojo port).

Mirrors `pyduct.data.standard_sizes`:

* `STANDARD_ROUND_DUCT_SIZES_MM` — 22 nominal diameters [mm]
* `nearest_round_size(diameter_mm, round_up=True)` — bisect lookup
"""


def _round_sizes_mm() -> List[Int]:
    """EN 1506:2007 nominal round duct diameters [mm]."""
    return [
        63, 80, 100, 125, 150, 160, 200, 250, 300, 315, 355,
        400, 450, 500, 560, 630, 710, 800, 900, 1000, 1120, 1250,
    ]


def nearest_round_size(diameter_mm: Float64, round_up: Bool = True) raises -> Int:
    """Return the nearest EN 1506 nominal diameter [mm].

    If ``round_up`` is True (default), pick the smallest standard size that is
    ≥ ``diameter_mm``. Otherwise pick the closest standard size in either
    direction.
    """
    var sizes = _round_sizes_mm()
    var n = len(sizes)
    var first = sizes[0]
    var last = sizes[n - 1]
    if diameter_mm <= Float64(first):
        return first
    if diameter_mm >= Float64(last):
        return last

    # Linear search — 22 elements; bisect's overhead isn't worth it here.
    var idx = 0
    for i in range(n):
        if Float64(sizes[i]) >= diameter_mm:
            idx = i
            break

    var hi = sizes[idx]
    if round_up or Float64(hi) == diameter_mm:
        return hi
    var lo = sizes[idx - 1]
    var d_hi = Float64(hi) - diameter_mm
    var d_lo = diameter_mm - Float64(lo)
    return hi if d_hi < d_lo else lo
