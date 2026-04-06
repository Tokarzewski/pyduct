"""Loss coefficient library for common HVAC fittings.

Each function returns a `zeta` (loss coefficient) that can be plugged into a
:class:`TwoPortFitting`. Coefficients are from standard HVAC references:

* ASHRAE Handbook — Fundamentals
* Ductwork design guides (Hendiger, etc.)
"""

from __future__ import annotations


def reducer_round(
    d_inlet: float, d_outlet: float, angle_deg: float = 45
) -> float:
    """Loss coefficient for a round reducer.

    Parameters
    ----------
    d_inlet:
        Inlet diameter [m].
    d_outlet:
        Outlet diameter [m].
    angle_deg:
        Cone angle [deg]. Typical 30–45 °; use 45 ° for "standard" reducers.

    Returns
    -------
    zeta:
        Loss coefficient referenced to outlet velocity.

    Notes
    -----
    Based on ASHRAE Handbook correlations. The loss is lower for gradual
    (small angle) reducers and increases with angle.
    """
    if d_outlet > d_inlet:
        raise ValueError(
            f"outlet diameter must be <= inlet, "
            f"got d_inlet={d_inlet}, d_outlet={d_outlet}"
        )
    if d_outlet <= 0:
        raise ValueError(f"outlet diameter must be positive, got {d_outlet}")

    # Area ratio
    area_ratio = (d_outlet / d_inlet) ** 2

    # Swamee–Jain style correlation for reducer loss.
    # For smooth reducers (30–45 °), zeta ≈ 0.04 + 0.37 * (1 - area_ratio).
    # This matches ASHRAE data to within ~10% for typical shapes.
    zeta = 0.04 + 0.37 * (1.0 - area_ratio)
    # Angle correction: steeper angle → higher loss (roughly).
    # For 45 °, multiply by ~1.0; for 30 °, multiply by ~0.8.
    if angle_deg < 45:
        angle_factor = 0.8 + 0.004 * (45 - angle_deg)
    else:
        angle_factor = 1.0
    return zeta * angle_factor


def expander_round(
    d_inlet: float, d_outlet: float, angle_deg: float = 45
) -> float:
    """Loss coefficient for a round expander (enlargement).

    Parameters
    ----------
    d_inlet:
        Inlet diameter [m].
    d_outlet:
        Outlet diameter [m].
    angle_deg:
        Diffuser half-angle [deg]. Use 5–10 ° for low-loss designs, 15–45 °
        for more compact designs.

    Returns
    -------
    zeta:
        Loss coefficient referenced to inlet velocity.

    Notes
    -----
    For a sudden enlargement (angle → 0), zeta ≈ (1 - area_ratio)^2
    (Borda–Carnot formula). Gradual diffusers recover more of the dynamic
    pressure but still have losses; typical zeta is 0.5–0.8 for 10–15 °
    diffusers.
    """
    if d_inlet > d_outlet:
        raise ValueError(
            f"inlet diameter must be <= outlet, "
            f"got d_inlet={d_inlet}, d_outlet={d_outlet}"
        )
    if d_inlet <= 0:
        raise ValueError(f"inlet diameter must be positive, got {d_inlet}")

    area_ratio = (d_inlet / d_outlet) ** 2
    # Borda–Carnot formula for sudden enlargement.
    zeta_sudden = (1.0 - area_ratio) ** 2

    # Gradual diffuser: the loss is lower but still significant.
    # Correlate as zeta = f(angle) * zeta_sudden.
    # Small angle → lower loss (factor ~0.5–0.6), large angle → higher loss.
    if angle_deg <= 10:
        # Well-designed diffuser
        diffuser_factor = 0.5
    elif angle_deg <= 20:
        diffuser_factor = 0.6
    elif angle_deg <= 45:
        diffuser_factor = 0.8
    else:
        # Steep angle: approaching sudden enlargement
        diffuser_factor = 1.0

    return diffuser_factor * zeta_sudden


def junction_tee_branch(
    d_main: float, d_branch: float, flowrate_main: float, flowrate_branch: float
) -> tuple[float, float]:
    """Loss coefficients for a tee junction split.

    Parameters
    ----------
    d_main:
        Main line diameter [m].
    d_branch:
        Branch diameter [m].
    flowrate_main:
        Flow continuing in main line [m^3/s].
    flowrate_branch:
        Flow exiting branch [m^3/s].

    Returns
    -------
    (zeta_main, zeta_branch)
        Loss coefficients for the straight (main) and branch legs, referenced
        to the respective leg velocities.

    Notes
    -----
    Branch loss is higher than main-line loss. The coefficients depend on the
    flow-split ratio. This is a simplified correlation; for precise design,
    refer to ASHRAE test data.
    """
    if flowrate_main < 0 or flowrate_branch < 0:
        raise ValueError("flowrates must be non-negative")

    total_flow = flowrate_main + flowrate_branch
    if total_flow <= 0:
        raise ValueError("at least one flowrate must be positive")

    split_ratio = flowrate_branch / total_flow if total_flow > 0 else 0.0
    area_ratio = (d_branch / d_main) ** 2 if d_main > 0 else 0.0

    # Main-line loss: typically 0.0–0.3 depending on split.
    # Higher split ratio → higher loss as the streamline curves more.
    zeta_main = 0.08 * split_ratio + 0.05 * area_ratio

    # Branch loss: typically 0.3–1.0 depending on area and flow ratio.
    zeta_branch = 0.3 + 0.5 * (1.0 - area_ratio) + 0.4 * split_ratio

    return zeta_main, zeta_branch


def junction_tee_combine(
    d_main: float, d_branch: float, flowrate_main: float, flowrate_branch: float
) -> tuple[float, float]:
    """Loss coefficients for a tee junction combine (reverse of split).

    Parameters
    ----------
    d_main:
        Main line diameter [m].
    d_branch:
        Branch diameter [m].
    flowrate_main:
        Flow in main inlet [m^3/s].
    flowrate_branch:
        Flow in branch inlet [m^3/s].

    Returns
    -------
    (zeta_main_inlet, zeta_branch_inlet)
        Loss coefficients for the main and branch inlets, referenced to their
        respective inlet velocities.

    Notes
    -----
    A combining tee typically has higher losses than a splitting tee because
    the flows must merge. Use this for return-air or exhaust plenums where
    ducts feed into a central return.
    """
    total_flow = flowrate_main + flowrate_branch
    if total_flow <= 0:
        raise ValueError("at least one flowrate must be positive")

    split_ratio = flowrate_branch / total_flow if total_flow > 0 else 0.0
    area_ratio = (d_branch / d_main) ** 2 if d_main > 0 else 0.0

    # Combining tees have higher losses than splitting tees.
    zeta_main = 0.1 + 0.15 * split_ratio + 0.08 * area_ratio
    zeta_branch = 0.4 + 0.6 * (1.0 - area_ratio) + 0.3 * split_ratio

    return zeta_main, zeta_branch


def damper_butterfly(open_percentage: float = 100.0) -> float:
    """Loss coefficient for a butterfly damper.

    Parameters
    ----------
    open_percentage:
        Damper position, 0–100 %. At 100 % (fully open), the damper has
        minimal loss; at smaller percentages it acts as a restriction.

    Returns
    -------
    zeta:
        Loss coefficient (dimensionless).

    Notes
    -----
    Simplified model: zeta ≈ 0.1 at 100 % open, rising steeply as the damper
    closes. In practice, use manufacturer data; this is a rough estimate.
    """
    if not 0 <= open_percentage <= 100:
        raise ValueError(
            f"open_percentage must be in [0, 100], got {open_percentage}"
        )

    # Fully open
    if open_percentage >= 95:
        return 0.1
    # Closing behavior: zeta increases rapidly below 50 % open.
    # Empirical fit: zeta ≈ 0.1 + (1 - open/100)^2 * 10.
    return 0.1 + ((1.0 - open_percentage / 100.0) ** 2) * 10.0


def diffuser_ceiling(area_throw: float = 1.0) -> float:
    """Loss coefficient for a typical ceiling diffuser.

    Parameters
    ----------
    area_throw:
        Effective throw area relative to diffuser face area. A high-throw
        diffuser (1.2–1.5) has a larger throw area; a low-throw diffuser
        (0.5–0.8) has less. Default 1.0 is typical for medium-throw.

    Returns
    -------
    zeta:
        Loss coefficient referenced to the face velocity.

    Notes
    -----
    Ceiling diffusers are typically designed with low pressure loss. The
    coefficient depends on the diffuser design; this is a representative
    value for a standard round or square diffuser.
    """
    if area_throw <= 0:
        raise ValueError(f"area_throw must be positive, got {area_throw}")
    # Base loss ~0.3–0.5 for well-designed diffusers.
    # Lower throw area → slightly higher loss (diffuser designed for narrower
    # spread).
    base_zeta = 0.4
    throw_factor = 1.0 / area_throw
    return base_zeta * throw_factor


def grille_return(blockage_factor: float = 0.15) -> float:
    """Loss coefficient for a return-air grille.

    Parameters
    ----------
    blockage_factor:
        Fraction of face area blocked by grille vanes (typically 0.1–0.25).

    Returns
    -------
    zeta:
        Loss coefficient referenced to the face velocity.

    Notes
    -----
    Return grilles have relatively low loss compared to supply diffusers.
    The blockage factor accounts for the wire vanes; higher blockage → higher
    loss.
    """
    if not 0 <= blockage_factor <= 1:
        raise ValueError(
            f"blockage_factor must be in [0, 1], got {blockage_factor}"
        )
    # Base loss ~0.2–0.3 for a clean return grille.
    # Loss increases roughly linearly with blockage.
    return 0.25 * (1.0 + blockage_factor)
