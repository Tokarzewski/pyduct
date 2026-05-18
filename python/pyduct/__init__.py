"""pyduct — ductwork sizing & pressure-drop calculations.

Public API:

>>> from pyduct import Network, Source, RigidDuct, Terminal, Round, solve
>>> net = Network("supply")
>>> net.add("ahu", Source("AHU"))
>>> net.add("d1", RigidDuct("d1", Round(0.2), length=10))
>>> net.add("t1", Terminal("t1", flowrate=0.1))
>>> net.connect("ahu", "d1")
>>> net.connect("d1", "t1")
>>> total_dp = solve(net)
"""

# Bootstrap the sibling Mojo package on sys.path so the Mojo-backed
# solver kernel + math shims are importable, then install the
# mojo.importer hook so .mojo sources auto-compile on first import.
# Must run before any module that imports wenta.ext — that's why every
# package import below carries an E402 noqa (intentionally not at the
# top of the file).
import sys as _sys  # noqa: E402
from pathlib import Path as _Path  # noqa: E402

_repo_root = str(_Path(__file__).resolve().parents[2])
if _repo_root not in _sys.path:
    _sys.path.insert(0, _repo_root)

import mojo.importer  # noqa: F401, E402

del _sys, _Path, _repo_root

from .components import (  # noqa: E402
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
from .components.fittings_library import (  # noqa: E402
    damper_butterfly,
    diffuser_ceiling,
    expander_round,
    grille_return,
    junction_tee_branch,
    junction_tee_combine,
    mitered_elbow,
    rectangular_elbow,
    reducer_round,
)
from .core import (  # noqa: E402
    STANDARD_AIR,
    CrossSection,
    Fluid,
    Rectangular,
    Round,
    air_at_altitude,
    equivalent_round_diameter,
)
from .data import (  # noqa: E402
    STANDARD_RECTANGULAR_DUCT_SIZES,
    STANDARD_RECTANGULAR_SECTIONS,
    STANDARD_ROUND_BRANCH_SIZES,
    STANDARD_ROUND_DUCT_SIZES,
    STANDARD_ROUND_SECTIONS,
    STANDARD_ROUND_TRANSFORMATION_SIZES,
    nearest_round_size,
)
from .io import (  # noqa: E402
    load_from_json,
    load_from_yaml,
    load_network_from_dict,
    save_network_to_dict,
    save_to_json,
    save_to_yaml,
)
from .network import (  # noqa: E402
    Network,
    compute_pressure_drops,
    critical_path,
    critical_path_pressure_drop,
    propagate_flowrates,
    solve,
)
from .results import (  # noqa: E402
    ComponentResult,
    extract_results,
    results_as_csv,
    results_as_dicts,
    results_summary,
)
from .schemas import (  # noqa: E402
    CrossSectionSchema,
    FlexDuctSchema,
    FluidSchema,
    NetworkDesignSchema,
    RigidDuctSchema,
    SizingRequestSchema,
    SourceSchema,
    TeeSchema,
    TerminalSchema,
    TwoPortFittingSchema,
)
from .sizing import (  # noqa: E402
    NOISE_LIMITS_M_S,
    aspect_ratio_method,
    equal_friction_method,
    noise_limit_method,
    pressure_drop_budget,
    velocity_method,
    velocity_method_batch,
)
from .units import (  # noqa: E402
    air_changes_per_hour,
    c_to_f,
    cfm_to_m3s,
    f_to_c,
    fpm_to_ms,
    ft_to_m,
    in_to_m,
    inwc_to_pa,
    m3s_to_cfm,
    m_to_ft,
    m_to_in,
    ms_to_fpm,
    pa_to_inwc,
)

__version__ = "0.1.0"

__all__ = [
    # core
    "Fluid",
    "STANDARD_AIR",
    "CrossSection",
    "Round",
    "Rectangular",
    "equivalent_round_diameter",
    "air_at_altitude",
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
    "rectangular_elbow",
    "mitered_elbow",
    # data
    "STANDARD_RECTANGULAR_DUCT_SIZES",
    "STANDARD_RECTANGULAR_SECTIONS",
    "STANDARD_ROUND_DUCT_SIZES",
    "STANDARD_ROUND_SECTIONS",
    "STANDARD_ROUND_BRANCH_SIZES",
    "STANDARD_ROUND_TRANSFORMATION_SIZES",
    "nearest_round_size",
    # sizing
    "velocity_method",
    "velocity_method_batch",
    "equal_friction_method",
    "pressure_drop_budget",
    "aspect_ratio_method",
    "noise_limit_method",
    "NOISE_LIMITS_M_S",
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
    # units
    "cfm_to_m3s",
    "m3s_to_cfm",
    "inwc_to_pa",
    "pa_to_inwc",
    "ft_to_m",
    "m_to_ft",
    "in_to_m",
    "m_to_in",
    "fpm_to_ms",
    "ms_to_fpm",
    "f_to_c",
    "c_to_f",
    "air_changes_per_hour",
]
