using System;
using System.IO;
using System.Windows.Forms;
using ZwSoft.ZwCAD.Runtime;
using ZwSoft.ZwCAD.Windows;

[assembly: ExtensionApplication(typeof(WentaZwcad.Plugin))]

namespace WentaZwcad
{
    /// <summary>
    /// Plugin entry point. The palette is no longer auto-opened at startup —
    /// the ribbon (Wenta.CUIX) is the primary UI; run WENTAPANEL to open the
    /// sizing palette on demand.
    /// </summary>
    public class Plugin : IExtensionApplication
    {
        // stable GUID so ZWCAD remembers dock position / visibility
        private static readonly Guid SetId =
            new Guid("7C2E7FE9-6D2A-4A5B-9E1C-2F8B4D3A5C60");

        private static PaletteSet _set;

        public void Initialize()
        {
            Log("plugin loaded (wenta C# core)");
        }

        public void Terminate()
        {
        }

        [CommandMethod("WENTAPANEL")]
        public void OpenPanel()
        {
            if (_set == null)
            {
                _set = new PaletteSet("Wenta", SetId);
                _set.Style = PaletteSetStyles.Snappable;
                _set.MinimumSize = new System.Drawing.Size(240, 260);
                _set.Add("Wenta", new WentaPanel());
            }
            _set.Visible = true;
            Log("WENTAPANEL ok");
        }

        internal static void Log(string message)
        {
            try
            {
                File.AppendAllText(LogPath,
                    string.Format("{0:u}  {1}\r\n", DateTime.Now, message));
            }
            catch (System.Exception) { }
        }

        private static string LogPath
        {
            get { return Path.Combine(Path.GetTempPath(), "wenta_zwcad_test.txt"); }
        }
    }
}
