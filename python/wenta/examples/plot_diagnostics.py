"""Render the 5-panel network diagnostic image (IDs / Name / Critical path /
Flowrate / Pressure drop) for an example supply network.

Run from the repo root:

    python -m wenta.examples.plot_diagnostics

This writes ``img/network_diagnostics.png`` next to the existing
``img/five graphs.jpg`` reference.

Requires the optional ``[plot]`` extra:  ``pip install -e ".[plot]"``.
"""

from __future__ import annotations

from pathlib import Path

from wenta import (
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
from wenta.visualization import plot_diagnostics


def build_example_network() -> Network:
    """A small branched supply network with elbow, tee and two terminals."""
    section_main = Round(0.20)
    section_branch = Round(0.16)

    net = Network("diagnostics_example")
    net.add("ahu", Source("air terminal"))
    net.add("duct1", RigidDuct("duct1", section_main, length=8.0))

    elbow_main = ElbowRound(bend_radius=0.30, diameter=section_main.diameter, angle=90)
    net.add(
        "elbow1",
        TwoPortFitting("elbow round", section_main, zeta=elbow_main.zeta),
    )

    net.add("duct2", RigidDuct("duct2", section_main, length=6.0))
    net.add("tee", Tee("branch", section_main, zeta_straight=0.05, zeta_branch=0.5))
    net.add("duct3a", RigidDuct("duct3a", section_branch, length=4.0))
    net.add("duct3b", RigidDuct("duct3b", section_branch, length=4.0))
    net.add("term_a", Terminal("air terminal", flowrate=0.05))
    net.add("term_b", Terminal("cap", flowrate=0.05))

    net.connect("ahu", "duct1")
    net.connect("duct1", "elbow1")
    net.connect("elbow1", "duct2")
    net.connect("duct2", "tee.combined")
    net.connect("tee.straight", "duct3a")
    net.connect("tee.branch", "duct3b")
    net.connect("duct3a", "term_a")
    net.connect("duct3b", "term_b")
    return net


def main() -> None:
    net = build_example_network()
    solve(net)

    out_dir = Path(__file__).resolve().parents[2] / "img"
    out_dir.mkdir(exist_ok=True)
    out_path = out_dir / "network_diagnostics.png"

    plot_diagnostics(net, save_path=str(out_path), show=False)
    print(f"Wrote {out_path}")


if __name__ == "__main__":
    main()
