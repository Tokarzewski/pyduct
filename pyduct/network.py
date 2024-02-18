from dataclasses import dataclass, field, replace
from typing import Literal
import networkx as nx

# All objects have connectors
# All objects and connectors are nodes in NetworkX
# Edges are the connections between the objects and connectors
# All calculations are in nodes and never in edges


@dataclass
class Ductwork:
    name: str
    type: Literal["supply", "exhaust"]
    objects: dict = field(default_factory=dict)
    connectors: dict = field(default_factory=dict)
    graph = nx.DiGraph()

    def add_object(self, id, obj):
        obj = replace(obj)
        self.objects[id] = obj

        # add connectors
        for c_id, connector in enumerate(obj.connectors, start=1):
            connector.id = f"{id}.{c_id}"
            self.connectors[f"{id}.{c_id}"] = connector

        self.graph.add_edge(id, f"{id}.1")
        connectors_count = len(obj.connectors)
        if connectors_count > 1:
            edges = [(f"{id}.{x+1}", id) for x in range(1, connectors_count)]
            self.graph.add_edges_from(edges)
        else: 
            self.graph.nodes[id]["flowrate"] = obj.connectors[0].flowrate

    def pass_attribute_from_graph_to_connectors(self, attribute):
        for id, connector in self.connectors.items():
            if attribute in self.graph.nodes[id]:
                setattr(connector, attribute, self.graph.nodes[id][attribute])

    def pass_attribute_from_connectors_to_graph(self, attribute):
        for id, connector in self.connectors.items():
            if getattr(connector, attribute) != None:
                self.graph.nodes[id][attribute] = getattr(connector, attribute)

    def pass_flowrate_through_graph(self):
        G = self.graph

        # Set flowrate to 0 for nodes without flowrate attribute
        for node in G.nodes():
            G.nodes[node].setdefault("flowrate", 0)

        # Propagate flowrate to successors
        for node in nx.topological_sort(G):
            for successor in G.successors(node):
                G.nodes[successor]['flowrate'] += G.nodes[node]['flowrate']
        
        # Pass flowrate attribute from graph to connectors
        self.pass_attribute_from_graph_to_connectors("flowrate")

    def placeholder_calculate_dimmensions(self):
        # 1. velocity method
        # 2. pressure drop per unit length method
        pass

    def calculate_pressure_drops(self):
        """calculates linear and point pressure drops"""
        for obj in self.objects.values():
            obj.calculate()
        nx.set_node_attributes(self.graph, 0, "pressure_drop")
        self.pass_attribute_from_connectors_to_graph("pressure_drop")
