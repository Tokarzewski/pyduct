import networkx as nx
import matplotlib.pyplot as plt
from example3_object_defintion_and_connection_direction import sup1

if __name__ == "__main__":
    # pass flowrates from air terminals down to last object
    sup1.pass_flowrate()
    G = sup1.Graph
    flowrate_labels = nx.get_node_attributes(G, name="flowrate")
    nx.draw(G, labels=flowrate_labels, with_labels=True)
    plt.show()
