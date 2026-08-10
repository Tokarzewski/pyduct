using System;
using Venti.Core;
using Xunit;

namespace Venti.Core.Tests
{
    /// <summary>
    /// Binding tests (issue #18): the C# facade must return the same numbers as
    /// the venti Rust/Node references. NativeCore tests P/Invoke the cdylib;
    /// if the native lib isn't present (no Rust build on this machine) they
    /// skip. These run headlessly — no ZWCAD required.
    /// </summary>
    public class CoreBindingTests
    {
        // Standard air (mirrors venti STANDARD_AIR).
        private const double Rho = 1.204;
        private const double Mu = 1.825e-5;
        private const double Nu = Mu / Rho;

        private static IVentiCore TryNative()
        {
            try
            {
                var c = VentiCoreFactory.Create(VentiBackend.Native);
                c.FrictionFactor(50000, 0.0009); // force DllImport resolution
                return c;
            }
            catch (DllNotFoundException) { return null; }
            catch (EntryPointNotFoundException) { return null; }
            catch (BadImageFormatException) { return null; }
        }

        [Fact]
        public void Native_velocity_method_round_returns_200mm_duct()
        {
            using var core = TryNative();
            if (core == null) return; // native lib not present; skip

            var sized = core.VelocityMethodRound(0.1, 4.0);
            Assert.True(sized.VelocityMs <= 4.0);
            Assert.Equal(0.2, sized.DiameterM, 3); // 200 mm standard duct
            // v = Q/A for D=0.2: 0.1/(pi*0.01)
            Assert.Equal(0.1 / (Math.PI * 0.01), sized.VelocityMs, 4);
        }

        [Fact]
        public void Native_friction_and_drop_match_reference()
        {
            using var core = TryNative();
            if (core == null) return;

            // Swamee-Jain at Re=5e4, eps/D=9e-4 -> ~0.02364
            double f = core.FrictionFactor(50000.0, 0.0009);
            Assert.Equal(0.0236446, f, 4);

            // local drop zeta=1, v=4, rho=1.204 -> 1.204*16/2 = 9.632
            Assert.Equal(9.632, core.LocalPressureDrop(1.0, 4.0, Rho), 3);
        }

        [Fact]
        public void Native_chain_solve_matches_23_28_Pa()
        {
            using var core = TryNative();
            if (core == null) return;

            double q = 0.1, d = 0.2, length = 20.0, rough = 0.0001;
            double area = Math.PI * (d / 2) * (d / 2);
            double v = q / area;
            double f = core.FrictionFactor(
                core.Reynolds(v, d, Nu), core.RelativeRoughness(rough, d));
            double total = core.StraightPressureDrop(f, length, d, v, Rho)
                         + core.LocalPressureDrop(0.5, v, Rho)
                         + core.LocalPressureDrop(1.0, v, Rho);
            Assert.Equal(23.284, total, 2); // matches Rust/Node reference
        }

        [Fact]
        public void Native_balancing_matches_reference()
        {
            using var core = TryNative();
            if (core == null) return;

            // required_zeta for surplus 19.264 Pa at v=4, rho=1.204 -> 2.0
            Assert.Equal(2.0, core.RequiredZeta(19.264, 4.0, Rho), 3);
            // butterfly inversion is consistent with the native damper zeta
            double z = core.RequiredZeta(19.264, 4.0, Rho);
            double open = core.DamperOpenPercentage(z);
            Assert.InRange(open, 0.0, 100.0);
        }

        [Fact]
        public void Factory_returns_NativeCore_by_default()
        {
            var c = VentiCoreFactory.Create();
            Assert.IsType<NativeCore>(c);
        }
    }
}
