from fittings import OneWayFitting, TwoWayFitting, ThreeWayFitting, FourWayFitting
from ducts import RigidDuct, RigidDuctType
from ductwork import Ductwork

naw1 = Ductwork("Naw1", "Supply")

air_terminal = OneWayFitting("Air Terminal")
cap = OneWayFitting("Cap")
elbow = TwoWayFitting("Elbow")
branch = ThreeWayFitting("Branch")

ducttype1 = RigidDuctType("ductype1", "rectangular", 0.00009, None, 1, 1)
duct1 = RigidDuct("duct1", ducttype1, 10, 5)

naw1.add_object(1, duct1)
naw1.add_object(2, cap)
naw1.add_object(3, elbow)
naw1.add_object(4, branch)

naw1.connect_objects_by_connectors("1.1", "2.1")

print("nodes:", naw1.network.nodes)
print("edges:", naw1.network.edges)