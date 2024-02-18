import networkx as nx

G = nx.DiGraph()

# option 1
for node in G.nodes():
            G.nodes[node].setdefault("flowrate", 0)

# option 2
nx.set_node_attributes(G, 0, name="flowrate")


# option 3
nx.set_node_attributes(G, dict, name="flowrate")
