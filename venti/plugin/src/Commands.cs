using ZwSoft.ZwCAD.ApplicationServices;
using ZwSoft.ZwCAD.EditorInput;
using ZwSoft.ZwCAD.Runtime;

namespace Venti.Plugin
{
    /// <summary>
    /// ZWCAD .NET command entry points for the venti ductwork-design plugin
    /// (issue #13 — scaffold). Load with NETLOAD, then run VENTI.
    ///
    /// The ZWCAD .NET API is source-compatible with AutoCAD's; binding behind
    /// the host-agnostic <see cref="IVentiCore"/> (issue #14) keeps the math
    /// isolated in the Rust/WASM core.
    /// </summary>
    public static class Commands
    {
        /// <summary>
        /// VENTI — top-level command; confirms the plugin is loaded and reports
        /// the bundled venti core (later commands build on IVentiCore, #14).
        /// </summary>
        [CommandMethod("VENTI")]
        public static void Venti()
        {
            Editor ed = ActiveEditor();
            ed.WriteMessage("\nventi: ZWCAD ductwork-design plugin scaffold loaded.");
            ed.WriteMessage("\nventi: run VENTI_SIZE to size a duct (wiring in #15).");
        }

        /// <summary>
        /// VENTI_SIZE — prompts for a flowrate and (placeholder) would size a
        /// round duct. The actual sizing lands with IVentiCore (#14) + the
        /// binding (#15/#16).
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
            // TODO(#15): size via IVentiCore.SizeRound(...) and report D + v.
            ed.WriteMessage("\nventi: sized flowrate {0:F3} m^3/s (binding not yet wired).",
                res.Value);
        }

        private static Editor ActiveEditor()
        {
            var doc = Application.DocumentManager.MdiActiveDocument;
            if (doc == null)
                throw new System.InvalidOperationException("No active document.");
            return doc.Editor;
        }
    }
}
