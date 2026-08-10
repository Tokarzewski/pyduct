using System;
using System.Runtime.InteropServices;
using System.Text;

namespace Venti.Plugin
{
    /// <summary>
    /// P/Invoke backend: calls straight into the native venti C-ABI cdylib
    /// (libventi.so / venti.dll / libventi.dylib), built from the Rust crate.
    ///
    /// The C ABI is the same one the WASM core exposes, so signatures here map
    /// 1:1 to src/ffi.rs. Scalar functions return f64; sizing uses out-params
    /// plus an i32 status (0 = ok).
    /// </summary>
    public sealed class NativeCore : IVentiCore
    {
        private const string Library = "venti"; // venti.dll / libventi.so / libventi.dylib
        private readonly bool _disposed;

        // scalar f64 returns
        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)]
        private static extern double venti_friction_factor(double re, double relRoughness);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)]
        private static extern double venti_local_pressure_drop(double zeta, double v, double rho);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)]
        private static extern double venti_straight_pressure_drop(
            double f, double len, double dHyd, double v, double rho);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)]
        private static extern double venti_regenerated_noise_round(double v, double d, double rho);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)]
        private static extern double venti_duct_pressure_level(double lw, double area, double alpha);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)]
        private static extern int venti_nc_ok(IntPtr space, long spaceLen, double levelDb, out int outOk);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)]
        private static extern double venti_required_zeta(double dpPa, double v, double rho);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)]
        private static extern double venti_damper_open_percentage(double zeta);

        // sizing: status + out-params
        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)]
        private static extern int venti_velocity_method_round(
            double flowrate, double targetVelocity, out double oDiamM, out double oVelocity);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)]
        private static extern int venti_equal_friction_method_round(
            double flowrate, double targetPaPerM, double roughness, double density,
            double viscosity, out double oDiamM, out double oVelocity, out double oDpPerM);

        public SectionResult VelocityMethodRound(double flowrate, double targetVelocity)
        {
            if (venti_velocity_method_round(flowrate, targetVelocity,
                    out double dM, out double v) != 0)
                throw new InvalidOperationException("venti velocity sizing failed");
            return new SectionResult(dM, dM, v);
        }

        public SectionResult EqualFrictionMethodRound(double flowrate, double targetPaPerM,
            double roughness, double density, double viscosity)
        {
            if (venti_equal_friction_method_round(flowrate, targetPaPerM, roughness,
                    density, viscosity, out double dM, out double v, out _) != 0)
                throw new InvalidOperationException("venti equal-friction sizing failed");
            return new SectionResult(dM, dM, v);
        }

        public double FrictionFactor(double re, double relRoughness) =>
            venti_friction_factor(re, relRoughness);

        public double LocalPressureDrop(double zeta, double velocity, double density) =>
            venti_local_pressure_drop(zeta, velocity, density);

        public double StraightPressureDrop(double f, double length, double dHyd,
            double velocity, double density) =>
            venti_straight_pressure_drop(f, length, dHyd, velocity, density);

        public double RegeneratedNoiseRound(double v, double d, double rho) =>
            venti_regenerated_noise_round(v, d, rho);

        public double DuctPressureLevel(double lw, double area, double alpha) =>
            venti_duct_pressure_level(lw, area, alpha);

        public bool NcOk(string spaceType, double levelDb)
        {
            byte[] bytes = Encoding.UTF8.GetBytes(spaceType);
            IntPtr ptr = Marshal.AllocHGlobal(bytes.Length);
            try
            {
                Marshal.Copy(bytes, 0, ptr, bytes.Length);
                if (venti_nc_ok(ptr, bytes.Length, levelDb, out int ok) != 0)
                    return false;
                return ok != 0;
            }
            finally
            {
                Marshal.FreeHGlobal(ptr);
            }
        }

        public double RequiredZeta(double dpPa, double velocity, double density) =>
            venti_required_zeta(dpPa, velocity, density);

        public double DamperOpenPercentage(double zeta) =>
            venti_damper_open_percentage(zeta);

        public void Dispose()
        {
            // No unmanaged buffers owned here; reserved for symmetry.
        }
    }
}
