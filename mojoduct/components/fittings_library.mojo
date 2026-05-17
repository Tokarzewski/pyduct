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
