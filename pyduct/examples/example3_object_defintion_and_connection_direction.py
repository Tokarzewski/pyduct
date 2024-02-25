from pyduct.ducts import RigidDuct, RigidDuctType
from pyduct.fitting_types import elbow_round
from pyduct.fittings import OneWayFitting, TwoWayFitting, ThreeWayFitting
from pyduct.network import Ductwork
import networkx as nx
import matplotlib.pyplot as plt

# define ductwork
sup1 = Ductwork("ETA1", "supply")
G = sup1.graph

# define objects
air_terminal1 = OneWayFitting(name="air terminal_1", flowrate=5)
air_terminal2 = OneWayFitting(name="air terminal_2", flowrate=7)
cap = OneWayFitting("cap", flowrate=0)

duct_type1 = RigidDuctType(name="ductype1", diameter=1)
duct1 = RigidDuct(name="duct1", duct_type=duct_type1, length=10)

elbow_type = elbow_round(bend_radius=1, diameter=1, angle=90)
elbow = TwoWayFitting(name="elbow round", fitting_type=elbow_type)

# branch_type1= XYZ
branch = ThreeWayFitting("branch")

# add objects to ductwork
sup1.add_object("1", air_terminal1)
sup1.add_object("4", duct1)
sup1.add_object("5", branch)
sup1.add_object("6", branch)
sup1.add_object("8", elbow)
sup1.add_object("10", air_terminal2)
sup1.add_object("11", cap)

# convention - all connections must start from source connector - C.1
connections = [
    ("1.1", "8.2"),
    ("8.1", "4.2"),
    ("4.1", "5.2"),
    ("10.1", "5.3"),
    ("5.1", "6.2"),
    ("11.1", "6.3"),
]
G.add_edges_from(connections)


if __name__ == "__main__":
    plt.figure(1)
    plt.title("IDs")
    nx.draw_networkx(G, pos=nx.spring_layout(G, seed=0, center=(0, 0)))

    plt.figure(2)
    # name_labels = nx.get_node_attributes(G, name="name")
    name_labels = {id: obj.name for (id, obj) in sup1.objects.items()}
    plt.title("Name")

    # self.graph.add_node(id, name=obj.name)
    nx.draw_networkx(
        G, labels=name_labels, pos=nx.spring_layout(G, seed=0, center=(0, 0))
    )
    plt.show()
