using System;
using System.Collections.Generic;
using System.Globalization;

namespace Wenta
{
    public sealed class ComponentResult
    {
        public string ComponentId;
        public string Name;
        public string ComponentType;
        public double? FlowrateIn;
        public double? FlowrateOut;
        public double? VelocityIn;
        public double? VelocityOut;
        public double PressureDrop;

        public static readonly string[] Fields =
        {
            "component_id",
            "name",
            "component_type",
            "flowrate_in",
            "flowrate_out",
            "velocity_in",
            "velocity_out",
            "pressure_drop",
        };

        public List<KeyValuePair<string, string>> ToDict()
        {
            var d = new List<KeyValuePair<string, string>>();
            d.Add(new KeyValuePair<string, string>("component_id", ComponentId));
            d.Add(new KeyValuePair<string, string>("name", Name));
            d.Add(new KeyValuePair<string, string>("component_type", ComponentType));
            d.Add(new KeyValuePair<string, string>("flowrate_in", Results.FmtOpt(FlowrateIn)));
            d.Add(new KeyValuePair<string, string>("flowrate_out", Results.FmtOpt(FlowrateOut)));
            d.Add(new KeyValuePair<string, string>("velocity_in", Results.FmtOpt(VelocityIn)));
            d.Add(new KeyValuePair<string, string>("velocity_out", Results.FmtOpt(VelocityOut)));
            d.Add(new KeyValuePair<string, string>(
                "pressure_drop", PressureDrop.ToString(CultureInfo.InvariantCulture)));
            return d;
        }
    }

    public static class Results
    {
        public static List<ComponentResult> ExtractResults(Network network)
        {
            var results = new List<ComponentResult>();
            foreach (var kv in network.Components)
            {
                string cid = kv.Key;
                Component c = kv.Value;

                double? flowrateIn = FirstOpt(c.Inlets, p => p.Flowrate);
                double? flowrateOut = FirstOpt(c.Outlets, p => p.Flowrate);
                double? velocityIn = FirstOpt(c.Inlets, p => p.Flowrate.HasValue ? p.Velocity : null);
                double? velocityOut = FirstOpt(c.Outlets, p => p.Flowrate.HasValue ? p.Velocity : null);

                results.Add(new ComponentResult
                {
                    ComponentId = cid,
                    Name = c.Name,
                    ComponentType = c.GetType().Name,
                    FlowrateIn = flowrateIn,
                    FlowrateOut = flowrateOut,
                    VelocityIn = velocityIn,
                    VelocityOut = velocityOut,
                    PressureDrop = c.TotalPressureDrop(),
                });
            }
            return results;
        }

        private static double? FirstOpt(IEnumerable<Port> ports, Func<Port, double?> f)
        {
            foreach (Port p in ports)
            {
                double? v = f(p);
                if (v.HasValue) return v;
            }
            return null;
        }

        internal static string FmtOpt(double? v)
        {
            return v.HasValue ? v.Value.ToString(CultureInfo.InvariantCulture) : "";
        }

        public static string ResultsAsCsv(Network network, char delimiter)
        {
            List<ComponentResult> results = ExtractResults(network);
            if (results.Count == 0) return "";

            string sep = delimiter.ToString();
            var lines = new List<string> { string.Join(sep, ComponentResult.Fields) };
            foreach (ComponentResult r in results)
            {
                var values = new List<string>();
                foreach (KeyValuePair<string, string> kv in r.ToDict()) values.Add(kv.Value);
                lines.Add(string.Join(sep, values.ToArray()));
            }
            return string.Join("\n", lines.ToArray());
        }

        public static string ResultsSummary(Network network)
        {
            List<ComponentResult> results = ExtractResults(network);
            if (results.Count == 0) return "(no components)";

            string[] headerLabels = { "ID", "Name", "Type", "Q_in [m³/s]", "V_in [m/s]", "ΔP [Pa]" };
            int[] headerWidths = { 12, 20, 18, 13, 12, 11 };

            var headerParts = new string[headerLabels.Length];
            for (int i = 0; i < headerLabels.Length; i++)
                headerParts[i] = headerLabels[i].PadRight(headerWidths[i]);
            string header = string.Join(" | ", headerParts);
            string sepLine = new string('-', header.Length);

            var lines = new List<string> { sepLine, header, sepLine };
            foreach (ComponentResult r in results)
            {
                string qIn = r.FlowrateIn.HasValue
                    ? r.FlowrateIn.Value.ToString("0.000", CultureInfo.InvariantCulture)
                    : "—";
                string vIn = r.VelocityIn.HasValue
                    ? r.VelocityIn.Value.ToString("0.00", CultureInfo.InvariantCulture)
                    : "—";
                string dp = r.PressureDrop.ToString("0.00", CultureInfo.InvariantCulture);
                lines.Add(string.Format(CultureInfo.InvariantCulture,
                    "{0,-12} | {1,-20} | {2,-18} | {3,12} | {4,11} | {5,10}",
                    r.ComponentId, r.Name, r.ComponentType, qIn, vIn, dp));
            }
            lines.Add(sepLine);
            return string.Join("\n", lines.ToArray());
        }
    }
}
