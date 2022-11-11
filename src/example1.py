from friction import *

k = 0.09 / 1000  # m
d_h = 1  # m
v = 5  # m/s
L = 10  # m

Re = reynolds(v, d_h)
f = friction_coefficient(Re, k, d_h)
R = pressure_drop_per_meter(f, d_h, v)
linear_dp = linear_pressure_drop(R, L)

print(f"Re: {Re:.0f}")
print(f"f: {f}")
print(f"R: {R:.4f} Pa/m")
print(f"linear_dp: {linear_dp:.2f} Pa")

dzeta = 0.25

local_dp = local_pressure_drop(dzeta, v, rho=1.2)
print(f"local_dp: {local_dp:.2f} Pa")