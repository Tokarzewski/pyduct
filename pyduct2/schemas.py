"""Pydantic schemas for network serialization, validation, and I/O.

Enables:
- Type-safe validation of network inputs
- YAML/JSON serialization/deserialization
- Better error messages with validation failures
- IDE autocompletion and static analysis
"""

from __future__ import annotations

from typing import Any, Literal

from pydantic import BaseModel, ConfigDict, Field, field_validator, model_validator


class FluidSchema(BaseModel):
    """Schema for a fluid definition."""

    model_config = ConfigDict(json_schema_extra={
        "examples": [
            {"density": 1.204, "dynamic_viscosity": 1.825e-5, "comment": "Standard air 20°C"}
        ]
    })

    density: float = Field(gt=0, description="Mass density [kg/m³]")
    dynamic_viscosity: float = Field(gt=0, description="Dynamic viscosity [Pa·s]")


class CrossSectionSchema(BaseModel):
    """Schema for a duct cross-section."""

    shape: Literal["round", "rectangular"]
    diameter: float | None = Field(None, gt=0, description="Diameter [m] (round only)")
    width: float | None = Field(None, gt=0, description="Width [m] (rectangular only)")
    height: float | None = Field(None, gt=0, description="Height [m] (rectangular only)")

    @model_validator(mode="after")
    def validate_shape(self) -> CrossSectionSchema:
        if self.shape == "round":
            if self.diameter is None:
                raise ValueError("Round section requires 'diameter'")
            if self.width is not None or self.height is not None:
                raise ValueError("Round section cannot have 'width' or 'height'")
        else:  # rectangular
            if self.width is None or self.height is None:
                raise ValueError("Rectangular section requires 'width' and 'height'")
            if self.diameter is not None:
                raise ValueError("Rectangular section cannot have 'diameter'")
        return self


class RigidDuctSchema(BaseModel):
    """Schema for a rigid duct component."""

    name: str
    cross_section: CrossSectionSchema
    length: float = Field(gt=0, description="Duct length [m]")
    absolute_roughness: float = Field(default=0.0001, ge=0, description="Roughness [m]")


class FlexDuctSchema(BaseModel):
    """Schema for a flexible duct component."""

    name: str
    diameter: float = Field(gt=0, description="Diameter [m]")
    length: float = Field(gt=0, description="Length [m]")
    pressure_drop_per_meter: float = Field(gt=0, description="Pressure drop [Pa/m]")
    stretch_percentage: float = Field(default=100.0, gt=0, le=100, description="Stretch %")


class SourceSchema(BaseModel):
    """Schema for an AHU/source component."""

    name: str


class TerminalSchema(BaseModel):
    """Schema for a terminal (diffuser, grille, cap)."""

    name: str
    flowrate: float = Field(ge=0, description="Demanded flow [m³/s]")
    zeta: float = Field(default=0.0, ge=0, description="Loss coefficient")


class TwoPortFittingSchema(BaseModel):
    """Schema for a two-port fitting."""

    name: str
    cross_section: CrossSectionSchema
    zeta: float = Field(ge=0, description="Loss coefficient")


class TeeSchema(BaseModel):
    """Schema for a three-port tee."""

    name: str
    cross_section: CrossSectionSchema
    zeta_straight: float = Field(default=0.0, ge=0, description="Straight leg zeta")
    zeta_branch: float = Field(default=0.5, ge=0, description="Branch leg zeta")


# Union schema for any component
ComponentSchema = RigidDuctSchema | FlexDuctSchema | SourceSchema | TerminalSchema | TwoPortFittingSchema | TeeSchema


class ConnectionSchema(BaseModel):
    """Schema for a network connection between two ports."""

    source: str = Field(description="Source component:port (port optional)")
    target: str = Field(description="Target component:port (port optional)")


class NetworkDesignSchema(BaseModel):
    """Complete schema for a network design (serializable)."""

    name: str = Field(description="Network name")
    fluid: FluidSchema | None = Field(default=None, description="Custom fluid (optional)")
    components: dict[str, dict[str, Any]] = Field(
        description="Components: {id: {type, name, ...}}"
    )
    connections: list[ConnectionSchema] = Field(
        default_factory=list, description="Connections between components"
    )

    @field_validator("components")
    @classmethod
    def validate_components(cls, v: dict[str, Any]) -> dict[str, Any]:
        allowed_types = {
            "RigidDuct", "FlexDuct", "Source", "Terminal",
            "TwoPortFitting", "Tee"
        }
        for cid, comp in v.items():
            if "type" not in comp:
                raise ValueError(f"Component {cid!r} missing 'type' field")
            if comp["type"] not in allowed_types:
                raise ValueError(
                    f"Component {cid!r}: unknown type {comp['type']!r}. "
                    f"Allowed: {allowed_types}"
                )
        return v


class SizingRequestSchema(BaseModel):
    """Schema for a duct sizing request."""

    flowrate: float = Field(gt=0, description="Flowrate [m³/s]")
    method: Literal["velocity", "equal_friction", "pressure_budget"]
    shape: Literal["round", "rectangular"] = Field(default="round")

    # Velocity method
    target_velocity: float | None = Field(default=None, gt=0, description="Target m/s")

    # Equal-friction method
    target_pressure_drop_per_meter: float | None = Field(
        default=None, gt=0, description="Target Pa/m"
    )

    # Pressure budget method
    length: float | None = Field(default=None, gt=0, description="Duct length [m]")
    budget_pa: float | None = Field(default=None, gt=0, description="Budget [Pa]")

    absolute_roughness: float = Field(default=0.0001, ge=0)

    @model_validator(mode="after")
    def validate_method(self) -> SizingRequestSchema:
        if self.method == "velocity" and self.target_velocity is None:
            raise ValueError("velocity method requires 'target_velocity'")
        if self.method == "equal_friction" and self.target_pressure_drop_per_meter is None:
            raise ValueError("equal_friction method requires 'target_pressure_drop_per_meter'")
        if self.method == "pressure_budget":
            if self.length is None or self.budget_pa is None:
                raise ValueError("pressure_budget method requires 'length' and 'budget_pa'")
        return self
