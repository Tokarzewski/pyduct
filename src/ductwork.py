from dataclasses import dataclass, field
from typing import Literal
import networkx as nx

## Ductwork in NetowrkX
# All objects are in nodes
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
    objects: dict = field(default_factory=dict)
    Graph = nx.DiGraph()

    def add_object(self, id, object):
        # XYZ remove flowrate definition from this function
        self.objects.update({id: object})
        count = count_connectors(object)
        if count == 1:
            self.Graph.add_node(id, flowrate=object.flowrate)
            self.Graph.add_node(f'{id}.1', flowrate=object.flowrate)
            self.Graph.add_edge(id, f'{id}.1')
        else:
            self.Graph.add_node(id, flowrate=None)
            self.Graph.add_node(f'{id}.1', flowrate=None)
            self.Graph.add_edge(id, f'{id}.1')
            connector_pairs = [(f'{id}.{x+1}', id) for x in range(1, count)]
            for id_x, id in connector_pairs:
                self.Graph.add_node(id_x, flowrate=None)
                self.Graph.add_edge(id_x, id)

    def pass_flowrate(self):

        G = self.Graph
        air_terminals = set()
        fittings34 = set()

        for id, object in self.objects.items():
            if type(object).__name__ == "OneWayFitting":
                air_terminals.add(id)

        for x, y in G.degree():
            if y in (3, 4):
                fittings34.add(x)

        def pass_flowrate_from(fittings):
            # pass flowrate
            dict1 = {}
            W = G.copy()
            W.remove_nodes_from(fittings34)
            for set in nx.weakly_connected_components(W):
                for fitting in fittings:
                    if fitting+'.1' in set:
                        flowrate = G.nodes[fitting+'.1']['flowrate']
                        dict1.update({key: flowrate for key in set})
            nx.set_node_attributes(G, dict1, "flowrate")

        def pass_flowrate_in_fittings(fittings):
            set1 = set()
            dictionary = dict()
            for fitting in fittings:
                c2 = G.nodes[fitting+'.2']["flowrate"]
                c3 = G.nodes[fitting+'.3']["flowrate"]
                if None not in (c2, c3):
                    if type(self.objects[fitting]).__name__ == "ThreeWayFitting":
                        flowrate = c2 + c3
                    elif type(self.objects[fitting]).__name__ == "FourWayFitting":
                        c4 = G.nodes[fitting+'.4']["flowrate"]
                        flowrate = c2 + c3 + c4
                    set1.add(fitting)
                    dictionary.update({fitting+'.1': flowrate})
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

        # pass flowrate from duct nodes to duct objects
        for id, object in self.objects.items():
            if type(object).__name__ in ['RigidDuct', 'FlexDuct']:
                object.flowrate = self.Graph.nodes[id]['flowrate']

    def calculate(self):
        # calculate linear pressure drops
        for object in self.objects.values():
            if type(object).__name__ in ['RigidDuct', 'FlexDuct']:
                object.calculate()

        # calculate local pressure drops
        """     
        for object in self.objects.values()
            if type(object).__name__ in ['OneWayFitting', 'TwoWayFitting', 'ThreeWayFitting', 'FourWayFitting']:
                object.calculate()
        """
