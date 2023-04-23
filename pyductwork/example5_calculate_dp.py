from example4_pass_flowrates import sup1, G
import networkx as nx
import matplotlib.pyplot as plt
from pprint import pprint


sup1.calculate_pressure_drops()
sup1.pass_pressure_drops_from_objects_to_graph()

if __name__ == "__main__":
    with open("diagnostics.log", "w") as log_file:
        pprint(sup1.objects, log_file)
    
    dp_labels = nx.get_node_attributes(G, name="pressure_drop")
    nx.draw(G, labels=dp_labels)
    plt.show()