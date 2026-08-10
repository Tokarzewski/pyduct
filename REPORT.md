# venti — Sound-Calculation Engine (Issue #5 / pyduct)

Implemented a dependency-free **sound-calculation** module for the `venti` ductwork
library, mirroring CADvent's "calculation of sound" feature: airflow-regenerated
duct noise, the room equation turning duct sound power into an audible space
level, and NC (Noise Criterion) compliance checking.

## What was added

- **`venti/src/sound.rs`** — new module, pure `f64` math, **no extra crates**.
  - `regenerated_noise_round(velocity, diameter, density?) -> Result<f64, String>`
  - `duct_pressure_level(sound_power_db, room_surface_area, absorption_coefficient) -> Result<f64, String>`
  - `nc_ok(space_type, level_db) -> Result<bool, String>`
  - `nc_ok_target(nc_target, level_db) -> bool`
  - `NOISE_LIMITS_NC` constant (parallel to `sizing::NOISE_LIMITS_M_S`).
- **`venti/src/lib.rs`** — registered `pub mod sound;` and re-exported all public
  functions (`regenerated_noise_round`, `duct_pressure_level`, `nc_ok`,
  `nc_ok_target`, `NOISE_LIMITS_NC`) at the crate root.

## Formulas & documentation

### Regenerated noise — `regenerated_noise_round`
Duct airflow turbulence radiates acoustic power scaling like Lighthill's subsonic
`v⁶` law; for a fixed velocity the noise falls as the duct grows (turbulent energy
spread over a larger, thinner boundary layer):

```text
Lw [dB re 1e-12 W] = C + 10·log10(ρ/ρ₀) + 60·log10(v) − 20·log10(d)
```
- `v` [m/s], `d` [m], `ρ` [kg/m³] (defaults to `STANDARD_AIR` = 1.204), `ρ₀` = 1.204,
  `C` = 10 dB calibration offset.
- Grows with `v⁶`, falls with `1/d²` — the behaviour asserted in the unit tests.

### Domain transducer — `duct_pressure_level`
Diffuse-field "room equation" (ISO 3740 family / acoustic room theory) converting
duct sound *power* into the reverberant sound *pressure* level a person hears:

```text
Lp [dB re 20 µPa] = Lw + 10·log10( 4·(1 − α) / (α·S) )
```
- `S` = total room surface area [m²], `α` = average Sabine absorption coefficient ∈ (0,1).
- More absorption ⇒ lower SPL (monotonic, tested).

### NC compliance — `nc_ok` / `nc_ok_target`
`nc_ok` looks up `NOISE_LIMITS_NC` (studio 25, bedroom 25, office 35, classroom 35,
retail 40, industrial 60) and returns `level_db <= limit`. `nc_ok_target` takes a
numeric NC target instead and cannot fail.

## Tests

`sound.rs` adds **10 unit tests** with closed-form sanity checks:

| Test | Check |
|------|-------|
| `regenerated_noise_grows_with_velocity` | v↑ ⇒ dB↑, exact `+60·log10(v-ratio)` |
| `regenerated_noise_falls_with_diameter` | d↑ ⇒ dB↓, exact `−20·log10(d-ratio)` |
| `regenerated_noise_closed_form` | exact `10 + 60·log10(v) − 20·log10(d)` |
| `regenerated_noise_higher_density_is_louder` | ρ↑ ⇒ dB↑ |
| `regenerated_noise_rejects_bad_inputs` | non-positive v/d/ρ rejected |
| `duct_pressure_level_closed_form` | exact room equation |
| `duct_pressure_level_more_absorption_is_quieter` | α↑ ⇒ SPL↓ |
| `duct_pressure_level_rejects_bad_inputs` | S≤0 / α∉(0,1) rejected |
| `nc_ok_boundary` | ==limit passes, +0.1 dB fails |
| `nc_ok_lookup_and_target` | lookup vs numeric target agreement, unknown space err |

## Test output

```
running 65 tests
...
test result: ok. 65 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
(65 = original 55 + 10 new sound tests; 5 integration + 1 doctest also pass.
`cargo clippy --all-targets` is warning-free.)
