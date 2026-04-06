"""Small supply network — equivalent of the original `small_example_network.py`.

Topology (physical airflow direction):

    ahu --> elbow --> duct --> tee_a --+--> term1
                                       |
                                       +--> tee_b --+--> term2
                                                    |
                                                    +--> cap

Run as a script to print the critical-path pressure drop.
"""

from __future__ import annotations

from pyduct2 import (
    ElbowRound,
    Network,
    RigidDuct,
    Round,
    Source,
    Tee,
    Terminal,
    TwoPortFitting,
    solve,
)


def build_network() -> Network:
    section = Round(diameter=1.0)

    net = Network("supply")
    net.add("ahu", Source("AHU"))
    net.add(
        "elbow",
        TwoPortFitting(
            "elbow_90",
            cross_section=section,
            zeta=ElbowRound(bend_radius=1.0, diameter=1.0, angle=90).zeta,
        ),
    )
    net.add("duct", RigidDuct("duct1", cross_section=section, length=10))
    net.add("tee_a", Tee("tee_a", cross_section=section))
    net.add("tee_b", Tee("tee_b", cross_section=section))
    net.add("term1", Terminal("air_terminal_1", flowrate=5.0))
    net.add("term2", Terminal("air_terminal_2", flowrate=7.0))
    net.add("cap", Terminal("cap", flowrate=0.0))

    net.connect("ahu", "elbow")
    net.connect("elbow", "duct")
    net.connect("duct", "tee_a.combined")
    net.connect("tee_a.straight", "term1")
    net.connect("tee_a.branch", "tee_b.combined")
    net.connect("tee_b.straight", "term2")
    net.connect("tee_b.branch", "cap")
    return net


if __name__ == "__main__":
    net = build_network()
    total_dp = solve(net)
    print(f"Critical-path pressure drop: {total_dp:.2f} Pa")
