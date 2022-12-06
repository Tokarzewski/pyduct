from dataclasses import dataclass
from typing import Literal
import networkx as nx
from copy import copy
from fittings import OneWayFitting, TwoWayFitting, ThreeWayFitting, FourWayFitting

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
        count = count_connectors(object)
        if count == 1:
            self.Graph.add_node(id, object=copy(object),
                                flowrate=object.flowrate)
            self.Graph.add_node(f'{id}.1', flowrate=object.flowrate)
            self.Graph.add_edge(id, f'{id}.1')
        else:
            self.Graph.add_node(id, object=copy(object), flowrate=None)
            self.Graph.add_node(f'{id}.1', flowrate=None)
            self.Graph.add_edge(id, f'{id}.1')
            connector_pairs = [(f'{id}.{x+1}', id) for x in range(1, count)]
            for id_x, id in connector_pairs:
                self.Graph.add_node(id_x, flowrate=None)
                self.Graph.add_edge(id_x, id)

    def pass_flowrate(self):

        G = self.Graph

        air_terminals = set()
        for x, y in G.degree():
            if y == 1 and 'object' in G.nodes[x]:
                air_terminals.add(x)

        fittings34 = set()
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
            # calculate flowrate in fittings
            set1 = set()
            dictionary = dict()
            for fitting in fittings:
                c2 = G.nodes[fitting+'.2']["flowrate"]
                c3 = G.nodes[fitting+'.3']["flowrate"]
                if None not in (c2, c3):
                    if type(G.nodes[fitting]['object']) == ThreeWayFitting:
                        flowrate = c2 + c3
                    elif type(G.nodes[fitting]['object']) == FourWayFitting and G.nodes[fitting+'.4']["flowrate"] != None:
                        c4 = G.nodes[fitting+'.4']["flowrate"]
                        flowrate = c2 + c3 + c4
                    set1.add(fitting)
                    dictionary.update({fitting+'.1': flowrate})
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
