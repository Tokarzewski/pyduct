import networkx as nx
import matplotlib.pyplot as plt
from example3_object_defintion_and_connection_direction import sup1

sup1.pass_flowrate()

if __name__ == "__main__":
    G = sup1.Graph
    flowrate_labels = nx.get_node_attributes(G, name="flowrate")
    nx.draw(G, labels=flowrate_labels)
    #print(G.nodes(data=True))
    plt.show()