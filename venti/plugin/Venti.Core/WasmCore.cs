using System;

namespace Venti.Core
{
    /// <summary>
    /// WASM backend: embeds venti.wasm via the Wasmtime .NET SDK (NuGet
    /// "Wasmtime"), the cross-platform single-artifact option. This is the
    /// default for the "wasm" backend and needs no native .so/.dll.
    ///
    /// NOTE: exact Wasmtime C# API surface depends on the installed package
    /// version; adjust the Engine/Linker/Store/Instance calls if the version
    /// you restore differs (see Wasmtime's release notes). The P/Invoke
    /// backend (NativeCore) is the fallback and does not require this file.
    /// </summary>
    public sealed class WasmCore : IVentiCore
    {
        private readonly Wasmtime.Engine _engine;
        private readonly Wasmtime.Store _store;
        private readonly Wasmtime.Instance _instance;
        private readonly string _wasmPath;

        public WasmCore(string wasmPath = "venti.wasm")
        {
            _wasmPath = wasmPath;
            _engine = new Wasmtime.Engine();
            _store = new Wasmtime.Store(_engine);
            var wasi = new Wasmtime.Wasi(_engine,
                new Wasmtime.WasiConfiguration().WithInheritedStandardOutput());
            _store.SetWasi(wasi);

            var module = Wasmtime.Module.FromFile(_engine, wasmPath);
            var linker = new Wasmtime.Linker(_engine);
            linker.DefineWasi();
            _instance = linker.Instantiate(_store, module);
        }

        private Func<double, double, double> F2(string name) =>
            _instance.GetFunction<double, double, double>(name) ??
            throw new InvalidOperationException($"missing export {name} in {_wasmPath}");

        private Func<double, double, double, double> F3(string name) =>
            _instance.GetFunction<double, double, double, double>(name) ??
            throw new InvalidOperationException($"missing export {name} in {_wasmPath}");

        public SectionResult VelocityMethodRound(double flowrate, double targetVelocity)
        {
            // venti_velocity_method_round(flowrate, target, &diam_m, &vel)
            // For the scalar demo we reuse the native-style export signature;
            // a pointer-based helper writes to Wasm linear memory. See #15.
            throw new NotImplementedException(
                "WasmCore.VelocityMethodRound uses out-params in Wasm memory; wire in #15.");
        }

        public SectionResult EqualFrictionMethodRound(double flowrate, double targetPaPerM,
            double absoluteRoughness, double density, double dynamicViscosity)
        {
            throw new NotImplementedException(
                "WasmCore equal-friction out-param marshalling wired in #15.");
        }

        public double FrictionFactor(double re, double relRoughness) =>
            F2("venti_friction_factor")(re, relRoughness);

        public double Reynolds(double velocity, double dHyd, double kinVisc) =>
            F3("venti_reynolds")(velocity, dHyd, kinVisc);

        public double RelativeRoughness(double absRough, double dHyd) =>
            F2("venti_relative_roughness")(absRough, dHyd);

        public double LocalPressureDrop(double zeta, double v, double rho) =>
            F3("venti_local_pressure_drop")(zeta, v, rho);

        public double StraightPressureDrop(double f, double len, double dHyd, double v, double rho)
        {
            var fn = _instance.GetFunction<double, double, double, double, double, double>(
                "venti_straight_pressure_drop")
                ?? throw new InvalidOperationException("missing venti_straight_pressure_drop");
            return fn(f, len, dHyd, v, rho);
        }

        public double RegeneratedNoiseRound(double v, double d, double rho) =>
            F3("venti_regenerated_noise_round")(v, d, rho);

        public double DuctPressureLevel(double lw, double area, double alpha) =>
            F3("venti_duct_pressure_level")(lw, area, alpha);

        public bool NcOk(string spaceType, double levelDb)
        {
            // venti_nc_ok needs a (ptr,len) string in Wasm memory; simple stub
            // until the pointer helper is added in #15. Native backend fully works.
            throw new NotImplementedException("WasmCore.NcOk string marshalling wired in #15.");
        }

        public double RequiredZeta(double dpPa, double v, double rho) =>
            F3("venti_required_zeta")(dpPa, v, rho);

        public double DamperOpenPercentage(double zeta) =>
            F2("venti_damper_open_percentage")(zeta);

        public void Dispose()
        {
            _store?.Dispose();
            _engine?.Dispose();
        }
    }
}
