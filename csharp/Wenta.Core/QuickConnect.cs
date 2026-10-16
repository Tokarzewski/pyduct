using System;
using System.Collections.Generic;
using System.Globalization;

namespace Wenta
{
    public enum ConnectorKind
    {
        Direct,
        Reducer,
        Expander,
        Transition,
        Flex,
        Spacer,
    }

    public sealed class ConnectorPiece
    {
        public ConnectorKind Kind;
        public CrossSection From;
        public CrossSection To;
        public double LengthM;
        public double Zeta;
        public double PressureDropPa;
        public string Note;
    }

    public sealed class QuickConnectOptions
    {
        public double MaxTaperAngleDeg = 15.0;

        public double GapM = 0.0;

        public double FlowrateM3s;

        public Fluid Fluid;

        public double MaxFlexLengthM = 1.5;

        public bool PreferFlex = false;

        public ZetaCatalog Catalog;
    }

    public static class QuickConnect
    {
        private const double Tol = 1e-9;

        public static readonly string[] Fields =
        {
            "kind",
            "from",
            "to",
            "length_m",
            "zeta",
            "pressure_drop_pa",
            "note",
        };

        public static List<ConnectorPiece> Plan(CrossSection from, CrossSection to,
                                                QuickConnectOptions options)
        {
            if (from == null) throw new WentaException("from cross-section is null");
            if (to == null) throw new WentaException("to cross-section is null");
            if (options == null) throw new WentaException("options is null");
            if (options.FlowrateM3s <= 0.0)
                throw new WentaException("flowrate must be positive, got "
                    + options.FlowrateM3s.ToString(CultureInfo.InvariantCulture));
            if (options.GapM < 0.0)
                throw new WentaException("gap_m must be non-negative, got "
                    + options.GapM.ToString(CultureInfo.InvariantCulture));
            if (options.MaxFlexLengthM < 0.0)
                throw new WentaException("max_flex_length_m must be non-negative, got "
                    + options.MaxFlexLengthM.ToString(CultureInfo.InvariantCulture));
            if (options.MaxTaperAngleDeg <= 0.0 || options.MaxTaperAngleDeg >= 90.0)
                throw new WentaException("max_taper_angle_deg must be in (0, 90), got "
                    + options.MaxTaperAngleDeg.ToString(CultureInfo.InvariantCulture));

            Fluid fluid = options.Fluid ?? Fluid.StandardAir();
            double rho = fluid.Density;
            double smallestArea = from.Area <= to.Area ? from.Area : to.Area;
            double velocity = options.FlowrateM3s / smallestArea;

            var chain = new List<ConnectorPiece>();
            bool same = SameSection(from, to);
            double gapLeft = options.GapM;

            if (!same)
            {
                ConnectorPiece taper = BuildTaper(from, to, options, velocity, rho);
                chain.Add(taper);
                gapLeft = options.GapM - taper.LengthM;
                if (options.GapM > Tol && gapLeft < -Tol)
                {
                    taper.Note = taper.Note + "; minimum taper length "
                        + Fmt(taper.LengthM) + " m exceeds the requested gap "
                        + Fmt(options.GapM)
                        + " m — the chain is longer than the gap, pull the ends apart";
                }
                if (gapLeft < Tol) gapLeft = 0.0;
            }

            CrossSection filler = same ? from : to;
            if (gapLeft > Tol)
            {
                bool useFlex = options.PreferFlex || !same;
                if (useFlex && options.MaxFlexLengthM > Tol)
                {
                    double flexLen = gapLeft <= options.MaxFlexLengthM
                        ? gapLeft : options.MaxFlexLengthM;
                    chain.Add(BuildFlex(filler, flexLen, options, velocity, rho));
                    gapLeft -= flexLen;
                }
                if (gapLeft > Tol)
                    chain.Add(BuildSpacer(filler, gapLeft));
            }

            if (chain.Count == 0)
            {
                chain.Add(new ConnectorPiece
                {
                    Kind = ConnectorKind.Direct,
                    From = from,
                    To = to,
                    LengthM = 0.0,
                    Zeta = 0.0,
                    PressureDropPa = 0.0,
                    Note = "identical sections, no gap: direct flanged/butt joint, zeta 0",
                });
            }
            return chain;
        }

        public static double TotalPressureDropPa(IList<ConnectorPiece> pieces)
        {
            if (pieces == null) return 0.0;
            double total = 0.0;
            foreach (ConnectorPiece p in pieces)
                if (p != null) total += p.PressureDropPa;
            return total;
        }

        public static double TotalLengthM(IList<ConnectorPiece> pieces)
        {
            if (pieces == null) return 0.0;
            double total = 0.0;
            foreach (ConnectorPiece p in pieces)
                if (p != null) total += p.LengthM;
            return total;
        }

        public static string ToCsv(IList<ConnectorPiece> pieces)
        {
            var lines = new List<string> { string.Join(",", Fields) };
            if (pieces == null) return lines[0];
            foreach (ConnectorPiece p in pieces)
            {
                if (p == null) continue;
                lines.Add(string.Join(",", new[]
                {
                    p.Kind.ToString(),
                    Quote(p.From == null ? "" : p.From.Describe()),
                    Quote(p.To == null ? "" : p.To.Describe()),
                    p.LengthM.ToString(CultureInfo.InvariantCulture),
                    p.Zeta.ToString(CultureInfo.InvariantCulture),
                    p.PressureDropPa.ToString(CultureInfo.InvariantCulture),
                    Quote(p.Note),
                }));
            }
            return string.Join("\n", lines.ToArray());
        }

        public static double TaperLengthM(CrossSection a, CrossSection b,
                                          double includedAngleDeg)
        {
            if (a == null || b == null)
                throw new WentaException("cross-sections must not be null");
            if (includedAngleDeg <= 0.0 || includedAngleDeg >= 90.0)
                throw new WentaException("included angle must be in (0, 90), got "
                    + includedAngleDeg.ToString(CultureInfo.InvariantCulture));
            double offset = WallOffsetM(a, b);
            return offset / Math.Tan(includedAngleDeg * Math.PI / 360.0);
        }

        public static double CharacteristicDiameterM(CrossSection s)
        {
            if (s == null) throw new WentaException("cross-section is null");
            Round r = s as Round;
            if (r != null) return r.Diameter;
            return Math.Sqrt(4.0 * s.Area / Math.PI);
        }


        private static double WallOffsetM(CrossSection a, CrossSection b)
        {
            Round ra = a as Round, rb = b as Round;
            if (ra != null && rb != null)
                return Math.Abs(ra.Diameter - rb.Diameter) / 2.0;

            double aw, ah, bw, bh;
            FaceSizes(a, out aw, out ah);
            FaceSizes(b, out bw, out bh);
            double dw = Math.Abs(aw - bw);
            double dh = Math.Abs(ah - bh);
            return (dw >= dh ? dw : dh) / 2.0;
        }

        private static void FaceSizes(CrossSection s, out double width, out double height)
        {
            Round r = s as Round;
            if (r != null) { width = r.Diameter; height = r.Diameter; return; }
            Rectangular q = s as Rectangular;
            if (q != null) { width = q.Width; height = q.Height; return; }
            double d = CharacteristicDiameterM(s);
            width = d;
            height = d;
        }

        private static bool IsRound(CrossSection s) { return s is Round; }

        private static bool SameSection(CrossSection a, CrossSection b)
        {
            Round ra = a as Round, rb = b as Round;
            if (ra != null && rb != null)
                return Math.Abs(ra.Diameter - rb.Diameter) < Tol;
            Rectangular qa = a as Rectangular, qb = b as Rectangular;
            if (qa != null && qb != null)
                return Math.Abs(qa.Width - qb.Width) < Tol
                    && Math.Abs(qa.Height - qb.Height) < Tol;
            if ((ra == null) != (rb == null)) return false;
            return Math.Abs(a.Area - b.Area) < Tol
                && Math.Abs(a.HydraulicDiameter - b.HydraulicDiameter) < Tol;
        }

        private static ConnectorPiece BuildTaper(CrossSection from, CrossSection to,
            QuickConnectOptions options, double velocity, double rho)
        {
            double angle = options.MaxTaperAngleDeg;
            double dFrom = CharacteristicDiameterM(from);
            double dTo = CharacteristicDiameterM(to);
            bool shapeChange = IsRound(from) != IsRound(to);
            bool contracting = to.Area < from.Area;

            ConnectorKind kind = shapeChange
                ? ConnectorKind.Transition
                : (contracting ? ConnectorKind.Reducer : ConnectorKind.Expander);
            string type = kind == ConnectorKind.Reducer ? "reducer"
                        : kind == ConnectorKind.Expander ? "expander"
                        : "transition";

            double zeta;
            string note;
            string catalogSource;
            var sizeMm = new double[] { dFrom * 1000.0, dTo * 1000.0 };
            if (TryCatalogZeta(options.Catalog, type, sizeMm, out zeta, out catalogSource))
            {
                note = "zeta from " + catalogSource;
            }
            else if (contracting)
            {
                zeta = FittingsLibrary.ReducerRound(dFrom, dTo, angle);
                note = "zeta from FittingsLibrary.ReducerRound(d_in="
                     + Fmt(dFrom) + ", d_out=" + Fmt(dTo) + ", angle="
                     + Fmt(angle) + " deg)";
            }
            else
            {
                zeta = FittingsLibrary.ExpanderRound(dFrom, dTo, angle);
                note = "zeta from FittingsLibrary.ExpanderRound(d_in="
                     + Fmt(dFrom) + ", d_out=" + Fmt(dTo) + ", angle="
                     + Fmt(angle) + " deg)";
            }
            if (shapeChange)
            {
                note = note + "; round<->rect transition costed on area-equivalent"
                     + " diameters (no dedicated transition correlation in"
                     + " FittingsLibrary)";
            }
            note = note + "; length from the "
                 + Fmt(angle) + " deg taper rule";

            return new ConnectorPiece
            {
                Kind = kind,
                From = from,
                To = to,
                LengthM = TaperLengthM(from, to, angle),
                Zeta = zeta,
                PressureDropPa = Losses.LocalPressureDrop(zeta, velocity, rho),
                Note = note,
            };
        }

        private static ConnectorPiece BuildFlex(CrossSection section, double lengthM,
            QuickConnectOptions options, double velocity, double rho)
        {
            double zeta;
            string catalogSource;
            double d = CharacteristicDiameterM(section);
            string note;
            if (TryCatalogZeta(options.Catalog, "flex",
                    new double[] { d * 1000.0, d * 1000.0 }, out zeta, out catalogSource))
            {
                note = "flexible connection; zeta from " + catalogSource;
            }
            else
            {
                zeta = 0.0;
                note = "flexible connection; zeta 0 - FittingsLibrary has no flex"
                     + " correlation and none is invented here: price its friction"
                     + " with a FlexDuct of this length (manufacturer Pa/m)";
            }
            if (lengthM >= options.MaxFlexLengthM - Tol)
                note = note + "; capped at max_flex_length_m " + Fmt(options.MaxFlexLengthM) + " m";
            return new ConnectorPiece
            {
                Kind = ConnectorKind.Flex,
                From = section,
                To = section,
                LengthM = lengthM,
                Zeta = zeta,
                PressureDropPa = Losses.LocalPressureDrop(zeta, velocity, rho),
                Note = note,
            };
        }

        private static ConnectorPiece BuildSpacer(CrossSection section, double lengthM)
        {
            return new ConnectorPiece
            {
                Kind = ConnectorKind.Spacer,
                From = section,
                To = section,
                LengthM = lengthM,
                Zeta = 0.0,
                PressureDropPa = 0.0,
                Note = "straight filler; zeta 0 - a spacer's loss is skin friction"
                     + " over its length, which is the duct's job, not the"
                     + " fitting's: model it as a RigidDuct of this length",
            };
        }

        private static bool TryCatalogZeta(ZetaCatalog catalog, string type, double[] sizeMm,
                                           out double zeta, out string source)
        {
            zeta = 0.0;
            source = null;
            if (catalog == null) return false;
            ZetaCatalog.CatalogEntry e = catalog.Match(type, sizeMm);
            if (e == null) return false;
            zeta = e.Zeta;
            source = "catalog entry '" + e.Id + "'"
                   + (string.IsNullOrEmpty(e.Source) ? "" : " (source " + e.Source + ")");
            return true;
        }

        private static string Fmt(double v)
        {
            return v.ToString("0.###", CultureInfo.InvariantCulture);
        }

        private static string Quote(string s)
        {
            if (s == null) return "";
            if (s.IndexOfAny(new[] { ',', '"', '\n', '\r' }) < 0) return s;
            return "\"" + s.Replace("\"", "\"\"") + "\"";
        }
    }
}
