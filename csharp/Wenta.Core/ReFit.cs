using System;
using System.Collections.Generic;
using System.Globalization;

namespace Wenta
{
    public sealed class ReFitOptions
    {
        public SizingMethod Method = SizingMethod.Velocity;
        public double TargetVelocity = 4.0;
        public double TargetPaPerM = 1.0;
        public Standard Standard = Standard.En1505_1506;
        public bool SnapToStandard = true;
        public Fluid Fluid;
    }

    public sealed class ReFitChange
    {
        public string ComponentId;
        public double OldDiameterM;
        public double NewDiameterM;
        public double OldVelocityMs;
        public double NewVelocityMs;
        public bool Changed;
    }

    public sealed class ReFitResult
    {
        public Network Network;
        public List<ReFitChange> Changes = new List<ReFitChange>();
        public double OldCriticalDpPa;
        public double NewCriticalDpPa;

        public static readonly string[] Fields =
        {
            "component_id",
            "old_diameter_m",
            "new_diameter_m",
            "old_velocity_ms",
            "new_velocity_ms",
            "changed",
        };

        public string ToCsv()
        {
            var lines = new List<string> { string.Join(",", Fields) };
            if (Changes == null) return lines[0];
            foreach (ReFitChange c in Changes)
            {
                lines.Add(string.Join(",", new[]
                {
                    ReFit.Quote(c.ComponentId),
                    c.OldDiameterM.ToString(CultureInfo.InvariantCulture),
                    c.NewDiameterM.ToString(CultureInfo.InvariantCulture),
                    c.OldVelocityMs.ToString(CultureInfo.InvariantCulture),
                    c.NewVelocityMs.ToString(CultureInfo.InvariantCulture),
                    c.Changed ? "true" : "false",
                }));
            }
            return string.Join("\n", lines.ToArray());
        }
    }

    public sealed class BalancingHint
    {
        public string TerminalId;
        public double AvailableDpPa;
        public double RequiredDpPa;
        public double SurplusDpPa;
        public double DamperZeta;
        public double DamperOpenPercent;
    }

    public static class ReFit
    {

        public static ReFitResult Apply(Network network, ReFitOptions options = null)
        {
            if (network == null)
                throw new WentaException("ReFit.Apply: network is null");
            options = options ?? new ReFitOptions();
            Fluid fluid = options.Fluid ?? Fluid.StandardAir();

            if (network.Terminals().Count == 0)
                throw new WentaException(
                    "ReFit.Apply: network has no Terminal component, so no duct carries a flow to size from");

            double oldCritical = network.Solve(fluid);

            var result = new ReFitResult { OldCriticalDpPa = oldCritical };
            var rebuilt = new Network { Name = network.Name };

            foreach (KeyValuePair<string, Component> kv in network.Components)
            {
                string cid = kv.Key;
                Component c = kv.Value;

                RigidDuct rigid = c as RigidDuct;
                FlexDuct flex = c as FlexDuct;
                if (rigid == null && flex == null)
                {
                    rebuilt.Add(cid, CopyNonDuct(cid, c));
                    continue;
                }

                double oldArea = rigid != null ? rigid.CrossSection.Area : flex.Area;
                double oldDiameter = rigid != null
                    ? rigid.CrossSection.HydraulicDiameter : flex.Diameter;
                double flow = c.InletFlowrate() ?? 0.0;
                if (flow <= 0.0)
                    throw new WentaException("ReFit.Apply: duct '" + cid
                        + "' carries no flow (" + flow.ToString(CultureInfo.InvariantCulture)
                        + " m^3/s); it cannot be sized");

                Component resized;
                double newArea, newDiameter;
                if (rigid != null)
                {
                    CrossSection section = SizeSection(cid, rigid.CrossSection, flow,
                        rigid.Length, rigid.AbsoluteRoughness, options, fluid);
                    resized = new RigidDuct(rigid.Name, section, rigid.Length,
                        rigid.AbsoluteRoughness);
                    newArea = section.Area;
                    newDiameter = section.HydraulicDiameter;
                }
                else
                {
                    CrossSection section = SizeSection(cid, new Round(flex.Diameter), flow,
                        flex.Length, RigidDuct.DefaultAbsoluteRoughness, options, fluid);
                    double d = ((Round)section).Diameter;
                    resized = new FlexDuct(flex.Name, d, flex.Length,
                        flex.PressureDropPerMeter, flex.StretchPercentage);
                    newArea = section.Area;
                    newDiameter = d;
                }
                rebuilt.Add(cid, resized);

                result.Changes.Add(new ReFitChange
                {
                    ComponentId = cid,
                    OldDiameterM = oldDiameter,
                    NewDiameterM = newDiameter,
                    OldVelocityMs = flow / oldArea,
                    NewVelocityMs = flow / newArea,
                    Changed = Math.Abs(newDiameter - oldDiameter) > SizeEpsilonM,
                });
            }

            CopyConnections(network, rebuilt);
            result.Network = rebuilt;
            result.NewCriticalDpPa = rebuilt.Solve(fluid);
            return result;
        }

        private const double SizeEpsilonM = 1e-9;

        private static CrossSection SizeSection(string cid, CrossSection old, double flow,
            double length, double roughness, ReFitOptions options, Fluid fluid)
        {
            Round oldRound = old as Round;
            string shape = oldRound != null ? Sizing.ShapeRound : Sizing.ShapeRectangular;

            Sizing.SizingResult sized;
            switch (options.Method)
            {
                case SizingMethod.Velocity:
                    sized = Sizing.VelocityMethod(flow, shape, options.TargetVelocity);
                    break;
                case SizingMethod.EqualFriction:
                    sized = Sizing.EqualFrictionMethod(flow, options.TargetPaPerM, shape,
                        roughness, fluid);
                    break;
                case SizingMethod.PressureDropBudget:
                    sized = Sizing.PressureDropBudget(flow, length,
                        options.TargetPaPerM * length, shape, roughness, fluid);
                    break;
                case SizingMethod.NoiseLimit:
                    throw new WentaException(
                        "ReFit.Apply: SizingMethod.NoiseLimit needs a space type, which"
                        + " ReFitOptions does not carry; use BatchSizing, or"
                        + " SizingMethod.Velocity with TargetVelocity set to the NC limit");
                case SizingMethod.AspectRatio:
                    throw new WentaException(
                        "ReFit.Apply: SizingMethod.AspectRatio is not supported — it needs an"
                        + " aspect ratio and would force every duct rectangular; use BatchSizing");
                default:
                    throw new WentaException("ReFit.Apply: unknown sizing method "
                        + options.Method);
            }

            if (!options.SnapToStandard) return sized.Section;

            Round round = sized.Section as Round;
            if (round != null)
            {
                int mm = Standards.NearestRoundSizeFor(options.Standard, ToMm(round.Diameter), true);
                return new Round(mm / 1000.0);
            }
            Rectangular rect = (Rectangular)sized.Section;
            int[] wh = BatchSizing.SnapRectangular(options.Standard, ToMm(rect.Width),
                ToMm(rect.Height));
            if (wh == null)
                throw new WentaException("ReFit.Apply: no size in the chosen standard is large"
                    + " enough for duct '" + cid + "' (" + rect.Describe() + ")");
            return new Rectangular(wh[0] / 1000.0, wh[1] / 1000.0);
        }

        private static double ToMm(double metres)
        {
            return Math.Round(metres * 1000.0, 3);
        }

        private static Component CopyNonDuct(string cid, Component c)
        {
            Source source = c as Source;
            if (source != null) return new Source(source.Name);

            Terminal terminal = c as Terminal;
            if (terminal != null)
                return new Terminal(terminal.Name, terminal.Flowrate, terminal.CrossSection,
                    terminal.Zeta);

            Tee tee = c as Tee;
            if (tee != null)
                return new Tee(tee.Name, tee.CrossSection, tee.ZetaStraight, tee.ZetaBranch);

            TwoPortFitting fitting = c as TwoPortFitting;
            if (fitting != null)
                return new TwoPortFitting(fitting.Name, fitting.CrossSection, fitting.Zeta);

            throw new WentaException("ReFit.Apply: cannot copy component '" + cid
                + "' of unsupported type " + c.GetType().Name);
        }

        private static void CopyConnections(Network from, Network to)
        {
            foreach (KeyValuePair<string, Component> kv in from.Components)
            {
                foreach (Port p in kv.Value.Ports)
                {
                    if (!p.IsIn) continue;
                    foreach (string pred in from.Predecessors(Network.PortNodeId(kv.Key, p.Name)))
                    {
                        int colon = pred.LastIndexOf(':');
                        if (colon < 0) continue;
                        to.Connect(pred.Substring(0, colon) + "." + pred.Substring(colon + 1),
                            kv.Key + "." + p.Name);
                    }
                }
            }
        }


        public static List<BalancingHint> Hints(Network solvedNetwork, Fluid fluid = null)
        {
            if (solvedNetwork == null)
                throw new WentaException("ReFit.Hints: network is null");
            fluid = fluid ?? Fluid.StandardAir();
            if (solvedNetwork.Terminals().Count == 0)
                throw new WentaException("ReFit.Hints: network has no Terminal component");

            List<string> topo = solvedNetwork.TopoOrder();
            var dist = new Dictionary<string, double>(topo.Count);
            var prev = new Dictionary<string, string>(topo.Count);
            foreach (string n in topo)
            {
                List<string> preds = solvedNetwork.Predecessors(n);
                string bestP = null;
                double bestD = 0.0;
                foreach (string p in preds)
                {
                    double d = dist[p];
                    if (bestP == null || d > bestD) { bestP = p; bestD = d; }
                }
                prev[n] = bestP;
                dist[n] = bestD + solvedNetwork.NodeDp(n);
            }

            var portOf = new Dictionary<string, Port>();
            foreach (KeyValuePair<string, Component> kv in solvedNetwork.Components)
                foreach (Port p in kv.Value.Ports)
                    portOf[Network.PortNodeId(kv.Key, p.Name)] = p;

            var hints = new List<BalancingHint>();
            var nodes = new List<string>();
            double required = 0.0;
            foreach (KeyValuePair<string, Component> kv in solvedNetwork.Components)
            {
                if (!(kv.Value is Terminal)) continue;
                string node = Network.PortNodeId(kv.Key, kv.Value.Ports[0].Name);
                nodes.Add(node);
                double branch = dist[node];
                if (branch > required) required = branch;
                hints.Add(new BalancingHint
                {
                    TerminalId = kv.Key,
                    AvailableDpPa = branch,
                });
            }

            for (int i = 0; i < hints.Count; i++)
            {
                BalancingHint h = hints[i];
                h.RequiredDpPa = required;
                double surplus = required - h.AvailableDpPa;
                h.SurplusDpPa = surplus > 0.0 ? surplus : 0.0;
                double velocity = DamperVelocity(nodes[i], portOf, prev);
                h.DamperZeta = Balancing.BalancingZeta(required, h.AvailableDpPa, velocity,
                    fluid.Density);
                h.DamperOpenPercent = Balancing.DamperOpenPercentage(h.DamperZeta);
            }
            return hints;
        }

        private static double DamperVelocity(string terminalNode,
            IDictionary<string, Port> portOf, IDictionary<string, string> prev)
        {
            string node = terminalNode;
            while (node != null)
            {
                Port p;
                if (portOf.TryGetValue(node, out p) && p.Velocity.HasValue
                    && p.Velocity.Value > 0.0)
                    return p.Velocity.Value;
                node = prev[node];
            }
            return 0.0;
        }

        public static readonly string[] HintFields =
        {
            "terminal_id",
            "available_dp_pa",
            "required_dp_pa",
            "surplus_dp_pa",
            "damper_zeta",
            "damper_open_percent",
        };

        public static string HintsAsCsv(IList<BalancingHint> hints)
        {
            var lines = new List<string> { string.Join(",", HintFields) };
            if (hints == null) return lines[0];
            foreach (BalancingHint h in hints)
            {
                lines.Add(string.Join(",", new[]
                {
                    Quote(h.TerminalId),
                    h.AvailableDpPa.ToString(CultureInfo.InvariantCulture),
                    h.RequiredDpPa.ToString(CultureInfo.InvariantCulture),
                    h.SurplusDpPa.ToString(CultureInfo.InvariantCulture),
                    h.DamperZeta.ToString(CultureInfo.InvariantCulture),
                    h.DamperOpenPercent.ToString(CultureInfo.InvariantCulture),
                }));
            }
            return string.Join("\n", lines.ToArray());
        }

        internal static string Quote(string s)
        {
            if (s == null) return "";
            if (s.IndexOfAny(new[] { ',', '"', '\n', '\r' }) < 0) return s;
            return "\"" + s.Replace("\"", "\"\"") + "\"";
        }
    }
}
