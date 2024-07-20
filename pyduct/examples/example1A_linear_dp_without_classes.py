from pyduct.physics.friction import *

k = 0.09 / 1000  # m
d_h = 0.4  # m
v = 5  # m/s
L = 10  # m
dzeta = 0.25  # -

Re = reynolds(v, d_h)
E = relative_roughness(k, d_h)
f = friction_coefficient(Re, E)
R = pressure_drop_per_meter(f, d_h, v)
linear_dp = linear_pressure_drop(R, L, Beta=1)
local_dp = local_pressure_drop(dzeta, v, rho=1.2)
flex_stretch_coef = flex_stretch_correction_factor(0.150, 70)

print(f"Re: {Re:.0f}")
print(f"f: {f:.6f}")
print(f"R: {R:.4f} Pa/m")
print(f"linear_dp: {linear_dp:.2f} Pa")
print(f"local_dp: {local_dp:.2f} Pa")
print(f"flex_stretch_coef: {flex_stretch_coef:.1f}")
