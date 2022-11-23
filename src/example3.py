from fittings import OneWayFitting
from ducts import RigidDuct, RigidDuctType
from ductwork import Ductwork 

naw1 = Ductwork("Naw1", "Supply")

air_terminal = OneWayFitting("Air Terminal")
cap = OneWayFitting("Cap")

ducttype1 = RigidDuctType("ductype1", "rectangular", 0.00009, None, 1, 1)
duct1 = RigidDuct("duct1", ducttype1, 10, 5)

naw1.add_OneWayFitting(1, air_terminal)
naw1.add_OneWayFitting(2, cap)
naw1.add_Duct(1, 2, duct1)

naw1.calculate()

print("nodes:", naw1.network.nodes)
print("edges:", naw1.network.edges)
print(naw1.network.nodes(data=True))
print(naw1.network.edges(data=True))