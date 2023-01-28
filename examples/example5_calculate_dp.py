from example4_pass_flowrates import sup1
from pprint import pprint

# print(G.nodes(data="flowrate"))
# print("nodes:", G.nodes)
# print("edges:", G.edges)
# print(nx.shortest_path(G, '1.2', '2.1'))

sup1.calculate()

if __name__ == "__main__":
    pprint(sup1.objects)
