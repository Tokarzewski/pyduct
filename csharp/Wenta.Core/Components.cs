using System;
using System.Collections.Generic;

namespace Wenta
{
    public sealed class Port
    {
        public readonly string Name;
        public readonly bool IsIn;
        public double? Flowrate;
        public double? Velocity;
        public double PressureDrop;
        public string NodeId;

        public Port(string name, bool isIn)
        {
            Name = name;
            IsIn = isIn;
        }
    }

    public abstract class Component
    {
        public readonly string Name;
        public List<Port> Ports = new List<Port>();

        protected Component(string name) { Name = name; }

        public abstract void Compute(Fluid fluid);

        public Port Port_(string name)
        {
            foreach (Port p in Ports)
                if (p.Name == name) return p;
            throw new WentaException(GetType().Name + " '" + Name
                + "' has no port named '" + name + "'");
        }

        public IEnumerable<Port> Inlets
        {
            get { foreach (Port p in Ports) if (p.IsIn) yield return p; }
        }
        public IEnumerable<Port> Outlets
        {
            get { foreach (Port p in Ports) if (!p.IsIn) yield return p; }
        }

        public double? InletFlowrate()
        {
            foreach (Port p in Inlets)
                return p.Flowrate;
            return null;
        }

        public double TotalPressureDrop()
        {
            double total = 0.0;
            foreach (Port p in Ports) total += p.PressureDrop;
            return total;
        }
    }

    public sealed class RigidDuct : Component
    {
        public const double DefaultAbsoluteRoughness = 0.0001;

        public readonly CrossSection CrossSection;
        public readonly double Length;
        public readonly double AbsoluteRoughness;

        public RigidDuct(string name, CrossSection crossSection, double length,
                         double absoluteRoughness = DefaultAbsoluteRoughness)
            : base(name)
        {
            if (length <= 0.0)
                throw new WentaException("length must be positive, got " + length);
            CrossSection = crossSection;
            Length = length;
            AbsoluteRoughness = absoluteRoughness;
            Ports.Add(new Port("inlet", true));
            Ports.Add(new Port("outlet", false));
        }

        public override void Compute(Fluid fluid)
        {
            Port inlet = Ports[0], outlet = Ports[1];
            if (inlet.Flowrate == null)
                throw new WentaException("RigidDuct '" + Name + "': inlet flowrate not set");
            double v = inlet.Flowrate.Value / CrossSection.Area;
            double re = Friction.Reynolds(v, CrossSection.HydraulicDiameter,
                                          fluid.KinematicViscosity);
            double f = Friction.FrictionFactor(re,
                Friction.RelativeRoughness(AbsoluteRoughness, CrossSection.HydraulicDiameter));
            inlet.Velocity = v;
            outlet.Velocity = v;
            outlet.Flowrate = inlet.Flowrate;
            inlet.PressureDrop = Losses.StraightPressureDrop(
                f, Length, CrossSection.HydraulicDiameter, v, fluid.Density);
            outlet.PressureDrop = 0.0;
        }
    }

    public sealed class FlexDuct : Component
    {
        public readonly double Diameter;
        public readonly double Length;
        public readonly double PressureDropPerMeter;
        public readonly double StretchPercentage;
        public readonly double Area;

        public FlexDuct(string name, double diameter, double length,
                        double pressureDropPerMeter, double stretchPercentage = 100.0)
            : base(name)
        {
            if (diameter <= 0.0 || length <= 0.0)
                throw new WentaException("diameter and length must be positive");
            if (stretchPercentage <= 0.0 || stretchPercentage > 100.0)
                throw new WentaException("stretch_percentage must be in (0, 100], got "
                    + stretchPercentage);
            Diameter = diameter;
            Length = length;
            PressureDropPerMeter = pressureDropPerMeter;
            StretchPercentage = stretchPercentage;
            Area = Math.PI * (Diameter / 2.0) * (Diameter / 2.0);
            Ports.Add(new Port("inlet", true));
            Ports.Add(new Port("outlet", false));
        }

        public override void Compute(Fluid fluid)
        {
            Port inlet = Ports[0], outlet = Ports[1];
            if (inlet.Flowrate == null)
                throw new WentaException("FlexDuct '" + Name + "': inlet flowrate not set");
            double v = inlet.Flowrate.Value / Area;
            double beta = Flex.StretchCorrectionFactor(Diameter, StretchPercentage);
            inlet.Velocity = v;
            outlet.Velocity = v;
            outlet.Flowrate = inlet.Flowrate;
            inlet.PressureDrop = PressureDropPerMeter * Length * beta;
            outlet.PressureDrop = 0.0;
        }
    }

    public sealed class Source : Component
    {
        public Source(string name) : base(name)
        {
            Ports.Add(new Port("outlet", false));
        }

        public override void Compute(Fluid fluid)
        {
            Ports[0].Velocity = 0.0;
            Ports[0].PressureDrop = 0.0;
        }
    }

    public sealed class Terminal : Component
    {
        public readonly double Flowrate;
        public readonly CrossSection CrossSection;
        public readonly double Zeta;

        public Terminal(string name, double flowrate,
                        CrossSection crossSection = null, double zeta = 0.0)
            : base(name)
        {
            if (flowrate < 0.0)
                throw new WentaException("flowrate must be >= 0, got " + flowrate);
            Flowrate = flowrate;
            CrossSection = crossSection;
            Zeta = zeta;
            Ports.Add(new Port("inlet", true) { Flowrate = flowrate });
        }

        public override void Compute(Fluid fluid)
        {
            Port port = Ports[0];
            if (port.Flowrate == null || port.Flowrate == 0.0 || CrossSection == null)
            {
                port.Velocity = 0.0;
                port.PressureDrop = 0.0;
                return;
            }
            double v = port.Flowrate.Value / CrossSection.Area;
            port.Velocity = v;
            port.PressureDrop = Losses.LocalPressureDrop(Zeta, v, fluid.Density);
        }
    }

    public sealed class TwoPortFitting : Component
    {
        public readonly CrossSection CrossSection;
        public readonly double Zeta;

        public TwoPortFitting(string name, CrossSection crossSection, double zeta)
            : base(name)
        {
            CrossSection = crossSection;
            Zeta = zeta;
            Ports.Add(new Port("inlet", true));
            Ports.Add(new Port("outlet", false));
        }

        public override void Compute(Fluid fluid)
        {
            Port inlet = Ports[0], outlet = Ports[1];
            if (inlet.Flowrate == null)
                throw new WentaException("TwoPortFitting '" + Name + "': inlet flowrate not set");
            double v = inlet.Flowrate.Value / CrossSection.Area;
            inlet.Velocity = v;
            outlet.Velocity = v;
            outlet.Flowrate = inlet.Flowrate;
            inlet.PressureDrop = 0.0;
            outlet.PressureDrop = Losses.LocalPressureDrop(Zeta, v, fluid.Density);
        }
    }

    public sealed class Tee : Component
    {
        public readonly CrossSection CrossSection;
        public readonly double ZetaStraight;
        public readonly double ZetaBranch;

        public Tee(string name, CrossSection crossSection,
                   double zetaStraight = 0.0, double zetaBranch = 0.5)
            : base(name)
        {
            CrossSection = crossSection;
            ZetaStraight = zetaStraight;
            ZetaBranch = zetaBranch;
            Ports.Add(new Port("combined", true));
            Ports.Add(new Port("straight", false));
            Ports.Add(new Port("branch", false));
        }

        public override void Compute(Fluid fluid)
        {
            Port combined = Ports[0], straight = Ports[1], branch = Ports[2];
            if (straight.Flowrate == null || branch.Flowrate == null)
                throw new WentaException("Tee '" + Name + "': leg flowrates not set");
            double invA = 1.0 / CrossSection.Area;
            double rho = fluid.Density;
            double vs = straight.Flowrate.Value * invA;
            double vb = branch.Flowrate.Value * invA;
            combined.Flowrate = straight.Flowrate + branch.Flowrate;
            combined.Velocity = combined.Flowrate.Value * invA;
            combined.PressureDrop = 0.0;
            straight.Velocity = vs;
            branch.Velocity = vb;
            straight.PressureDrop = Losses.LocalPressureDrop(ZetaStraight, vs, rho);
            branch.PressureDrop = Losses.LocalPressureDrop(ZetaBranch, vb, rho);
        }
    }
}
