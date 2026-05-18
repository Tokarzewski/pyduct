"""Cross-section geometry primitives (Mojo native port).

Mirrors `wenta.core.geometry`:

* `Round(diameter)` — circular cross-section
* `Rectangular(width, height)` — rectangular cross-section
* `equivalent_round_diameter(width, height)` — ASHRAE helper

Both structs cache `area` and `hydraulic_diameter` at construction time so
hot-path code reads a field rather than recomputing the math on every access.
"""

from std.math import pi


struct Round(Copyable, ImplicitlyCopyable, Movable):
    """Circular duct cross-section."""

    var diameter: Float64
    var area: Float64
    var hydraulic_diameter: Float64

    def __init__(out self, diameter: Float64) raises:
        if diameter <= 0.0:
            raise Error("diameter must be positive")
        self.diameter = diameter
        var r = diameter * 0.5
        self.area = pi * r * r
        # For a round duct the hydraulic diameter equals the diameter.
        self.hydraulic_diameter = diameter


struct Rectangular(Copyable, ImplicitlyCopyable, Movable):
    """Rectangular duct cross-section."""

    var width: Float64
    var height: Float64
    var area: Float64
    var hydraulic_diameter: Float64

    def __init__(out self, width: Float64, height: Float64) raises:
        if width <= 0.0 or height <= 0.0:
            raise Error("width and height must be positive")
        self.width = width
        self.height = height
        self.area = width * height
        # D_h = 4 A / P = 2 W H / (W + H)
        self.hydraulic_diameter = 2.0 * width * height / (width + height)


def equivalent_round_diameter(width: Float64, height: Float64) raises -> Float64:
    """ASHRAE equivalent round diameter for a rectangular duct.

        D_eq = 1.30 * (a*b)**0.625 / (a + b)**0.25     [m]
    """
    if width <= 0.0 or height <= 0.0:
        raise Error("width and height must be positive")
    return 1.30 * (width * height) ** 0.625 / (width + height) ** 0.25
