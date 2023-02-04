from pyductwork.connectors import Connector
from pyductwork.ducts import RigidDuct, RigidDuctType
from pyductwork.fitting_types import type1
from pyductwork.fittings import OneWayFitting, ThreeWayFitting, TwoWayFitting
from pyductwork.network import Ductwork
import networkx as nx
import matplotlib.pyplot as plt

# define ductwork
sup1 = Ductwork("sup1", "Supply")
G = sup1.Graph

# define objects
air_terminal = OneWayFitting("Air Terminal", Connector(id="1", flowrate=5))
cap = OneWayFitting("Cap", Connector(flowrate=0))

duct_type1 = RigidDuctType(
    name="ductype1", shape="rectangular", absolute_roughness=0.00009, height=1, width=1
)
duct1 = RigidDuct(name="duct1", duct_type=duct_type1, length=10)

elbow_type = type1(name="elbow", bend_radius=1, diameter=1, angle=90)
elbow = TwoWayFitting("Elbow", elbow_type)

branch = ThreeWayFitting("Branch")

# add objects to ductowrk
sup1.add_object("1", air_terminal)
sup1.add_object("2", duct1)
sup1.add_object("3", branch)
sup1.add_object("4", duct1)
sup1.add_object("5", branch)
sup1.add_object("6", branch)
sup1.add_object("8", elbow)
sup1.add_object("9", air_terminal)
sup1.add_object("10", air_terminal)
sup1.add_object("11", cap)

# define connections
# XYZ limitation - they must start from 1
connections = [
    ("1.1", "2.2"),
    ("2.1", "3.3"),
    ("3.1", "4.2"),
    ("8.1", "3.2"),
    ("9.1", "8.2"),
    ("4.1", "5.2"),
    ("10.1", "5.3"),
    ("5.1", "6.2"),
    ("11.1", "6.3"),
]
G.add_edges_from(connections)

if __name__ == "__main__":
    nx.draw_networkx(G, with_labels=True)
    plt.show()
