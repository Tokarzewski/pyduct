using System;
using System.Collections.Generic;
using System.Globalization;

namespace Wenta
{
    public sealed class Mark
    {
        public uint BranchNo;
        public string ComponentId;
        public string Kind;
        public double? SizeMm;
        public double? FlowM3s;
    }

    public static class Marking
    {
        public static List<Mark> AssignBranchMarks(Network network)
        {
            var marks = new List<Mark>();
            if (network.Components.Count == 0)
                return marks;

            var seeds = new List<string>();
            foreach (var kv in network.Components)
                if (kv.Value is Source) seeds.Add(kv.Key);
            seeds.Sort(StringComparer.Ordinal);

            var visited = new HashSet<string>();
            var queue = new Queue<string>(seeds);
            uint nextBranch = 1;

            while (queue.Count > 0)
            {
                string cid = queue.Dequeue();
                if (!visited.Add(cid)) continue;

                Component component;
                if (!network.Components.TryGetValue(cid, out component)) continue;

                RigidDuct duct = component as RigidDuct;
                if (duct != null)
                {
                    marks.Add(new Mark
                    {
                        BranchNo = nextBranch,
                        ComponentId = cid,
                        Kind = component.GetType().Name,
                        SizeMm = Math.Round(duct.CrossSection.HydraulicDiameter * 1000.0),
                        FlowM3s = component.InletFlowrate(),
                    });
                    nextBranch++;
                }

                var downstream = new SortedSet<string>(StringComparer.Ordinal);
                foreach (Port port in component.Outlets)
                {
                    foreach (string node in network.Successors(port.NodeId))
                    {
                        int colon = node.IndexOf(':');
                        if (colon < 0) continue;
                        string ncid = node.Substring(0, colon);
                        if (ncid != cid && !visited.Contains(ncid))
                            downstream.Add(ncid);
                    }
                }
                foreach (string ncid in downstream)
                    queue.Enqueue(ncid);
            }

            return marks;
        }

        public static string MarksAsCsv(IList<Mark> marks)
        {
            var lines = new List<string> { "branch_no,component_id,kind,size_mm,flow_m3s" };
            foreach (Mark m in marks)
            {
                lines.Add(string.Join(",", new[]
                {
                    m.BranchNo.ToString(CultureInfo.InvariantCulture),
                    m.ComponentId,
                    m.Kind,
                    Results.FmtOpt(m.SizeMm),
                    Results.FmtOpt(m.FlowM3s),
                }));
            }
            return string.Join("\n", lines.ToArray());
        }
    }
}
