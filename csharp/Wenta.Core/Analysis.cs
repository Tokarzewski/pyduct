using System;
using System.Collections.Generic;

namespace Wenta
{
    public sealed class BranchInfo
    {
        public string ComponentId;
        public string Kind;
        public double FlowM3s;
        public double VelocityMs;
        public double PressureDropPa;
        public double? RegeneratedNoiseDb;
        public double? BalancingZeta;
    }

    public sealed class AnalysisSummary
    {
        public double CriticalDpPa;
        public List<BranchInfo> Branches = new List<BranchInfo>();
        public int NBranches { get { return Branches.Count; } }
    }

    public static class Analysis
    {
        public static AnalysisSummary Analyze(Network network, Fluid fluid)
        {
            double criticalDpPa = network.Solve(fluid);

            var branches = new List<BranchInfo>();

            foreach (var kv in network.Components)
            {
                string cid = kv.Key;
                Component c = kv.Value;

                double area, diameter;
                string kind;

                RigidDuct rigid = c as RigidDuct;
                FlexDuct flex = c as FlexDuct;
                if (rigid != null)
                {
                    area = rigid.CrossSection.Area;
                    diameter = rigid.CrossSection.HydraulicDiameter;
                    kind = "RigidDuct";
                }
                else if (flex != null)
                {
                    area = flex.Area;
                    diameter = flex.Diameter;
                    kind = "FlexDuct";
                }
                else
                {
                    continue;
                }

                double flowM3s = c.InletFlowrate() ?? 0.0;

                double velocityMs = flowM3s / area;

                double pressureDropPa = c.TotalPressureDrop();

                double? regeneratedNoiseDb =
                    !(velocityMs <= 0.0) && !(diameter <= 0.0) && !(fluid.Density <= 0.0)
                        ? (double?)Sound.RegeneratedNoiseRound(velocityMs, diameter, fluid.Density)
                        : null;

                double? balancingZeta = velocityMs > 0.0
                    ? (double?)Balancing.BalancingZeta(criticalDpPa, pressureDropPa, velocityMs, fluid.Density)
                    : null;

                branches.Add(new BranchInfo
                {
                    ComponentId = cid,
                    Kind = kind,
                    FlowM3s = flowM3s,
                    VelocityMs = velocityMs,
                    PressureDropPa = pressureDropPa,
                    RegeneratedNoiseDb = regeneratedNoiseDb,
                    BalancingZeta = balancingZeta,
                });
            }

            return new AnalysisSummary
            {
                CriticalDpPa = criticalDpPa,
                Branches = branches,
            };
        }
    }
}
