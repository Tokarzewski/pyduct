from duct import *

ducttype1 = ducttype("ductype1", "rigid", "rectangular", 0.00009, None, 1, 1)
duct1 = duct("duct1", ducttype1, 10, 5)


for argument in ["area", "hydraulic_diameter"]:
    value = str(ducttype1.__getattribute__(argument))
    print(f"{argument}: {value}")

for argument in ["velocity", "pressure_drop_per_meter", "linear_pressure_drop"]:
    value = str(duct1.__getattribute__(argument))
    print(f"{argument}: {value}")