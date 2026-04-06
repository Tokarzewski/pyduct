"""Example: Load a network from YAML, solve, and export results.

Demonstrates Pydantic schemas, YAML I/O, and full workflow.
"""

from pathlib import Path

from pyduct2 import (
    critical_path_pressure_drop,
    load_from_yaml,
    results_summary,
    save_to_json,
    solve,
)


def main() -> None:
    """Load, solve, and export a network."""

    print("=" * 70)
    print("YAML Network Loading Example")
    print("=" * 70)

    # Find the YAML file (same directory as this script)
    script_dir = Path(__file__).parent
    yaml_file = script_dir / "network_yaml.yaml"

    if not yaml_file.exists():
        print(f"Error: {yaml_file} not found")
        return

    # ---- Load network from YAML ----
    print(f"\n1. Loading network from {yaml_file.name}...")
    try:
        net = load_from_yaml(yaml_file)
        print(f"   ✓ Network loaded: {net.name}")
        print(f"   ✓ Components: {len(net.components)}")
    except Exception as e:
        print(f"   ✗ Error loading YAML: {e}")
        return

    # ---- Solve ----
    print("\n2. Solving network...")
    dp = solve(net)
    print(f"   ✓ Critical-path pressure drop: {dp:.2f} Pa")

    # ---- Display results ----
    print("\n3. Component Results:")
    print(results_summary(net))

    # ---- Export to JSON ----
    print("\n4. Exporting results to JSON...")
    output_json = script_dir / "network_solved.json"
    save_to_json(net, output_json)
    print(f"   ✓ Saved to {output_json.name}")

    print("\n" + "=" * 70)


if __name__ == "__main__":
    main()
