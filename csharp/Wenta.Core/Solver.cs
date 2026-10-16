using System;
using System.Collections.Generic;

namespace Wenta
{
    public static class Solver
    {
        public static void PropagateFlowrates(Network network)
        {
            List<string> topo = network.TopoOrder();

            foreach (var kv in network.Components)
            {
                network.SetNodeFlow(kv.Key, 0.0);
                foreach (Port p in kv.Value.Ports)
                    network.SetNodeFlow(p.NodeId, 0.0);
            }

            foreach (Terminal term in network.Terminals())
                network.SetNodeFlow(term.Ports[0].NodeId, term.Flowrate);

            for (int i = topo.Count - 1; i >= 0; i--)
            {
                string node = topo[i];
                double flow = network.GetNodeFlow(node);
                if (flow != 0.0)
                {
                    foreach (string pred in network.Predecessors(node))
                        network.SetNodeFlow(pred, network.GetNodeFlow(pred) + flow);
                }
            }

            foreach (var kv in network.Components)
                foreach (Port p in kv.Value.Ports)
                    p.Flowrate = network.GetNodeFlow(p.NodeId);
        }

        public static void ComputePressureDrops(Network network, Fluid fluid)
        {
            foreach (var kv in network.Components)
                kv.Value.Compute(fluid);

            foreach (var kv in network.Components)
                foreach (Port p in kv.Value.Ports)
                {
                    network.SetNodeDp(p.NodeId, p.PressureDrop);
                }
        }

        public static List<string> CriticalPath(Network network)
        {
            List<string> topo = network.TopoOrder();
            var dist = new Dictionary<string, double>(topo.Count);
            var prev = new Dictionary<string, string>(topo.Count);
            foreach (string n in topo)
            {
                List<string> preds = network.Predecessors(n);
                string bestP = null;
                double bestD = 0.0;
                if (preds.Count == 1)
                {
                    bestP = preds[0];
                    bestD = dist[bestP];
                }
                else if (preds.Count > 1)
                {
                    foreach (string p in preds)
                    {
                        double d = dist[p];
                        if (bestP == null || d > bestD) { bestP = p; bestD = d; }
                    }
                }
                prev[n] = bestP;
                dist[n] = bestD + network.GetNodeDp(n);
            }
            string end = null;
            double endD = double.NegativeInfinity;
            foreach (var kv in dist)
                if (kv.Value > endD) { endD = kv.Value; end = kv.Key; }
            if (end == null) return new List<string>();
            var path = new List<string>();
            string cur = end;
            while (cur != null)
            {
                path.Add(cur);
                cur = prev[cur];
            }
            path.Reverse();
            return path;
        }

        public static double CriticalPathPressureDrop(Network network)
        {
            List<string> path = CriticalPath(network);
            double sum = 0.0;
            foreach (string nodeId in path)
                sum += network.GetNodeDp(nodeId);
            return sum;
        }

        public static double Solve(Network network, Fluid fluid)
        {
            PropagateFlowrates(network);
            ComputePressureDrops(network, fluid);
            return CriticalPathPressureDrop(network);
        }
    }
}
