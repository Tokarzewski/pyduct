import fittings
import ducts
import networkx as nx

## Ductwork in NetowrkX
# Fittings are nodes
# Ducts can be edges, but not all edges are ducts

G = nx.Graph()

cap = fittings.OneWayFitting("Cap")
elbow = fittings.TwoWayFitting("Elbow")
ducttype1 = ducts.RigidDuctType("ductype1", "rectangular", 0.00009, None, 1, 1)
duct1 = ducts.RigidDuct("duct1", ducttype1, 10, 5)

# calculate objects
ducttype1.calculate()
duct1.calculate()

G.add_node(1, object=cap)
G.add_node(2, object=elbow)
G.add_edge(1, 2, object=duct1)

print("nodes:", G.nodes)
print("edges:", G.edges)
print(G.edges[1,2]["object"].name)