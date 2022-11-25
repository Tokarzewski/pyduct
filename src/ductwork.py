from dataclasses import dataclass
from typing import Literal
import networkx as nx
from copy import copy

## Ductwork in NetowrkX
# All objects are nodes
# Objects have connectors and they are nodes too
# Edges are the connections between the connectors 

def count_connectors(object):
    if hasattr(object, 'connector4'):
        return 4
    elif hasattr(object, 'connector3'):
        return 3
    elif hasattr(object, 'connector2'):
        return 2
    else:
        return 1

@dataclass
class Ductwork:
    name: str
    type: Literal["Supply", "Exhaust"]
    Graph = nx.DiGraph()

    def add_object(self, id, object):
        self.Graph.add_node(id, object=copy(object))
        count = count_connectors(object)
        if count == 1:
            self.Graph.add_edge(f'{id}.1', id)
        else:
            self.Graph.add_edge(f'{id}.1', id)
            connector_pairs = [(id, f'{id}.{x+1}') for x in range(1, count)]
            self.Graph.add_edges_from(connector_pairs)