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
    
    # round pressure drop values
    digit_precision = 2
    dp_labels = {key : round(value, digit_precision) for key, value in dp_labels.items() if value is not None}
    nx.draw(G, labels=dp_labels)
    plt.show()