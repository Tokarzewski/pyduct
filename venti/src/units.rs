//! Unit converters between SI (venti native) and US customary units.

use crate::Result;
pub const CFM_TO_M3S: f64 = 0.0004719474432; // ft^3/min -> m^3/s
pub const INWC_TO_PA: f64 = 249.0889; // inch H2O (4 C) -> Pa
pub const FT_TO_M: f64 = 0.3048;
pub const IN_TO_M: f64 = 0.0254;
pub const FPM_TO_MS: f64 = 0.00508;

pub fn cfm_to_m3s(cfm: f64) -> f64 {
    cfm * CFM_TO_M3S
}
pub fn m3s_to_cfm(m3s: f64) -> f64 {
    m3s / CFM_TO_M3S
}
pub fn inwc_to_pa(inwc: f64) -> f64 {
    inwc * INWC_TO_PA
}
pub fn pa_to_inwc(pa: f64) -> f64 {
    pa / INWC_TO_PA
}
pub fn ft_to_m(ft: f64) -> f64 {
    ft * FT_TO_M
}
pub fn m_to_ft(m: f64) -> f64 {
    m / FT_TO_M
}
pub fn in_to_m(inches: f64) -> f64 {
    inches * IN_TO_M
}
pub fn m_to_in(m: f64) -> f64 {
    m / IN_TO_M
}
pub fn fpm_to_ms(fpm: f64) -> f64 {
    fpm * FPM_TO_MS
}
pub fn ms_to_fpm(ms: f64) -> f64 {
    ms / FPM_TO_MS
}
pub fn f_to_c(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}
pub fn c_to_f(celsius: f64) -> f64 {
    celsius * 9.0 / 5.0 + 32.0
}

/// ACH = (flowrate × 3600) / room_volume.
pub fn air_changes_per_hour(flowrate_m3s: f64, volume_m3: f64) -> Result<f64> {
    if volume_m3 <= 0.0 {
        return Err("volume_m3 must be positive".into());
    }
    if flowrate_m3s < 0.0 {
        return Err("flowrate_m3s must be non-negative".into());
    }
    Ok(flowrate_m3s * 3600.0 / volume_m3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cfm_roundtrip() {
        let x = 500.0;
        assert!((m3s_to_cfm(cfm_to_m3s(x)) - x).abs() < 1e-9);
    }

    #[test]
    fn inwc_pa_value() {
        // 1 inWC ~ 249 Pa.
        assert!((inwc_to_pa(1.0) - 249.0889).abs() < 1e-9);
    }

    #[test]
    fn temperature_conversions() {
        assert!((c_to_f(100.0) - 212.0).abs() < 1e-9);
        assert!((f_to_c(212.0) - 100.0).abs() < 1e-9);
        assert!((c_to_f(0.0) - 32.0).abs() < 1e-9);
    }

    #[test]
    fn ach_value() {
        // 0.1 m^3/s in a 100 m^3 room = 3.6 ACH.
        assert!((air_changes_per_hour(0.1, 100.0).unwrap() - 3.6).abs() < 1e-9);
    }

    #[test]
    fn ach_rejects_bad_volume() {
        assert!(air_changes_per_hour(0.1, 0.0).is_err());
        assert!(air_changes_per_hour(-1.0, 100.0).is_err());
    }
}
