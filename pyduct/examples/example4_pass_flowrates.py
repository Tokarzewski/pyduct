import networkx as nx
from example3_object_defintion_and_connection_direction import sup1, G

sup1.pass_flowrate_through_graph()

if __name__ == "__main__":
    import matplotlib.pyplot as plt

    flowrate_labels = nx.get_node_attributes(G, name="flowrate")
    plt.figure(3)
    plt.title("Flowrate [m3/s]")
    nx.draw_networkx(G, labels=flowrate_labels, pos=nx.spring_layout(G, seed=0))
    plt.show()

    # print(G.nodes(data=True))
