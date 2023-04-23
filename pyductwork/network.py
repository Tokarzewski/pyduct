from dataclasses import dataclass, field, replace
from typing import Literal
import networkx as nx
from connectors import Connector

## Ductwork in NetowrkX
# All objects have connectors
# All objects and connectors are nodes in NetworkX
# Edges are the connections between the connectors


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

        self.objects.update({id: object})
        count = count_connectors(object.connectors)
        if count == 1:
            self.Graph.add_edge(id, f"{id}.1")
        else:
            self.Graph.add_edge(id, f"{id}.1")
            connector_pairs = [(f"{id}.{x+1}", id) for x in range(1, count)]
            for id_x, id in connector_pairs:
                self.Graph.add_edge(id_x, id)

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

        self.pass_terminal_flowrate_from_object_to_graph()

        G = self.Graph
        air_terminals = set()
        fittings34 = set()

        for object_id, object in self.objects.items():
            if type(object).__name__ == "OneWayFitting":
                air_terminals.add(object_id)

        for x, y in G.degree():
            if y in (3, 4):
                fittings34.add(x)

        def pass_flowrate_from(fittings):
            dict1 = {}
            W = G.copy()
            W.remove_nodes_from(fittings34)
            for set in nx.weakly_connected_components(W):
                for fitting in fittings:
                    if fitting + ".1" in set:
                        flowrate = G.nodes[fitting + ".1"]["flowrate"]
                        dict1.update({key: flowrate for key in set})
            nx.set_node_attributes(G, dict1, "flowrate")

        def pass_flowrate_in_fittings(fittings):
            set1 = set()
            dictionary = dict()
            for fitting in fittings:
                c2 = G.nodes[fitting + ".2"]["flowrate"]
                c3 = G.nodes[fitting + ".3"]["flowrate"]
                if None not in (c2, c3):
                    if type(self.objects[fitting]).__name__ == "ThreeWayFitting":
                        flowrate = c2 + c3
                    elif type(self.objects[fitting]).__name__ == "FourWayFitting":
                        c4 = G.nodes[fitting + ".4"]["flowrate"]
                        flowrate = c2 + c3 + c4
                    set1.add(fitting)
                    dictionary.update({fitting + ".1": flowrate})
                    dictionary.update({fitting: flowrate})
            nx.set_node_attributes(G, dictionary, "flowrate")
            return set1

        pass_flowrate_from(air_terminals)
        remaining_fittings = fittings34

        fitting_count = len(fittings34)
        while fitting_count > 0:
            calculated_fittings = pass_flowrate_in_fittings(remaining_fittings)
            pass_flowrate_from(remaining_fittings)
            remaining_fittings = remaining_fittings - calculated_fittings
            fitting_count -= 1

    def calculate_dimmensions(self):
        3

    def calculate_pressure_drops(self):
        # calculate linear pressure drops
        for object in self.objects.values():
            if type(object).__name__ in ["RigidDuct", "FlexDuct"]:
                # print(object)
                object.calculate()

            # calculate local pressure drops
            if type(object).__name__ in [  #'OneWayFitting',
                "TwoWayFitting",
                "ThreeWayFitting",
                #'FourWayFitting'
            ]:
                object.calculate()

    def pass_pressure_drops_from_objects_to_graph(self):
        # set empty pressure drop attribute to all nodes 
        nx.set_node_attributes(self.Graph, None, "pressure_drop")

        for id, object in self.objects.items():
            count = count_connectors(object.connectors)
            if count == 1:
                pressure_drop = object.connectors.pressure_drop
                self.Graph.nodes[id]["pressure_drop"] = pressure_drop
                self.Graph.nodes[f"{id}.1"]["pressure_drop"] = pressure_drop
            if count == 2 and type(object).__name__ in ["RigidDuct", "FlexDuct"]:
                pressure_drop = object.linear_pressure_drop
                self.Graph.nodes[id]["pressure_drop"] = pressure_drop
            elif count == 2:
                pressure_drop = object.connectors[0].pressure_drop
                self.Graph.nodes[f"{id}.1"]["pressure_drop"] = pressure_drop
            if count == 3:
                connectors = object.connectors
                print(list)
                for x in [0, 1, 2]:
                    self.Graph.nodes[f"{id}.{x+1}"]["pressure_drop"] = connectors[x].pressure_drop