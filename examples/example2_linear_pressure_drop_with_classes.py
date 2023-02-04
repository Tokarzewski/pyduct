from pyductwork.ducts import *

# define objects
duct_type1 = RigidDuctType("ducttype1", "rectangular", 0.00009, None, 1, 1)
duct1 = RigidDuct("duct1", duct_type1, 10, 5)

# calculate objects
duct_type1.calculate()
duct1.calculate()

for argument in ['name', "cross_sectional_area", "hydraulic_diameter"]:
    value = str(duct_type1.__getattribute__(argument))
    print(f"{argument}: {value}")

for argument in ['name', 'velocity', 'roughness_correction_factor', 'length', 'pressure_drop_per_meter', 'linear_pressure_drop']:
    value = str(duct1.__getattribute__(argument))
    print(f"{argument}: {value}")