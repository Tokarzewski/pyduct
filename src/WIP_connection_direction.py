from fittings import OneWayFitting, TwoWayFitting, ThreeWayFitting, FourWayFitting
from ducts import RigidDuct, RigidDuctType
from ductwork import Ductwork
import networkx as nx
import matplotlib.pyplot as plt

naw1 = Ductwork("Naw1", "Supply")
G = naw1.Graph

air_terminal = OneWayFitting("Air Terminal", 0.001)
cap = OneWayFitting("Cap", 0.0)
elbow = TwoWayFitting("Elbow")
branch = ThreeWayFitting("Branch")

ducttype1 = RigidDuctType("ductype1", "rectangular", 0.00009, None, 1, 1)
duct1 = RigidDuct("duct1", ducttype1, 10)

naw1.add_object("1", air_terminal)
naw1.add_object("2", duct1)
naw1.add_object("3", elbow)
naw1.add_object("4", branch)
naw1.add_object("5", branch)
naw1.add_object("6", duct1)

connections = (('1.1', '2.1'), ('2.2', '3.1'),
               ('3.2', '4.1'), ('4.2', '5.1'), ('5.2', '6.1'))
G.add_edges_from(connections)

nx.draw_networkx(G, pos=None, arrows=None, with_labels=True)
plt.show()
