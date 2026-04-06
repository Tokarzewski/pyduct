"""pyduct2 — ductwork sizing & pressure-drop calculations.

A clean rewrite of the original `pyduct` package. Public API:

>>> from pyduct2 import Network, Source, RigidDuct, Terminal, Round, solve
>>> net = Network("supply")
>>> net.add("ahu", Source("AHU"))
>>> net.add("d1", RigidDuct("d1", Round(0.2), length=10))
>>> net.add("t1", Terminal("t1", flowrate=0.1))
>>> net.connect("ahu", "d1")
>>> net.connect("d1", "t1")
>>> total_dp = solve(net)
"""

from .components import (
    Component,
    ElbowRound,
    FlexDuct,
    Port,
    PortDirection,
    RigidDuct,
    Source,
    Tee,
    Terminal,
    TwoPortFitting,
)
from .components.fittings_library import (
    damper_butterfly,
    diffuser_ceiling,
    expander_round,
    grille_return,
    junction_tee_branch,
    junction_tee_combine,
    reducer_round,
)
from .core import STANDARD_AIR, CrossSection, Fluid, Rectangular, Round
from .data import (
    STANDARD_RECTANGULAR_DUCT_SIZES,
    STANDARD_ROUND_BRANCH_SIZES,
    STANDARD_ROUND_DUCT_SIZES,
    STANDARD_ROUND_TRANSFORMATION_SIZES,
    nearest_round_size,
)
from .network import (
    Network,
    compute_pressure_drops,
    critical_path,
    critical_path_pressure_drop,
    propagate_flowrates,
    solve,
)
from .results import (
    ComponentResult,
    extract_results,
    results_as_csv,
    results_as_dicts,
    results_summary,
)
from .sizing import (
    equal_friction_method,
    pressure_drop_budget,
    velocity_method,
)
from .io import (
    load_from_json,
    load_from_yaml,
    load_network_from_dict,
    save_to_json,
    save_to_yaml,
    save_network_to_dict,
)
from .schemas import (
    NetworkDesignSchema,
    SizingRequestSchema,
    RigidDuctSchema,
    FlexDuctSchema,
    SourceSchema,
    TerminalSchema,
    TwoPortFittingSchema,
    TeeSchema,
    CrossSectionSchema,
    FluidSchema,
)

__version__ = "0.1.0"

__all__ = [
    # core
    "Fluid",
    "STANDARD_AIR",
    "CrossSection",
    "Round",
    "Rectangular",
    # components
    "Component",
    "Port",
    "PortDirection",
    "RigidDuct",
    "FlexDuct",
    "Source",
    "Terminal",
    "TwoPortFitting",
    "Tee",
    "ElbowRound",
    # fittings library
    "reducer_round",
    "expander_round",
    "junction_tee_branch",
    "junction_tee_combine",
    "damper_butterfly",
    "diffuser_ceiling",
    "grille_return",
    # data
    "STANDARD_RECTANGULAR_DUCT_SIZES",
    "STANDARD_ROUND_DUCT_SIZES",
    "STANDARD_ROUND_BRANCH_SIZES",
    "STANDARD_ROUND_TRANSFORMATION_SIZES",
    "nearest_round_size",
    # sizing
    "velocity_method",
    "equal_friction_method",
    "pressure_drop_budget",
    # network / solver
    "Network",
    "propagate_flowrates",
    "compute_pressure_drops",
    "critical_path",
    "critical_path_pressure_drop",
    "solve",
    # results
    "ComponentResult",
    "extract_results",
    "results_summary",
    "results_as_dicts",
    "results_as_csv",
    # I/O & schemas
    "load_from_json",
    "load_from_yaml",
    "load_network_from_dict",
    "save_to_json",
    "save_to_yaml",
    "save_network_to_dict",
    "NetworkDesignSchema",
    "SizingRequestSchema",
    "RigidDuctSchema",
    "FlexDuctSchema",
    "SourceSchema",
    "TerminalSchema",
    "TwoPortFittingSchema",
    "TeeSchema",
    "CrossSectionSchema",
    "FluidSchema",
]
