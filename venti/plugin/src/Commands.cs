using System;
using ZwSoft.ZwCAD.ApplicationServices;
using ZwSoft.ZwCAD.EditorInput;
using ZwSoft.ZwCAD.Runtime;

namespace Venti.Plugin
{
    /// <summary>
    /// ZWCAD .NET command entry points for the venti ductwork-design plugin
    /// (issues #13 scaffold, #16 wiring). Load with NETLOAD, then run VENTI,
    /// VENTI_SIZE or VENTI_SOLVE. Compute is behind <see cref="IVentiCore"/>.
    /// </summary>
    public static class Commands
    {
        // Standard dry air (matches venti STANDARD_AIR).
        private const double AirDensity = 1.204;          // kg/m^3
        private const double AirDynamicViscosity = 1.825e-5; // Pa.s
        private const double AirKinViscosity = AirDynamicViscosity / AirDensity; // m^2/s

        /// <summary>VENTI — banner + passive check that a backend loads.</summary>
        [CommandMethod("VENTI")]
        public static void Venti()
        {
            Editor ed = ActiveEditor();
            ed.WriteMessage("\nventi: ZWCAD ductwork-design plugin loaded (backend {0}).",
                VentiCoreFactory.DefaultBackend);
        }

        /// <summary>
        /// VENTI_SIZE — prompt for a flowrate and size a round duct by the
        /// velocity method (issue #15/#16).
        /// </summary>
        [CommandMethod("VENTI_SIZE")]
        public static void VentiSize()
        {
            Editor ed = ActiveEditor();
            var res = ed.GetDouble("\nventi: flowrate [m^3/s]: ");
            if (res.Status != PromptStatus.OK)
            {
                ed.WriteMessage("\nventi: cancelled.");
                return;
            }
            using (IVentiCore core = VentiCoreFactory.Create())
            {
                var sized = core.VelocityMethodRound(res.Value, targetVelocity: 4.0);
                var f = core.FrictionFactor(
                    core.Reynolds(sized.VelocityMs, sized.DiameterM, AirKinViscosity),
                    core.RelativeRoughness(0.0001, sized.DiameterM));
                ed.WriteMessage("\nventi: sized duct {0}  (f = {1:F4}).", sized, f);
            }
        }

        /// <summary>
        /// VENTI_SOLVE — solve a small Source → Duct → Fitting → Terminal chain
        /// and report its critical-path pressure drop (issue #16), using only
        /// IVentiCore primitives (venti physics).
        /// </summary>
        [CommandMethod("VENTI_SOLVE")]
        public static void VentiSolve()
        {
            Editor ed = ActiveEditor();
            using (IVentiCore core = VentiCoreFactory.Create())
            {
                // Example: Q=0.1 m3/s, D=0.2 m, L=20 m rigid duct, elbow zeta=0.5,
                // terminal zeta=1.0, galvanised steel roughness 0.0001 m.
                double q = 0.1, d = 0.2, length = 20.0, roughness = 0.0001;
                double zetaFit = 0.5, zetaTerminal = 1.0;

                double area = Math.PI * (d / 2) * (d / 2);
                double v = q / area;

                double re = core.Reynolds(v, d, AirKinViscosity);
                double relRough = core.RelativeRoughness(roughness, d);
                double f = core.FrictionFactor(re, relRough);

                double dpDuct = core.StraightPressureDrop(f, length, d, v, AirDensity);
                double dpFit = core.LocalPressureDrop(zetaFit, v, AirDensity);
                double dpTerminal = core.LocalPressureDrop(zetaTerminal, v, AirDensity);
                double dpTotal = dpDuct + dpFit + dpTerminal;

                ed.WriteMessage("\nventi: Q={0:F3} m3/s  D={1:F3} m  v={2:F2} m/s", q, d, v);
                ed.WriteMessage("\nventi: duct {0:F2} + fitting {1:F2} + terminal {2:F2} Pa",
                    dpDuct, dpFit, dpTerminal);
                ed.WriteMessage("\nventi: critical-path static pressure = {0:F2} Pa", dpTotal);
            }
        }

        private static Editor ActiveEditor()
        {
            var doc = Application.DocumentManager.MdiActiveDocument;
            if (doc == null)
                throw new InvalidOperationException("No active document.");
            return doc.Editor;
        }
    }
}
