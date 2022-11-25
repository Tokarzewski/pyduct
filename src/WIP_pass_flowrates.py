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
connections = (('1.1', '2.1'), ('2.2', '3.1'), ('3.2', '4.1'))

nx.set_node_attributes(G, 0, "flowrate")

# air_terminal
node1 = G.nodes["1"]
node1["flowrate"] = node1['object'].flowrate
y = node1['object'].flowrate

# air_terminal connector
node2 = G["1"]
dict1 = {"1.1": 0.001}
nx.set_node_attributes(G, dict1, "flowrate")

print(node2)
print(G.nodes(data="flowrate"))

G.add_edges_from(connections)
nx.draw_networkx(G, pos=None, arrows=None, with_labels=True)
plt.show()

#print("nodes:", G.nodes)
#print("edges:", G.edges)
#print(nx.shortest_path(G, '1.2', '2.1'))
