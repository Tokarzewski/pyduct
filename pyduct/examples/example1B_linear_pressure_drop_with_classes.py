from pyduct.ducts import RigidDuctType, RigidDuct
from pyduct.connectors import Connector

# define objects
duct_type1 = RigidDuctType(name="ductype1", shape="rectangular", absolute_roughness=0.00009, height=1.0, width=1.0)
duct1 = RigidDuct(name="duct1", duct_type=duct_type1, length=10, flowrate=1)
duct1.connectors = [Connector(id="1", flowrate=5.0), Connector(id="2", flowrate=5.0)]

# calculate objects
duct_type1.calculate()
duct1.calculate()

for argument in ['name', "cross_sectional_area", "hydraulic_diameter"]:
    value = str(duct_type1.__getattribute__(argument))
    print(f"{argument}: {value}")

for argument in ['name', 'flowrate', 'velocity', 'roughness_correction_factor', 'length', 'pressure_drop_per_meter', 'linear_pressure_drop']:
    value = str(duct1.__getattribute__(argument))
    print(f"{argument}: {value}")