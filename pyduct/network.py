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
    type: Literal["Supply", "Exhaust"]
    objects: dict = field(default_factory=dict)
    #connectors: dict = field(default_factory=dict)

    def __post_init__(self):
        self.graph = nx.DiGraph()
            
    def add_object(self, id, obj):
        obj = replace(obj)
        self.objects[id] = obj

        """for c_id, connector in enumerate(obj.connectors, start=1):
            connector.id = f"{id}.{c_id}"
            self.connectors[f"{id}.{c_id}"] = connector
            self.add_connectors(id, obj)
        """
        
        # adding connectors to Graph
        self.graph.add_node(id, name=obj.name) # move this to passing attributes from objects to graphs
        self.graph.add_edge(id, f"{id}.1")

        connectors_count = len(obj.connectors)
        if connectors_count > 1:
            
            edges = [(f"{id}.{x+1}", id) for x in range(1, connectors_count)]
            self.graph.add_edges_from(edges)
        else:
            self.graph.nodes[f"{id}.1"]["flowrate"] = obj.connectors[0].flowrate
                

    def pass_attribute_from_graph_to_objects(self, attribute):
        for id, object in self.objects.items():
            for connector in object.connectors:
                node_id = f"{id}.{connector.id}"
                setattr(connector, attribute, self.graph.nodes[node_id][attribute])

    def nodes_without_attribute(self, attribute):
        return [node for node in self.graph.nodes()
            if attribute not in self.graph.nodes[node]]

    def pass_flowrate_through_graph(self):
        # set flowrate attribute to all non air teminals nodes
        nodes_without_flowrate = self.nodes_without_attribute("flowrate")
        subgraph_without_florate = self.graph.subgraph(nodes_without_flowrate)
        nx.set_node_attributes(subgraph_without_florate, None, "flowrate")

        terminals = {id for id, edges in self.graph.degree() if edges == 1}
        fittings34 = {id for id, edges in self.graph.degree() if edges > 2}

        def pass_flowrate_in_chains(fittings):
            dictionary = {}
            W = self.graph.copy()
            W.remove_nodes_from(fittings34)
            for components in nx.weakly_connected_components(W):
                for fitting in fittings:
                    if fitting + ".1" in components:
                        flowrate = self.graph.nodes[fitting + ".1"]["flowrate"]
                        dictionary.update({key: flowrate for key in components})
            nx.set_node_attributes(self.graph, dictionary, "flowrate")

        def calc_flowrate_in_graph_fittings(fittings):
            updated_fittings = set()
            flowrate_dict = dict()
            for fitting in fittings:
                c2 = self.graph.nodes[fitting + ".2"]["flowrate"]
                c3 = self.graph.nodes[fitting + ".3"]["flowrate"]
                if None not in (c2, c3):
                    flowrate = c2 + c3
                    updated_fittings.add(fitting)
                    flowrate_dict.update({fitting: flowrate, fitting + ".1": flowrate})
            nx.set_node_attributes(self.graph, flowrate_dict, "flowrate")
            return updated_fittings

        pass_flowrate_in_chains(terminals)
        remaining_fittings = fittings34

        while len(remaining_fittings) > 0:
            calculated_fittings = calc_flowrate_in_graph_fittings(remaining_fittings)
            pass_flowrate_in_chains(remaining_fittings)
            remaining_fittings -= calculated_fittings

        self.pass_attribute_from_graph_to_objects("flowrate")

    def placeholder_calculate_dimmensions(self):
        # 1. velocity method
        # 2. pressure drop per unit length method
        pass

    def calculate_pressure_drops(self):
        # calculate both linear and point pressure drops
        for obj in self.objects.values():
            obj.calculate()
        self.pass_pressure_drops_from_objects_to_graph()

    def pass_pressure_drops_from_objects_to_graph(self):
        # set pressure drop attribute in connector nodes
        # TODO create a single dictionary first and then update graph
        nx.set_node_attributes(self.graph, 0, "pressure_drop")
        dictionary = dict()

        for id, obj in self.objects.items():
            connectors = obj.connectors
            if type(obj).__name__ in ["RigidDuct", "FlexDuct"]:
                self.graph.nodes[f"{id}.1"]["pressure_drop"] = obj.linear_pressure_drop
            else:
                for connector_id, connector in enumerate(connectors, start=1):
                    key = f"{id}.{connector_id}"
                    dictionary[key] = connector.pressure_drop

        nx.set_node_attributes(self.graph, dictionary, "pressure_drop")
