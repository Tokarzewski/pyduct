using System;

namespace Venti.Core
{
    /// <summary>Selectable compute backends.</summary>
    public enum VentiBackend
    {
        /// <summary>P/Invoke the native venti cdylib (libventi.so / venti.dll).</summary>
        Native,
        /// <summary>Embed venti.wasm via the Wasmtime .NET SDK.</summary>
        Wasm,
    }

    /// <summary>
    /// Creates an <see cref="IVentiCore"/> for a chosen backend (issue #14).
    /// The choice can be persisted in the plugin config (e.g. a registry key
    /// or settings file) and swapped without touching command code.
    /// </summary>
    public static class VentiCoreFactory
    {
        /// <summary>Default backend; override via <see cref="ConfiguredBackend"/>.</summary>
        public static VentiBackend DefaultBackend { get; set; } = VentiBackend.Native;

        /// <summary>Backend selected by the current (persisted) configuration.</summary>
        public static VentiBackend ConfiguredBackend { get; set; } = VentiBackend.Native;

        public static IVentiCore Create() => Create(ConfiguredBackend);

        public static IVentiCore Create(VentiBackend backend) => backend switch
        {
            VentiBackend.Wasm => new WasmCore("venti.wasm"),
            _ => new NativeCore(),
        };
    }
}
