from fittings import OneWayFitting, TwoWayFitting, ThreeWayFitting
from ducts import RigidDuct, RigidDuctType
from ductwork import Ductwork
import networkx as nx
import matplotlib.pyplot as plt

# ductwork system definition
sup1 = Ductwork("sup1", "Exhaust")
G = sup1.Graph

# define objects
air_terminal = OneWayFitting("Air Terminal", 5)
cap = OneWayFitting("Cap", 0)
elbow = TwoWayFitting("Elbow")
branch = ThreeWayFitting("Branch")
ducttype1 = RigidDuctType("ductype1", "rectangular", 0.00009, None, 1, 1)
duct1 = RigidDuct("duct1", ducttype1, 10)

# connect object connectors from air_terimnals to AHU
sup1.add_object("1", air_terminal)
sup1.add_object("2", duct1)
sup1.add_object("3", branch)
sup1.add_object("4", duct1)
sup1.add_object("5", branch)
sup1.add_object("6", air_terminal)
sup1.add_object("8", duct1)
sup1.add_object("9", air_terminal)
sup1.add_object("10", branch)
sup1.add_object("11", cap)
connections = [('1.1', '2.2'), ('2.1', '3.3'), ('3.1', '4.2'), ('8.1', '3.2'), 
('9.1', '8.2'), ('4.1', '5.2'), ('6.1', '5.3'), ('5.1', '10.2'), ('11.1', '10.3')]
G.add_edges_from(connections)

if __name__ == "__main__":
    nx.draw_networkx(G, with_labels=True)
    plt.show()