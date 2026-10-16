using System;
using System.Collections.Generic;
using System.Globalization;

namespace Wenta
{
    public sealed class PressureReportRow
    {
        public string ComponentId;
        public string Kind;
        public string Port;
        public double? FlowM3s;
        public double? VelocityMs;
        public double DpPa;
        public double CumulativeDpPa;
        public double SharePercent;
    }

    public sealed class PressureReport
    {
        public string NetworkName;
        public double CriticalDpPa;
        public List<PressureReportRow> Rows = new List<PressureReportRow>();

        public static readonly string[] Fields =
        {
            "component_id",
            "kind",
            "port",
            "flow_m3s",
            "velocity_ms",
            "dp_pa",
            "cumulative_dp_pa",
            "share_percent",
        };

        public static PressureReport Build(Network solvedNetwork)
        {
            if (solvedNetwork == null)
                throw new WentaException("network is null");
            if (solvedNetwork.Sources().Count == 0)
                throw new WentaException("network has no Source component");
            if (solvedNetwork.Terminals().Count == 0)
                throw new WentaException("network has no Terminal component");

            var report = new PressureReport { NetworkName = solvedNetwork.Name ?? "" };

            List<string> path = Solver.CriticalPath(solvedNetwork);
            ExtendToTerminal(solvedNetwork, path);
            double cumulative = 0.0;
            foreach (string nodeId in path)
            {
                int colon = nodeId.IndexOf(':');
                if (colon < 0) continue;

                string cid = nodeId.Substring(0, colon);
                string portName = nodeId.Substring(colon + 1);
                Component component;
                if (!solvedNetwork.Components.TryGetValue(cid, out component))
                    throw new WentaException("critical path references unknown component '" + cid + "'");
                Port port = component.Port_(portName);

                double dp = solvedNetwork.NodeDp(nodeId);
                cumulative += dp;
                report.Rows.Add(new PressureReportRow
                {
                    ComponentId = cid,
                    Kind = component.GetType().Name,
                    Port = portName,
                    FlowM3s = port.Flowrate,
                    VelocityMs = port.Flowrate.HasValue ? port.Velocity : null,
                    DpPa = dp,
                    CumulativeDpPa = cumulative,
                });
            }

            report.CriticalDpPa = cumulative;
            foreach (PressureReportRow row in report.Rows)
                row.SharePercent = cumulative != 0.0 ? row.DpPa / cumulative * 100.0 : 0.0;
            return report;
        }

        private static void ExtendToTerminal(Network network, List<string> path)
        {
            if (path.Count == 0) return;
            string end = path[path.Count - 1];
            if (IsTerminalNode(network, end)) return;

            var prev = new Dictionary<string, string>();
            var queue = new Queue<string>();
            prev[end] = null;
            queue.Enqueue(end);
            string found = null;
            while (queue.Count > 0 && found == null)
            {
                string n = queue.Dequeue();
                foreach (string s in network.Successors(n))
                {
                    if (prev.ContainsKey(s)) continue;
                    prev[s] = n;
                    if (IsTerminalNode(network, s)) { found = s; break; }
                    queue.Enqueue(s);
                }
            }
            if (found == null) return;

            var tail = new List<string>();
            for (string cur = found; cur != end; cur = prev[cur])
                tail.Add(cur);
            tail.Reverse();
            path.AddRange(tail);
        }

        private static bool IsTerminalNode(Network network, string nodeId)
        {
            Component c;
            return network.Components.TryGetValue(nodeId, out c) && c is Terminal;
        }

        public string ToText()
        {
            string[] headerLabels = { "Component", "Kind", "Port", "Q [m³/s]", "V [m/s]", "ΔP [Pa]", "Σ ΔP [Pa]", "Share [%]" };
            int[] headerWidths = { 12, 14, 10, 10, 8, 9, 10, 9 };

            var headerParts = new string[headerLabels.Length];
            for (int i = 0; i < headerLabels.Length; i++)
                headerParts[i] = headerLabels[i].PadRight(headerWidths[i]);
            string header = string.Join(" | ", headerParts);
            string sepLine = new string('-', header.Length);

            var lines = new List<string>();
            if (!string.IsNullOrEmpty(NetworkName))
                lines.Add("network: " + NetworkName);
            lines.Add(sepLine);
            lines.Add(header);
            lines.Add(sepLine);
            foreach (PressureReportRow r in Rows)
            {
                string q = r.FlowM3s.HasValue
                    ? r.FlowM3s.Value.ToString("0.000", CultureInfo.InvariantCulture)
                    : "—";
                string v = r.VelocityMs.HasValue
                    ? r.VelocityMs.Value.ToString("0.00", CultureInfo.InvariantCulture)
                    : "—";
                lines.Add(string.Format(CultureInfo.InvariantCulture,
                    "{0,-12} | {1,-14} | {2,-10} | {3,10} | {4,8} | {5,9} | {6,10} | {7,9}",
                    r.ComponentId, r.Kind, r.Port, q, v,
                    r.DpPa.ToString("0.00", CultureInfo.InvariantCulture),
                    r.CumulativeDpPa.ToString("0.00", CultureInfo.InvariantCulture),
                    r.SharePercent.ToString("0.0", CultureInfo.InvariantCulture)));
            }
            lines.Add(sepLine);
            lines.Add("critical path ΔP = "
                + CriticalDpPa.ToString("0.00", CultureInfo.InvariantCulture) + " Pa");
            return string.Join("\n", lines.ToArray());
        }

        public string ToCsv(char delimiter = ',')
        {
            string sep = delimiter.ToString();
            var lines = new List<string> { string.Join(sep, Fields) };
            foreach (PressureReportRow r in Rows)
            {
                lines.Add(string.Join(sep, new[]
                {
                    r.ComponentId,
                    r.Kind,
                    r.Port,
                    Results.FmtOpt(r.FlowM3s),
                    Results.FmtOpt(r.VelocityMs),
                    r.DpPa.ToString(CultureInfo.InvariantCulture),
                    r.CumulativeDpPa.ToString(CultureInfo.InvariantCulture),
                    r.SharePercent.ToString(CultureInfo.InvariantCulture),
                }));
            }
            return string.Join("\n", lines.ToArray());
        }
    }
}
