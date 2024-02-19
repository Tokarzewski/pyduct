from example4_pass_flowrates import sup1, G
import networkx as nx
import matplotlib.pyplot as plt
from pprint import pprint


sup1.calculate_pressure_drops()
dp_labels = nx.get_node_attributes(G, name="pressure_drop")

if __name__ == "__main__":
    with open("diagnostics.log", "w") as log_file:
        pprint(sup1, log_file)

    plt.figure(4)
    plt.title("Pressure drop [Pa]")

    # dp_labels = {k: v if v is not None else 0 for k, v in dp_labels.items()}
    dp_labels = {key: round(value, 2) for key, value in dp_labels.items()}
    nx.draw_networkx(
        G, labels=dp_labels, pos=nx.spring_layout(G, seed=0, center=(0, 0))
    )
    plt.show()
