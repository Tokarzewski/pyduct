//! Fluid properties — density, dynamic & kinematic viscosity.

use crate::Result;
/// A working fluid (typically air).
///
/// `density` [kg/m³], `dynamic_viscosity` [Pa·s]. Kinematic viscosity
/// `nu = mu / rho` is cached at construction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fluid {
    pub density: f64,             // rho [kg/m^3]
    pub dynamic_viscosity: f64,   // mu [Pa.s]
    pub kinematic_viscosity: f64, // nu = mu / rho [m^2/s]
}

impl Fluid {
    pub fn new(density: f64, dynamic_viscosity: f64) -> Result<Self> {
        if density <= 0.0 {
            return Err("density must be positive".into());
        }
        if dynamic_viscosity <= 0.0 {
            return Err("dynamic_viscosity must be positive".into());
        }
        Ok(Fluid {
            density,
            dynamic_viscosity,
            kinematic_viscosity: dynamic_viscosity / density,
        })
    }

    /// Kinematic viscosity nu = mu / rho [m²/s].
    #[inline]
    pub fn kinematic_viscosity(&self) -> f64 {
        self.kinematic_viscosity
    }
}

/// Standard dry air at 20 °C, 101 325 Pa.
///
/// Values match CoolProp.PropsSI("D"/"V", "T", 293.15, "P", 101325, "Air")
/// to 4 significant figures, so the library has no runtime dependency on
/// CoolProp.
pub const STANDARD_AIR: Fluid = Fluid {
    density: 1.204,
    dynamic_viscosity: 1.825e-5,
    kinematic_viscosity: 1.825e-5 / 1.204,
};

/// Dry-air properties at altitude [m] and temperature [°C] (ISA atmosphere +
/// Sutherland viscosity).
pub fn air_at_altitude(altitude_m: f64, temperature_c: f64) -> Result<Fluid> {
    if altitude_m < 0.0 {
        return Err("altitude_m must be non-negative".into());
    }
    let h = if altitude_m < 11000.0 {
        altitude_m
    } else {
        11000.0
    };
    // ISA pressure up to the tropopause.
    let pressure = 101325.0 * (1.0 - 2.25577e-5 * h).powf(5.2561);
    let t_k = temperature_c + 273.15;
    let r_specific = 287.058; // J/(kg·K) for dry air
    let density = pressure / (r_specific * t_k);
    // Sutherland: mu(T) = 1.458e-6 * T^1.5 / (T + 110.4)
    let mu = 1.458e-6 * t_k.powf(1.5) / (t_k + 110.4);
    Fluid::new(density, mu)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinematic_viscosity_is_mu_over_rho() {
        let f = Fluid::new(1.204, 1.825e-5).unwrap();
        assert!((f.kinematic_viscosity - 1.825e-5 / 1.204).abs() < 1e-20);
    }

    #[test]
    fn standard_air_matches_known_values() {
        let f = Fluid::new(1.204, 1.825e-5).unwrap();
        assert!((f.density - STANDARD_AIR.density).abs() < 1e-15);
        assert!((f.kinematic_viscosity - STANDARD_AIR.kinematic_viscosity).abs() < 1e-15);
    }

    #[test]
    fn air_at_sea_level_is_standard_air_like() {
        let f = air_at_altitude(0.0, 20.0).unwrap();
        // ~1.204 kg/m^3 at sea level, 20 C.
        assert!((f.density - 1.204).abs() < 0.05);
        // Sutherland gives ~1.815e-5 Pa.s; STANDARD_AIR is a rounded
        // CoolProp constant (1.825e-5), so allow a small band.
        assert!((f.dynamic_viscosity - 1.825e-5).abs() < 2e-7);
    }

    #[test]
    fn air_density_decreases_with_altitude() {
        let sea = air_at_altitude(0.0, 20.0).unwrap();
        let high = air_at_altitude(3000.0, 20.0).unwrap();
        assert!(high.density < sea.density);
    }

    #[test]
    fn rejects_bad_fluid() {
        assert!(Fluid::new(0.0, 1.0).is_err());
        assert!(Fluid::new(1.0, 0.0).is_err());
    }
}
