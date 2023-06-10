from dataclasses import dataclass, field, replace
from typing import Literal
import networkx as nx
from .connectors import Connector

## Ductwork in NetowrkX
# All objects have connectors
# All objects and connectors are nodes in NetworkX
# Edges are the connections between the objects and connectors


def count_connectors(connectors):
    if type(connectors) == Connector:
        return 1
    else:
        return len(connectors)


@dataclass
class Ductwork:
    name: str
    type: Literal["Supply", "Exhaust"]
    objects: dict = field(default_factory=dict)
    Graph = nx.DiGraph()


    def add_object(self, id, object):
        # new instance of class
        object = replace(object)
        count = count_connectors(object.connectors)

        self.objects.update({id: object})
        self.Graph.add_edge(id, f"{id}.1")
        if count > 1:
            self.Graph.add_edges_from([[f"{id}.{x+1}", id] for x in range(1, count)])


    def pass_terminal_flowrate_from_object_to_graph(self):
        for id, object in self.objects.items():
            count = count_connectors(object.connectors)
            if count == 1:
                flowrate = object.connectors.flowrate
                self.Graph.nodes[id]["flowrate"] = flowrate
                self.Graph.nodes[f"{id}.1"]["flowrate"] = flowrate


    def pass_attribute_from_graph_nodes_to_objects(self, attribute):
        for object_id, object in self.objects.items():
            if count_connectors(object.connectors) > 1:
                for connector in object.connectors:
                    node_id = object_id + "." + connector.id
                    setattr(connector, attribute, self.Graph.nodes[node_id][attribute])
            else:
                connector = object.connectors
                node_id = object_id + "." + connector.id
                setattr(connector, attribute, self.Graph.nodes[node_id][attribute])


    def pass_flowrate_through_graph(self):
        # set empty flowrate attribute to all nodes
        nx.set_node_attributes(self.Graph, None, "flowrate")

        G = self.Graph
        terminals = {id for id, edges in G.degree() if edges == 1}
        fittings34 = {id for id, edges in G.degree() if edges > 2}

        self.pass_terminal_flowrate_from_object_to_graph()

        def pass_flowrate_from(fittings):
            dictionary = {}
            W = G.copy()
            W.remove_nodes_from(fittings34)
            for set in nx.weakly_connected_components(W):
                for fitting in fittings:
                    if fitting + ".1" in set:
                        flowrate = G.nodes[fitting + ".1"]["flowrate"]
                        dictionary.update({key: flowrate for key in set})
            nx.set_node_attributes(G, dictionary, "flowrate")

        def pass_flowrate_in_fittings(fittings):
            set1 = set()
            dictionary = dict()
            for fitting in fittings:
                c2 = G.nodes[fitting + ".2"]["flowrate"]
                c3 = G.nodes[fitting + ".3"]["flowrate"]
                if None not in (c2, c3):
                    flowrate = c2 + c3
                    set1.add(fitting)
                    dictionary.update({fitting: flowrate, fitting + ".1": flowrate})
            nx.set_node_attributes(G, dictionary, "flowrate")
            return set1

        pass_flowrate_from(terminals)
        remaining_fittings = fittings34

        while len(remaining_fittings) > 0:
            calculated_fittings = pass_flowrate_in_fittings(remaining_fittings)
            pass_flowrate_from(remaining_fittings)
            remaining_fittings = remaining_fittings - calculated_fittings

    def placeholder_calculate_dimmensions(self):
        3

    def calculate_pressure_drops(self):
        # calculate both linear and point pressure drops
        for object in self.objects.values():
            object.calculate()

    def pass_pressure_drops_from_objects_to_graph(self):
        # set pressure drop attribute to specific connector nodes
        # TODO create a single dictionary first and then update graph
        nx.set_node_attributes(self.Graph, None, "pressure_drop")
        connector_options = [1,2,3]
        dictionary = dict()
        for id, object in self.objects.items():
            count = count_connectors(object.connectors)
            if count == 1:
                self.Graph.nodes[f"{id}.1"]["pressure_drop"] = object.connectors.pressure_drop
            elif type(object).__name__ in ["RigidDuct", "FlexDuct"]:
                self.Graph.nodes[f"{id}.2"]["pressure_drop"] = object.linear_pressure_drop
            else:
                connectors = object.connectors
                for x in connector_options[1:count]:
                    key = f"{id}.{x}"
                    value = connectors[x-1].pressure_drop
                    dictionary.update({key: value})
        nx.set_node_attributes(self.Graph, dictionary, "pressure_drop")