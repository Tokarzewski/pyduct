"""Tests for Pydantic schemas and validation."""

import pytest
from pydantic import ValidationError

from pyduct2 import (
    CrossSectionSchema,
    FluidSchema,
    NetworkDesignSchema,
    RigidDuctSchema,
    SizingRequestSchema,
    TerminalSchema,
    TeeSchema,
)


class TestFluidSchema:
    def test_valid_fluid(self) -> None:
        fluid = FluidSchema(density=1.2, dynamic_viscosity=1.8e-5)
        assert fluid.density == 1.2

    def test_invalid_density(self) -> None:
        with pytest.raises(ValidationError):
            FluidSchema(density=-1.0, dynamic_viscosity=1.8e-5)

    def test_invalid_viscosity(self) -> None:
        with pytest.raises(ValidationError):
            FluidSchema(density=1.2, dynamic_viscosity=0)


class TestCrossSectionSchema:
    def test_valid_round(self) -> None:
        cs = CrossSectionSchema(shape="round", diameter=0.2)
        assert cs.shape == "round"
        assert cs.diameter == 0.2

    def test_valid_rectangular(self) -> None:
        cs = CrossSectionSchema(shape="rectangular", width=0.3, height=0.2)
        assert cs.shape == "rectangular"

    def test_round_missing_diameter(self) -> None:
        with pytest.raises(ValidationError):
            CrossSectionSchema(shape="round", diameter=None)

    def test_rectangular_missing_dims(self) -> None:
        with pytest.raises(ValidationError):
            CrossSectionSchema(shape="rectangular", width=0.3, height=None)

    def test_round_cannot_have_width(self) -> None:
        with pytest.raises(ValidationError):
            CrossSectionSchema(shape="round", diameter=0.2, width=0.1)


class TestRigidDuctSchema:
    def test_valid_duct(self) -> None:
        cs = CrossSectionSchema(shape="round", diameter=0.2)
        duct = RigidDuctSchema(name="d1", cross_section=cs, length=10)
        assert duct.name == "d1"
        assert duct.length == 10

    def test_zero_length_rejected(self) -> None:
        cs = CrossSectionSchema(shape="round", diameter=0.2)
        with pytest.raises(ValidationError):
            RigidDuctSchema(name="d1", cross_section=cs, length=0)


class TestTerminalSchema:
    def test_valid_terminal(self) -> None:
        term = TerminalSchema(name="t1", flowrate=0.1)
        assert term.flowrate == 0.1

    def test_negative_flowrate(self) -> None:
        with pytest.raises(ValidationError):
            TerminalSchema(name="t1", flowrate=-0.1)

    def test_zeta_default(self) -> None:
        term = TerminalSchema(name="t1", flowrate=0.1)
        assert term.zeta == 0.0


class TestTeeSchema:
    def test_valid_tee(self) -> None:
        cs = CrossSectionSchema(shape="round", diameter=0.2)
        tee = TeeSchema(name="tee", cross_section=cs)
        assert tee.zeta_straight == 0.0
        assert tee.zeta_branch == 0.5

    def test_custom_zeta(self) -> None:
        cs = CrossSectionSchema(shape="round", diameter=0.2)
        tee = TeeSchema(
            name="tee",
            cross_section=cs,
            zeta_straight=0.1,
            zeta_branch=0.8,
        )
        assert tee.zeta_straight == 0.1


class TestNetworkDesignSchema:
    def test_valid_network(self) -> None:
        cs = {"shape": "round", "diameter": 0.2}
        data = {
            "name": "test",
            "components": {
                "ahu": {"type": "Source", "name": "ahu"},
                "d1": {
                    "type": "RigidDuct",
                    "name": "d1",
                    "cross_section": cs,
                    "length": 10,
                },
                "term": {"type": "Terminal", "name": "term", "flowrate": 0.1},
            },
            "connections": [
                {"source": "ahu", "target": "d1"},
                {"source": "d1", "target": "term"},
            ],
        }
        schema = NetworkDesignSchema(**data)
        assert schema.name == "test"
        assert len(schema.components) == 3

    def test_missing_component_type(self) -> None:
        data = {
            "name": "test",
            "components": {"d1": {"name": "d1"}},  # missing type
            "connections": [],
        }
        with pytest.raises(ValidationError):
            NetworkDesignSchema(**data)

    def test_unknown_component_type(self) -> None:
        data = {
            "name": "test",
            "components": {"x": {"type": "UnknownFitting", "name": "x"}},
            "connections": [],
        }
        with pytest.raises(ValidationError):
            NetworkDesignSchema(**data)


class TestSizingRequestSchema:
    def test_velocity_method(self) -> None:
        req = SizingRequestSchema(
            flowrate=0.1,
            method="velocity",
            target_velocity=4.0,
        )
        assert req.method == "velocity"

    def test_equal_friction_method(self) -> None:
        req = SizingRequestSchema(
            flowrate=0.1,
            method="equal_friction",
            target_pressure_drop_per_meter=1.0,
        )
        assert req.method == "equal_friction"

    def test_pressure_budget_method(self) -> None:
        req = SizingRequestSchema(
            flowrate=0.1,
            method="pressure_budget",
            length=20.0,
            budget_pa=50.0,
        )
        assert req.method == "pressure_budget"

    def test_velocity_missing_target(self) -> None:
        with pytest.raises(ValidationError):
            SizingRequestSchema(
                flowrate=0.1,
                method="velocity",
                # missing target_velocity
            )

    def test_pressure_budget_missing_length(self) -> None:
        with pytest.raises(ValidationError):
            SizingRequestSchema(
                flowrate=0.1,
                method="pressure_budget",
                budget_pa=50.0,
                # missing length
            )


class TestSchemaRoundTrip:
    def test_schema_to_dict_to_schema(self) -> None:
        """Verify schemas can be serialized and deserialized."""
        original = SizingRequestSchema(
            flowrate=0.15,
            method="velocity",
            target_velocity=4.5,
            shape="rectangular",
        )
        data = original.model_dump()
        restored = SizingRequestSchema(**data)
        assert restored.flowrate == original.flowrate
        assert restored.method == original.method
