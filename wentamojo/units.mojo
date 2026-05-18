"""Unit converters between SI (wentamojo native) and US customary (Mojo port)."""


comptime CFM_TO_M3S: Float64 = 0.0004719474432    # ft^3/min → m^3/s
comptime INWC_TO_PA: Float64 = 249.0889            # inch H2O (4 °C) → Pa
comptime FT_TO_M: Float64 = 0.3048
comptime IN_TO_M: Float64 = 0.0254
comptime FPM_TO_MS: Float64 = 0.00508


def cfm_to_m3s(cfm: Float64) -> Float64:
    return cfm * CFM_TO_M3S


def m3s_to_cfm(m3s: Float64) -> Float64:
    return m3s / CFM_TO_M3S


def inwc_to_pa(inwc: Float64) -> Float64:
    return inwc * INWC_TO_PA


def pa_to_inwc(pa: Float64) -> Float64:
    return pa / INWC_TO_PA


def ft_to_m(ft: Float64) -> Float64:
    return ft * FT_TO_M


def m_to_ft(m: Float64) -> Float64:
    return m / FT_TO_M


def in_to_m(inches: Float64) -> Float64:
    return inches * IN_TO_M


def m_to_in(m: Float64) -> Float64:
    return m / IN_TO_M


def fpm_to_ms(fpm: Float64) -> Float64:
    return fpm * FPM_TO_MS


def ms_to_fpm(ms: Float64) -> Float64:
    return ms / FPM_TO_MS


def f_to_c(fahrenheit: Float64) -> Float64:
    return (fahrenheit - 32.0) * 5.0 / 9.0


def c_to_f(celsius: Float64) -> Float64:
    return celsius * 9.0 / 5.0 + 32.0


def air_changes_per_hour(flowrate_m3s: Float64, volume_m3: Float64) raises -> Float64:
    """ACH = (flowrate × 3600) / room_volume."""
    if volume_m3 <= 0.0:
        raise Error("volume_m3 must be positive")
    if flowrate_m3s < 0.0:
        raise Error("flowrate_m3s must be non-negative")
    return flowrate_m3s * 3600.0 / volume_m3
