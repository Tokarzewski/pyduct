using System;

namespace Wenta
{
    public enum UnitSystem
    {
        Si,

        Ip
    }

    public class ProjectSettings
    {
        public Standard Standard;

        public double DefaultDiameterMm;

        public double AbsoluteRoughnessM;

        public double TargetVelocityMs;

        public double TargetPaPerM;

        public string NoiseSpace;

        public UnitSystem UnitSystem;

        public ProjectSettings()
        {
            Standard = Standard.En1505_1506;
            DefaultDiameterMm = 200.0;
            AbsoluteRoughnessM = 0.0001;
            TargetVelocityMs = 4.0;
            TargetPaPerM = 1.0;
            NoiseSpace = "office";
            UnitSystem = UnitSystem.Si;
        }

        public void Validate()
        {
            if (DefaultDiameterMm <= 0.0)
                throw new WentaException("default_diameter_mm must be > 0");
            if (AbsoluteRoughnessM <= 0.0)
                throw new WentaException("absolute_roughness_m must be > 0");
            if (TargetVelocityMs <= 0.0)
                throw new WentaException("target_velocity_ms must be > 0");
            if (TargetPaPerM <= 0.0)
                throw new WentaException("target_pa_per_m must be > 0");
        }
    }
}
