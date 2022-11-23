from dataclasses import dataclass
from typing import Literal
import networkx as nx
from copy import copy

## Ductwork in NetowrkX
# Fittings are nodes
# Ducts can be edges, but not all edges are ducts


@dataclass
class Ductwork:
    name: str
    type: Literal["Supply", "Exhaust"]
    network = nx.Graph()

    def add_OneWayFitting(self, id, OneWayFitting):
        self.network.add_node(id, object=copy(OneWayFitting))

    def add_Duct(self, fitting1, fitting2, duct):
        self.network.add_edge(fitting1, fitting2, object=duct)

    def calculate(self):
        # for node in self.network.nodes:
        #    node.calculate()
        for u, v, data in self.network.edges(data=True):
            edge_object = data["object"]
            edge_object.duct_type.calculate()
            edge_object.calculate()
