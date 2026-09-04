using System;
using System.Globalization;
using System.Windows.Forms;
using ZwSoft.ZwCAD.ApplicationServices;

namespace WentaZwcad
{
    /// <summary>
    /// Dockable "Wenta" sizing palette: flow / target velocity / shape in,
    /// one click queues a fully-answered WENTADUCT in the active drawing.
    /// </summary>
    public class WentaPanel : UserControl
    {
        private readonly NumericUpDown _flow = new NumericUpDown();
        private readonly ComboBox _velocity = new ComboBox();
        private readonly ComboBox _shape = new ComboBox();
        private readonly Label _preview = new Label();

        public WentaPanel()
        {
            var header = new Label
            {
                Text = "Wenta — duct sizing",
                Dock = DockStyle.Top,
                Height = 32,
                TextAlign = System.Drawing.ContentAlignment.MiddleLeft,
                Padding = new Padding(6, 0, 0, 0),
                Font = new System.Drawing.Font("Segoe UI", 10F, System.Drawing.FontStyle.Bold)
            };

            int y = 40;
            Label MkLabel(string text, int top)
            {
                return new Label
                {
                    Text = text,
                    Left = 8, Top = top, Width = 90,
                    TextAlign = System.Drawing.ContentAlignment.MiddleLeft
                };
            }

            Controls.Add(MkLabel("Flow [m³/s]:", y));
            _flow.Left = 100; _flow.Top = y - 3; _flow.Width = 110;
            _flow.Minimum = 0.001m; _flow.Maximum = 20m;
            _flow.DecimalPlaces = 3; _flow.Increment = 0.01m;
            _flow.Value = 0.1m;
            _flow.ValueChanged += delegate { UpdatePreview(); };
            Controls.Add(_flow);

            y += 32;
            Controls.Add(MkLabel("Target v [m/s]:", y));
            _velocity.Left = 100; _velocity.Top = y - 3; _velocity.Width = 110;
            _velocity.DropDownStyle = ComboBoxStyle.DropDownList;
            foreach (string v in new[] { "2.0", "2.5", "3.0", "4.0", "5.0", "6.0", "7.5" })
                _velocity.Items.Add(v);
            _velocity.SelectedIndex = 3; // 4.0
            _velocity.SelectedIndexChanged += delegate { UpdatePreview(); };
            Controls.Add(_velocity);

            y += 32;
            Controls.Add(MkLabel("Shape:", y));
            _shape.Left = 100; _shape.Top = y - 3; _shape.Width = 110;
            _shape.DropDownStyle = ComboBoxStyle.DropDownList;
            _shape.Items.Add("Rectangular");
            _shape.Items.Add("Round");
            _shape.SelectedIndex = 0;
            _shape.SelectedIndexChanged += delegate { UpdatePreview(); };
            Controls.Add(_shape);

            _preview.Left = 8; _preview.Top = y + 32; _preview.Width = 220; _preview.Height = 26;
            _preview.ForeColor = System.Drawing.Color.Teal;
            Controls.Add(_preview);

            var btn = new Button
            {
                Text = "Draw sized duct section",
                Dock = DockStyle.Bottom,
                Height = 38,
                FlatStyle = FlatStyle.System
            };
            btn.Click += delegate
            {
                SendCommand(string.Format(
                    "_.WENTADUCT {0} {1} {2} ",
                    _shape.SelectedItem,
                    _flow.Value.ToString(CultureInfo.InvariantCulture),
                    _velocity.SelectedItem));
            };
            Controls.Add(btn);

            var btnInfo = new Button
            {
                Text = "Plugin info",
                Dock = DockStyle.Bottom,
                Height = 30,
                FlatStyle = FlatStyle.System
            };
            btnInfo.Click += delegate { SendCommand("_.WENTAHELLO "); };
            Controls.Add(btnInfo);

            var footer = new Label
            {
                Text = "wenta (pyduct) — MIT license",
                Dock = DockStyle.Bottom,
                Height = 20,
                TextAlign = System.Drawing.ContentAlignment.MiddleCenter,
                ForeColor = System.Drawing.Color.Gray
            };
            Controls.Add(footer);

            Controls.Add(header);
            UpdatePreview();
        }

        /// <summary>Live preview: the section WENTADUCT would pick right now.</summary>
        private void UpdatePreview()
        {
            try
            {
                double flow = (double)_flow.Value;
                double tv = double.Parse((string)_velocity.SelectedItem,
                                         CultureInfo.InvariantCulture);
                bool round = (string)_shape.SelectedItem == "Round";
                Wenta.Sizing.SizingResult r = Wenta.Sizing.VelocityMethod(
                    flow,
                    round ? Wenta.Sizing.ShapeRound : Wenta.Sizing.ShapeRectangular,
                    tv);
                _preview.Text = "→ " + r.Section.Describe()
                    + "   v = " + r.Velocity.ToString("0.00") + " m/s";
            }
            catch (System.Exception ex)
            {
                _preview.Text = ex.Message;
            }
        }

        private static void SendCommand(string cmd)
        {
            Document doc = ZwSoft.ZwCAD.ApplicationServices.Application
                .DocumentManager.MdiActiveDocument;
            if (doc == null) return;
            doc.SendStringToExecute(cmd, true, true, true);
        }
    }
}
