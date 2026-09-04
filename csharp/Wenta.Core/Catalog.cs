using System;
using System.Collections.Generic;
using System.IO;
using System.Web.Script.Serialization;

namespace Wenta
{
    /// <summary>Open ζ-catalog: pluggable manufacturer loss data.
    ///
    /// The competitive answer to Wentyle's sponsored vendor libraries and
    /// Ventpack's PartShelf24: an *open* JSON format anyone can produce.
    /// Fittings are matched by id or by (type + size window); each entry
    /// carries provenance so drawings document which data sized them.
    ///
    /// Format (catalog JSON):
    /// {
    ///   "name": "example-generic-rect",
    ///   "version": 1,
    ///   "fittings": [
    ///     {
    ///       "id": "rect-elbow-r1.0",
    ///       "type": "rect_elbow",           // rect_elbow | round_elbow | tee |
    ///                                        // damper | diffuser | grille | ...
    ///       "size_min_mm": [100, 100],      // inclusive window [w,h] or [d]
    ///       "size_max_mm": [1200, 2000],
    ///       "zeta": 0.21,
    ///       "source": "Hendiger tab. 4.2",
    ///       "knr": "2.08.02.01"             // optional KNR estimate code
    ///     }
    ///   ]
    /// }</summary>
    public sealed class ZetaCatalog
    {
        public sealed class CatalogEntry
        {
            public string Id;
            public string Type;
            public double[] SizeMinMm;   // may be null (applies to all sizes)
            public double[] SizeMaxMm;   // may be null
            public double Zeta;
            public string Source;
            public string Knr;
        }

        public string Name;
        public int Version;
        public readonly List<CatalogEntry> Fittings = new List<CatalogEntry>();

        /// <summary>Load a catalog from a JSON file.</summary>
        public static ZetaCatalog Load(string path)
        {
            string json = File.ReadAllText(path);
            return Parse(json, path);
        }

        public static ZetaCatalog Parse(string json, string origin)
        {
            var ser = new JavaScriptSerializer();
            var root = ser.Deserialize<Dictionary<string, object>>(json);
            if (root == null)
                throw new WentaException("invalid catalog JSON: " + origin);
            var cat = new ZetaCatalog();
            if (root.ContainsKey("name")) cat.Name = (string)root["name"];
            cat.Version = root.ContainsKey("version")
                ? Convert.ToInt32(root["version"]) : 1;
            if (!root.ContainsKey("fittings"))
                throw new WentaException("catalog JSON has no 'fittings' array: " + origin);
            var fittings = (System.Collections.ArrayList)root["fittings"];
            foreach (object o in fittings)
            {
                var f = (Dictionary<string, object>)o;
                var e = new CatalogEntry();
                e.Id = (string)f["id"];
                e.Type = f.ContainsKey("type") ? (string)f["type"] : null;
                e.Zeta = Convert.ToDouble(f["zeta"]);
                e.Source = f.ContainsKey("source") ? (string)f["source"] : null;
                e.Knr = f.ContainsKey("knr") ? (string)f["knr"] : null;
                e.SizeMinMm = ToDoubles(f.ContainsKey("size_min_mm") ? f["size_min_mm"] : null);
                e.SizeMaxMm = ToDoubles(f.ContainsKey("size_max_mm") ? f["size_max_mm"] : null);
                cat.Fittings.Add(e);
            }
            return cat;
        }

        /// <summary>Exact-id lookup.</summary>
        public CatalogEntry ById(string id)
        {
            foreach (CatalogEntry e in Fittings)
                if (e.Id == id) return e;
            return null;
        }

        /// <summary>Match by type and size window (mm). First match wins;
        /// entries without a size window match any size of that type.</summary>
        public CatalogEntry Match(string type, double[] sizeMm)
        {
            foreach (CatalogEntry e in Fittings)
            {
                if (e.Type != type) continue;
                if (e.SizeMinMm == null) return e;
                bool ok = true;
                for (int i = 0; i < sizeMm.Length && i < e.SizeMinMm.Length; i++)
                {
                    if (sizeMm[i] < e.SizeMinMm[i] || sizeMm[i] > e.SizeMaxMm[i])
                    { ok = false; break; }
                }
                if (ok) return e;
            }
            return null;
        }

        /// <summary>ζ for (type, size); falls back to the built-in
        /// correlation library when the catalog has no match.</summary>
        public double ZetaFor(string type, double[] sizeMm)
        {
            CatalogEntry e = Match(type, sizeMm);
            if (e != null) return e.Zeta;
            return FittingsCorrelationFallback(type, sizeMm);
        }

        private static double FittingsCorrelationFallback(string type, double[] s)
        {
            switch (type)
            {
                case "rect_elbow":
                    return FittingsLibrary.RectangularElbow(s[0] / 1000.0, s[1] / 1000.0,
                                                             s[0] / 1000.0, 90.0);
                case "mitered_elbow":
                    return FittingsLibrary.MiteredElbow(90.0, false);
                case "damper":
                    return FittingsLibrary.DamperButterfly(100.0);
                case "diffuser":
                    return FittingsLibrary.DiffuserCeiling(1.0);
                case "grille":
                    return FittingsLibrary.GrilleReturn(0.15);
                case "tee":
                    return 0.5;
                default:
                    throw new WentaException(
                        "no catalog entry and no correlation for type '" + type + "'");
            }
        }

        private static double[] ToDoubles(object o)
        {
            if (o == null) return null;
            var arr = (System.Collections.ArrayList)o;
            var d = new double[arr.Count];
            for (int i = 0; i < arr.Count; i++)
                d[i] = Convert.ToDouble(arr[i]);
            return d;
        }
    }
}
