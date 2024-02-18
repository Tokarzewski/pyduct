from example7_calculate_dp import sup1, G
import networkx as nx
import matplotlib.pyplot as plt


nodes_on_critical_path = sup1.critical_path()
#print(nodes_on_critical_path)

color_map = []
for node in G:
    if node in nodes_on_critical_path:
        color_map.append('orange')
    else: 
        color_map.append('green')      

#nx.draw(G, node_color=color_map, with_labels=True)

if __name__ == "__main__":
    plt.figure(5)
    plt.title("Critical Path - Pressure drop [Pa]")
    dp_labels = nx.get_node_attributes(G, name="pressure_drop")
    dp_labels = {key: round(value, 2) for key, value in dp_labels.items()}
    nx.draw_networkx(G, node_color=color_map, labels=dp_labels, pos=nx.spring_layout(G, seed=0, center=(0, 0)))
    plt.show()