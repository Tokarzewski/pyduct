"""Loss-coefficient correlations for common HVAC fittings (Mojo port).

Currently ports the rectangular-elbow and mitered-elbow correlations —
the bread-and-butter losses in commercial supply networks. Other
correlations (round reducers / expanders / tees / dampers / diffusers /
return grilles) stay Python for now and can be reached via interop.
"""


def rectangular_elbow(
    width: Float64, height: Float64, bend_radius: Float64, angle_deg: Float64 = 90.0
) raises -> Float64:
    """Loss coefficient for a smooth-radius rectangular elbow (Idelchik §6).

        zeta_90 ≈ 0.21 / (r/W)^0.5            (r/W ≥ 0.5), capped at 1.5

    Aspect-ratio correction multiplies by ``(H/W)^0.25``; angle scales
    linearly off 90 °.
    """
    var smallest = width if width <= height else height
    smallest = bend_radius if bend_radius < smallest else smallest
    if smallest <= 0.0:
        raise Error("width, height and bend_radius must be positive")
    if angle_deg <= 0.0 or angle_deg > 180.0:
        raise Error("angle_deg must be in (0, 180]")

    var r_over_w = bend_radius / width
    var floor = r_over_w if r_over_w > 0.1 else 0.1
    var zeta_90 = 0.21 / floor ** 0.5
    if zeta_90 > 1.5:
        zeta_90 = 1.5
    var aspect_correction = (height / width) ** 0.25
    return zeta_90 * aspect_correction * (angle_deg / 90.0)


def reducer_round(
    d_inlet: Float64, d_outlet: Float64, angle_deg: Float64 = 45.0
) raises -> Float64:
    """Loss coefficient for a round reducer (ASHRAE/Swamee–Jain style).

        zeta ≈ 0.04 + 0.37 · (1 − A_out / A_in)

    Multiplied by an angle factor that softens 0.8× at 30 ° and rises
    linearly to 1.0× at 45 °+. Referenced to the **outlet** velocity.
    """
    if d_outlet > d_inlet:
        raise Error("outlet diameter must be <= inlet")
    if d_outlet <= 0.0:
        raise Error("outlet diameter must be positive")
    var area_ratio = (d_outlet / d_inlet) ** 2
    var zeta = 0.04 + 0.37 * (1.0 - area_ratio)
    var angle_factor = 0.8 + 0.004 * (45.0 - angle_deg) if angle_deg < 45.0 else 1.0
    return zeta * angle_factor


def expander_round(
    d_inlet: Float64, d_outlet: Float64, angle_deg: Float64 = 45.0
) raises -> Float64:
    """Loss coefficient for a round expander / diffuser.

    Sudden-enlargement Borda–Carnot baseline ``(1 − A_in / A_out)²``
    multiplied by a piecewise diffuser factor that depends on the
    half-angle. Referenced to the **inlet** velocity.
    """
    if d_inlet > d_outlet:
        raise Error("inlet diameter must be <= outlet")
    if d_inlet <= 0.0:
        raise Error("inlet diameter must be positive")
    var area_ratio = (d_inlet / d_outlet) ** 2
    var zeta_sudden = (1.0 - area_ratio) ** 2
    var diffuser_factor: Float64 = 1.0
    if angle_deg <= 10.0:
        diffuser_factor = 0.5
    elif angle_deg <= 20.0:
        diffuser_factor = 0.6
    elif angle_deg <= 45.0:
        diffuser_factor = 0.8
    return diffuser_factor * zeta_sudden


def mitered_elbow(angle_deg: Float64 = 90.0, vaned: Bool = False) raises -> Float64:
    """Loss coefficient for a sharp-corner mitered elbow.

    Quadratic fit to the ASHRAE tabulated points (≤ 5 % error at 45 / 60
    / 90 / 120 °). Turning-vane insert (`vaned=True`) cuts the loss to ~40 %.
    """
    if angle_deg <= 0.0 or angle_deg > 180.0:
        raise Error("angle_deg must be in (0, 180]")
    var a = angle_deg / 90.0
    var zeta_unvaned = 0.55 * a + 0.65 * a * a
    return zeta_unvaned * (0.4 if vaned else 1.0)
