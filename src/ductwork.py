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
    elif hasattr(object, 'connector1'):
        return 1
    else:
        return 0

@dataclass
class Ductwork:
    name: str
    type: Literal["Supply", "Exhaust"]
    network = nx.Graph()

    def add_object(self, id, object):
        self.network.add_node(id, object=copy(object))
        count = count_connectors(object)
        if count == 1:
            self.network.add_edge(id, f'{id}.1')
        else:
            connector_pairs = [(id, f'{id}.{x+1}') for x in range(count)]
            self.network.add_edges_from(connector_pairs)

    def connect_objects_by_connectors(self, connector1_id, connector2_id):
        self.network.add_edge(connector1_id, connector2_id)
