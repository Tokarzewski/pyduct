from ductwork.ducts import *

# define objects
ducttype1 = RigidDuctType("ducttype1", "rectangular", 0.00009, None, 1, 1)
duct1 = RigidDuct("duct1", ducttype1, 10, 5)

# calculate objects
ducttype1.calculate()
duct1.calculate()

for argument in ['name', "cross_sectional_area", "hydraulic_diameter"]:
    value = str(ducttype1.__getattribute__(argument))
    print(f"{argument}: {value}")

for argument in ['name', 'velocity', 'roughness_correction_factor', 'length', 'pressure_drop_per_meter', 'linear_pressure_drop']:
    value = str(duct1.__getattribute__(argument))
    print(f"{argument}: {value}")