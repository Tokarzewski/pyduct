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


def _rect_sizes_mm() -> List[Tuple[Int, Int]]:
    """EN 1505:2001 nominal rectangular duct sizes (width × height) [mm]."""
    return [
        (100, 200), (150, 200), (200, 200), (100, 250), (150, 250),
        (200, 250), (250, 250), (100, 300), (150, 300), (200, 300),
        (250, 300), (300, 300), (100, 400), (150, 400), (200, 400),
        (250, 400), (300, 400), (400, 400), (150, 500), (200, 500),
        (250, 500), (300, 500), (400, 500), (500, 500), (150, 600),
        (200, 600), (250, 600), (300, 600), (400, 600), (500, 600),
        (600, 600), (200, 800), (250, 800), (300, 800), (400, 800),
        (500, 800), (600, 800), (800, 800), (250, 1000), (300, 1000),
        (400, 1000), (500, 1000), (600, 1000), (800, 1000), (1000, 1000),
        (300, 1200), (400, 1200), (500, 1200), (600, 1200), (800, 1200),
        (1000, 1200), (1200, 1200), (400, 1400), (500, 1400), (600, 1400),
        (800, 1400), (1000, 1400), (1200, 1400), (400, 1600), (500, 1600),
        (600, 1600), (800, 1600), (1000, 1600), (1200, 1600), (500, 1800),
        (600, 1800), (800, 1800), (1000, 1800), (1200, 1800), (500, 2000),
        (600, 2000), (800, 2000), (1000, 2000), (1200, 2000),
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
