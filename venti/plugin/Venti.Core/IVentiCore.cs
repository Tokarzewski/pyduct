using System;

namespace Venti.Core
{
    /// <summary>
    /// A sizing result: a round/rectangular section in metres plus the actual
    /// velocity, mirroring the C-ABI out-params of the venti WASM core.
    /// </summary>
    public readonly struct SectionResult
    {
        public SectionResult(double widthM, double heightM, double velocityMs)
        {
            WidthM = widthM;
            HeightM = heightM;
            VelocityMs = velocityMs;
        }
        public double WidthM { get; }
        public double HeightM { get; }
        public double VelocityMs { get; }
        /// <summary>Round ducts have WidthM == HeightM == diameter.</summary>
        public double DiameterM => WidthM;
        public override string ToString() =>
            HeightM > 0 && Math.Abs(WidthM - HeightM) > 1e-12
                ? $"{WidthM:F3} x {HeightM:F3} m @ {VelocityMs:F2} m/s"
                : $"{DiameterM:F3} m @ {VelocityMs:F2} m/s";
    }

    /// <summary>
    /// Host-agnostic facade over the venti computational core (issue #14).
    ///
    /// The ZWCAD/AutoCAD plugin talks only to this interface; the math lives in
    /// the Rust <c>venti</c> library reached either by embedding the WASM core
    /// (<see cref="WasmCore"/>) or P/Invoking the native <c>cdylib</c>
    /// (<see cref="NativeCore"/>). Backends are swappable at runtime via
    /// <see cref="VentiCoreFactory.Create"/>.
    /// </summary>
    public interface IVentiCore : IDisposable
    {
        // ---- sizing (venti::sizing) ----
        SectionResult VelocityMethodRound(double flowrate, double targetVelocity);
        SectionResult EqualFrictionMethodRound(double flowrate, double targetPaPerM,
            double absoluteRoughness, double density, double dynamicViscosity);

        // ---- friction / hydraulics (venti::physics) ----
        double FrictionFactor(double reynolds, double relRoughness);
        double Reynolds(double velocity, double hydraulicDiameter, double kinematicViscosity);
        double RelativeRoughness(double absoluteRoughness, double hydraulicDiameter);
        double LocalPressureDrop(double zeta, double velocity, double density);
        double StraightPressureDrop(double f, double length, double dHyd, double velocity, double density);

        // ---- sound (venti::sound) ----
        double RegeneratedNoiseRound(double velocity, double diameter, double density);
        double DuctPressureLevel(double soundPowerDb, double roomAreaM2, double absorption);
        bool NcOk(string spaceType, double levelDb);

        // ---- balancing (venti::balancing) ----
        double RequiredZeta(double requiredDpPa, double velocity, double density);
        double DamperOpenPercentage(double zeta);
    }
}
