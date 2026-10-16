using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Web.Script.Serialization;

namespace Wenta
{
    public sealed class KnrMap
    {
        public sealed class Override
        {
            public string Kind;
            public string Shape;
            public string Type;
            public string Code;
        }

        public const int SchemaVersion = 1;

        public string Edition;
        public int Version = SchemaVersion;
        public string Origin;

        public readonly Dictionary<string, string> Codes = new Dictionary<string, string>();
        public readonly List<Override> Overrides = new List<Override>();

        public readonly List<string> Unmapped = new List<string>();

        public static KnrMap Default()
        {
            var m = new KnrMap();
            m.Origin = "default";
            foreach (KeyValuePair<string, string> kv in Bom.KnrMap)
                m.Codes[kv.Key] = kv.Value;
            return m;
        }

        public static KnrMap Load(string path)
        {
            string json = File.ReadAllText(path);
            return Parse(json, path);
        }

        public static KnrMap Parse(string json, string origin)
        {
            var ser = new JavaScriptSerializer();
            Dictionary<string, object> root;
            try
            {
                root = ser.Deserialize<Dictionary<string, object>>(json);
            }
            catch (Exception ex)
            {
                throw new WentaException("invalid KNR map JSON: " + origin + " (" + ex.Message + ")");
            }
            if (root == null)
                throw new WentaException("invalid KNR map JSON: " + origin);

            var m = new KnrMap();
            m.Origin = origin;
            if (root.ContainsKey("edition")) m.Edition = root["edition"] as string;
            string label = "KNR map '" + (m.Edition ?? "?") + "' (" + origin + ")";

            m.Version = root.ContainsKey("schema_version")
                ? ParseVersion(root["schema_version"], label) : 1;
            if (m.Version > SchemaVersion)
                throw new WentaException(label + " declares schema_version "
                    + m.Version.ToString(CultureInfo.InvariantCulture)
                    + "; this build supports schema_version "
                    + SchemaVersion.ToString(CultureInfo.InvariantCulture) + " or older");

            var codes = root.ContainsKey("codes") ? root["codes"] as Dictionary<string, object> : null;
            if (codes == null)
                throw new WentaException(label + ": 'codes' must be an object of kind -> code");
            foreach (KeyValuePair<string, object> kv in codes)
            {
                if (kv.Key.StartsWith("_", StringComparison.Ordinal)) continue;
                m.Codes[kv.Key] = RequireString(kv.Value, label + ": codes['" + kv.Key + "']");
            }

            if (root.ContainsKey("overrides") && root["overrides"] != null)
            {
                var list = root["overrides"] as System.Collections.ArrayList;
                if (list == null)
                    throw new WentaException(label + ": 'overrides' must be an array");
                int index = 0;
                foreach (object o in list)
                {
                    string where = label + ": overrides[" + index.ToString(CultureInfo.InvariantCulture) + "]";
                    var entry = o as Dictionary<string, object>;
                    if (entry == null)
                        throw new WentaException(where + " is not an object");
                    var match = entry.ContainsKey("match") ? entry["match"] as Dictionary<string, object> : null;
                    if (match == null)
                        throw new WentaException(where + " has no 'match' object");
                    var ov = new Override();
                    ov.Kind = match.ContainsKey("kind") ? match["kind"] as string : null;
                    if (string.IsNullOrEmpty(ov.Kind))
                        throw new WentaException(where + ".match has no 'kind'");
                    ov.Shape = OptionalString(match, "shape", where + ".match.shape");
                    ov.Type = OptionalString(match, "type", where + ".match.type");
                    if (!entry.ContainsKey("code"))
                        throw new WentaException(where + " has no 'code'");
                    ov.Code = RequireString(entry["code"], where + ".code");
                    m.Overrides.Add(ov);
                    index++;
                }
            }
            return m;
        }

        public string CodeFor(string kind, string shape = null, string type = null)
        {
            if (kind == null) throw new WentaException("KnrMap.CodeFor: kind is null");
            foreach (Override ov in Overrides)
            {
                if (ov.Kind != kind) continue;
                if (ov.Shape != null && ov.Shape != shape) continue;
                if (ov.Type != null && ov.Type != type) continue;
                return ov.Code;
            }
            string code;
            if (Codes.TryGetValue(kind, out code)) return code;

            string query = QueryKey(kind, shape, type);
            if (!Unmapped.Contains(query)) Unmapped.Add(query);
            return null;
        }

        public static string QueryKey(string kind, string shape, string type)
        {
            string query = kind;
            if (shape != null) query += " shape=" + shape;
            if (type != null) query += " type=" + type;
            return query;
        }


        private static int ParseVersion(object v, string label)
        {
            if (v == null || v is string || v is bool)
                throw new WentaException(label + ": 'schema_version' must be a positive integer");
            double d = Convert.ToDouble(v, CultureInfo.InvariantCulture);
            if (d != Math.Floor(d) || d < 1.0 || d > int.MaxValue)
                throw new WentaException(label + ": 'schema_version' must be a positive integer, got "
                    + d.ToString("R", CultureInfo.InvariantCulture));
            return (int)d;
        }

        private static string RequireString(object v, string where)
        {
            string s = v as string;
            if (s == null)
                throw new WentaException(where + " must be a string");
            return s;
        }

        private static string OptionalString(Dictionary<string, object> obj, string key, string where)
        {
            if (!obj.ContainsKey(key) || obj[key] == null) return null;
            return RequireString(obj[key], where);
        }
    }
}
