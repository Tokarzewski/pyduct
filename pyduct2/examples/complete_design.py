"""Complete HVAC supply network design example.

Demonstrates:
  - Duct sizing using velocity method
  - Complex branching network with fittings
  - Results extraction and summary
  - Critical path analysis
"""

from pyduct2 import (
    ElbowRound,
    Network,
    RigidDuct,
    Round,
    Source,
    Tee,
    Terminal,
    TwoPortFitting,
    extract_results,
    results_summary,
    solve,
    velocity_method,
)
from pyduct2.components.fittings_library import (
    damper_butterfly,
    junction_tee_branch,
)


def main() -> None:
    """Design a 5-zone HVAC supply network."""

    print("=" * 70)
    print("HVAC Supply Network Design Example")
    print("=" * 70)

    # ---- Sizing ----
    # Main trunk: 0.25 m^3/s, size by velocity method
    print("\n1. Sizing ducts by velocity method (target 5 m/s):")
    main_section, main_v = velocity_method(0.25, "round", target_velocity=5.0)
    print(f"   Main trunk: D={main_section.diameter:.3f} m, v={main_v:.2f} m/s")

    # Zone branches: size individually
    zone_sizes = [
        ("Zone 1", 0.10),
        ("Zone 2", 0.08),
        ("Zone 3", 0.07),
    ]
    zones_info = []
    for name, flowrate in zone_sizes:
        sec, v = velocity_method(flowrate, "round", target_velocity=4.0)
        zones_info.append((name, flowrate, sec, v))
        print(f"   {name}: Q={flowrate:.3f} m³/s → D={sec.diameter:.3f} m, v={v:.2f} m/s")

    # ---- Network construction ----
    print("\n2. Building network topology:")
    net = Network("zone_supply")

    # AHU
    net.add("ahu", Source("AHU 0.25 m³/s"))
    print("   ✓ AHU (source)")

    # Main duct from AHU
    net.add("main_duct", RigidDuct("main_trunk", main_section, length=30))
    net.connect("ahu", "main_duct")
    print("   ✓ Main trunk duct (30 m)")

    # Elbow at main duct outlet
    main_elbow = ElbowRound(
        bend_radius=main_section.diameter, diameter=main_section.diameter, angle=90
    )
    net.add(
        "main_elbow",
        TwoPortFitting("elbow_main", main_section, zeta=main_elbow.zeta),
    )
    net.connect("main_duct", "main_elbow")
    print("   ✓ 90° elbow at main duct outlet")

    # Main tee for first branch
    net.add("tee_main", Tee("main_tee", main_section))
    net.connect("main_elbow", "tee_main.combined")
    print("   ✓ Main tee (splits to zone and sub-main)")

    # ---- Zones 1 & 2: tee split ----
    net.add("tee_sub1", Tee("sub_tee_1", zones_info[0][2]))
    net.connect("tee_main.straight", "tee_sub1.combined")
    print("   ✓ Sub-tee 1 (zones 1 & 2)")

    # Zone 1 with damper
    net.add("zone1_damper", TwoPortFitting("damper_z1", zones_info[0][2], zeta=damper_butterfly(80)))
    net.add("zone1_duct", RigidDuct("duct_z1", zones_info[0][2], length=15))
    net.add("zone1_term", Terminal("Zone 1", flowrate=0.10))
    net.connect("tee_sub1.straight", "zone1_damper")
    net.connect("zone1_damper", "zone1_duct")
    net.connect("zone1_duct", "zone1_term")
    print("   ✓ Zone 1: damper + duct + terminal")

    # Zone 2
    net.add("zone2_duct", RigidDuct("duct_z2", zones_info[1][2], length=20))
    net.add("zone2_term", Terminal("Zone 2", flowrate=0.08))
    net.connect("tee_sub1.branch", "zone2_duct")
    net.connect("zone2_duct", "zone2_term")
    print("   ✓ Zone 2: duct + terminal")

    # ---- Zone 3: branch from main ----
    net.add("zone3_duct", RigidDuct("duct_z3", zones_info[2][2], length=25))
    net.add("zone3_term", Terminal("Zone 3", flowrate=0.07))
    net.connect("tee_main.branch", "zone3_duct")
    net.connect("zone3_duct", "zone3_term")
    print("   ✓ Zone 3: duct + terminal")

    # ---- Solving ----
    print("\n3. Solving network:")
    total_dp = solve(net)
    print(f"   Total critical-path pressure drop: {total_dp:.1f} Pa")
    print(f"   (Equivalent to {total_dp / 1.2:.1f} mm H₂O)")

    # ---- Results ----
    print("\n4. Component summary:")
    print(results_summary(net))

    # ---- Detailed results ----
    print("\n5. Detailed component data:")
    results = extract_results(net)
    for res in results:
        if res.pressure_drop > 0.01:
            q_str = f"{res.flowrate_in:.4f}" if res.flowrate_in else "—"
            v_str = f"{res.velocity_in:.2f}" if res.velocity_in else "—"
            print(
                f"   {res.component_id:15s} {res.name:20s} "
                f"Q={q_str:>8s} m³/s  v={v_str:>6s} m/s  "
                f"ΔP={res.pressure_drop:>7.2f} Pa"
            )

    print("\n" + "=" * 70)
    print("Design complete. Total ductwork: 90 m, 3 zones, 1 damper control.")
    print("=" * 70)


if __name__ == "__main__":
    main()
