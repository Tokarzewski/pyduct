using System;
using System.Collections.Generic;
using System.Globalization;

namespace Wenta
{
    public sealed class DrawingLink
    {
        public string FromGuid;

        public string ToGuid;

        public string FromPort;

        public string ToPort;
    }

    public sealed class DrawingDocument
    {
        public string Scope;

        public Network Network;

        public Dictionary<string, ComponentMeta> Meta;
    }

    public static class MultiDrawing
    {

        public static DrawingDocument LoadDocument(string json, string scope)
        {
            if (string.IsNullOrEmpty(scope))
                throw new WentaException("MultiDrawing.LoadDocument: drawing scope is required");

            Dictionary<string, ComponentMeta> meta;
            Network net = NetworkJson.Parse(json, out meta);
            if (meta == null) meta = new Dictionary<string, ComponentMeta>();

            foreach (KeyValuePair<string, Component> kv in net.Components)
            {
                ComponentMeta m;
                if (!meta.TryGetValue(kv.Key, out m) || m == null)
                {
                    m = new ComponentMeta();
                    meta[kv.Key] = m;
                }
                if (string.IsNullOrEmpty(m.DrawingScope)) m.DrawingScope = scope;
                if (string.IsNullOrEmpty(m.Guid)) m.Guid = System.Guid.NewGuid().ToString("D");
            }

            var doc = new DrawingDocument();
            doc.Scope = scope;
            doc.Network = net;
            doc.Meta = meta;
            return doc;
        }

        public static string QualifiedId(string scope, string componentId)
        {
            return scope + "/" + componentId;
        }


        public static Network Merge(IList<DrawingDocument> docs, IList<DrawingLink> links,
                                    out Dictionary<string, ComponentMeta> mergedMeta)
        {
            if (docs == null || docs.Count == 0)
                throw new WentaException("MultiDrawing.Merge: no drawing documents");

            var ownerDoc = new Dictionary<string, int>();
            var ownerCid = new Dictionary<string, string>();
            for (int i = 0; i < docs.Count; i++)
            {
                DrawingDocument doc = docs[i];
                if (doc == null || doc.Network == null)
                    throw new WentaException("MultiDrawing.Merge: document "
                        + i.ToString(CultureInfo.InvariantCulture) + " is empty");
                if (string.IsNullOrEmpty(doc.Scope))
                    throw new WentaException("MultiDrawing.Merge: document "
                        + i.ToString(CultureInfo.InvariantCulture) + " has no drawing scope");

                foreach (KeyValuePair<string, Component> kv in doc.Network.Components)
                {
                    string guid = GuidOf(doc, kv.Key);
                    if (string.IsNullOrEmpty(guid)) continue;
                    if (ownerDoc.ContainsKey(guid))
                        throw new WentaException("MultiDrawing.Merge: guid '" + guid
                            + "' is used by component '" + ownerCid[guid] + "' of drawing '"
                            + docs[ownerDoc[guid]].Scope + "' and by component '" + kv.Key
                            + "' of drawing '" + doc.Scope + "'");
                    ownerDoc[guid] = i;
                    ownerCid[guid] = kv.Key;
                }
            }

            var merged = new Network();
            var name = "";
            mergedMeta = new Dictionary<string, ComponentMeta>();

            for (int i = 0; i < docs.Count; i++)
            {
                DrawingDocument doc = docs[i];
                name += (name.Length > 0 ? "+" : "") + doc.Scope;
                foreach (KeyValuePair<string, Component> kv in doc.Network.Components)
                {
                    string qid = QualifiedId(doc.Scope, kv.Key);
                    merged.Add(qid, kv.Value);
                    var m = new ComponentMeta();
                    m.Guid = GuidOf(doc, kv.Key);
                    m.DrawingScope = ScopeOf(doc, kv.Key);
                    mergedMeta[qid] = m;
                }
            }
            merged.Name = name;

            foreach (DrawingDocument doc in docs)
            {
                foreach (KeyValuePair<string, Component> kv in doc.Network.Components)
                {
                    foreach (Port p in kv.Value.Ports)
                    {
                        if (!p.IsIn) continue;
                        foreach (string pred in doc.Network.Predecessors(
                                     Network.PortNodeId(kv.Key, p.Name)))
                        {
                            int colon = pred.LastIndexOf(':');
                            if (colon < 0) continue;
                            string srcId = pred.Substring(0, colon);
                            string srcPort = pred.Substring(colon + 1);
                            merged.Connect(QualifiedId(doc.Scope, srcId) + "." + srcPort,
                                           QualifiedId(doc.Scope, kv.Key) + "." + p.Name);
                        }
                    }
                }
            }

            if (links != null)
            {
                foreach (DrawingLink link in links)
                {
                    if (link == null)
                        throw new WentaException("MultiDrawing.Merge: null drawing link");
                    int fromDoc = RequireOwner(ownerDoc, link.FromGuid, "from");
                    int toDoc = RequireOwner(ownerDoc, link.ToGuid, "to");
                    if (fromDoc == toDoc)
                        throw new WentaException("MultiDrawing.Merge: link '" + link.FromGuid
                            + "' -> '" + link.ToGuid + "' has both ends in drawing '"
                            + docs[fromDoc].Scope + "'; use Network.Connect for a link inside one drawing");
                    merged.Connect(
                        QualifiedId(docs[fromDoc].Scope, ownerCid[link.FromGuid]) + PortSuffix(link.FromPort),
                        QualifiedId(docs[toDoc].Scope, ownerCid[link.ToGuid]) + PortSuffix(link.ToPort));
                }
            }
            return merged;
        }

        public static string ToJson(Network merged, Dictionary<string, ComponentMeta> mergedMeta)
        {
            return NetworkJson.Serialize(merged, mergedMeta);
        }


        public static List<string> Validate(IList<DrawingDocument> docs, IList<DrawingLink> links)
        {
            var problems = new List<string>();
            if (docs == null || docs.Count == 0)
            {
                problems.Add("no drawing documents");
                return problems;
            }

            var ownerDoc = new Dictionary<string, int>();
            var ownerCid = new Dictionary<string, string>();
            for (int i = 0; i < docs.Count; i++)
            {
                DrawingDocument doc = docs[i];
                string where = "document " + i.ToString(CultureInfo.InvariantCulture);
                if (doc == null || doc.Network == null) { problems.Add(where + " is empty"); continue; }
                if (string.IsNullOrEmpty(doc.Scope)) { problems.Add(where + " has no drawing scope"); continue; }
                foreach (KeyValuePair<string, Component> kv in doc.Network.Components)
                {
                    string guid = GuidOf(doc, kv.Key);
                    if (string.IsNullOrEmpty(guid))
                    {
                        problems.Add("component '" + kv.Key + "' of drawing '" + doc.Scope
                            + "' has no guid, so no link can reference it");
                        continue;
                    }
                    if (ownerDoc.ContainsKey(guid))
                    {
                        problems.Add("duplicate guid '" + guid + "': component '" + ownerCid[guid]
                            + "' of drawing '" + docs[ownerDoc[guid]].Scope + "' and component '"
                            + kv.Key + "' of drawing '" + doc.Scope + "'");
                        continue;
                    }
                    ownerDoc[guid] = i;
                    ownerCid[guid] = kv.Key;
                }
            }

            var fed = new List<int>();
            if (links != null)
            {
                foreach (DrawingLink link in links)
                {
                    if (link == null) { problems.Add("null drawing link"); continue; }
                    bool knownFrom = link.FromGuid != null && ownerDoc.ContainsKey(link.FromGuid);
                    bool knownTo = link.ToGuid != null && ownerDoc.ContainsKey(link.ToGuid);
                    if (!knownFrom)
                        problems.Add("link references unknown guid '" + (link.FromGuid ?? "null") + "'");
                    if (!knownTo)
                        problems.Add("link references unknown guid '" + (link.ToGuid ?? "null") + "'");
                    if (!knownFrom || !knownTo) continue;

                    int fromDoc = ownerDoc[link.FromGuid], toDoc = ownerDoc[link.ToGuid];
                    if (fromDoc == toDoc)
                    {
                        problems.Add("link '" + link.FromGuid + "' -> '" + link.ToGuid
                            + "' has both ends in drawing '" + docs[fromDoc].Scope
                            + "'; use Network.Connect for a link inside one drawing");
                        continue;
                    }
                    CheckPort(problems, docs[fromDoc], ownerCid[link.FromGuid], link.FromPort, false);
                    CheckPort(problems, docs[toDoc], ownerCid[link.ToGuid], link.ToPort, true);
                    if (!fed.Contains(toDoc)) fed.Add(toDoc);
                }
            }

            for (int i = 0; i < docs.Count; i++)
            {
                DrawingDocument doc = docs[i];
                if (doc == null || doc.Network == null || string.IsNullOrEmpty(doc.Scope)) continue;
                if (fed.Contains(i)) continue;
                if (doc.Network.Sources().Count > 0) continue;
                problems.Add("drawing '" + doc.Scope
                    + "' has no Source and no incoming link, so nothing feeds it");
            }
            return problems;
        }


        private static void CheckPort(List<string> problems, DrawingDocument doc, string cid,
                                      string portName, bool wantIn)
        {
            string side = wantIn ? "arrives at" : "leaves";
            string kind = wantIn ? "an in-port" : "an out-port";
            Component c = doc.Network.Components[cid];
            if (string.IsNullOrEmpty(portName))
            {
                int n = 0;
                foreach (Port p in c.Ports) if (p.IsIn == wantIn) n++;
                if (n == 0)
                    problems.Add("link " + side + " component '" + QualifiedId(doc.Scope, cid)
                        + "', which has no " + kind);
                else if (n > 1)
                    problems.Add("link " + side + " component '" + QualifiedId(doc.Scope, cid)
                        + "', which has " + n.ToString(CultureInfo.InvariantCulture)
                        + " " + kind + "s; name one");
                return;
            }
            foreach (Port p in c.Ports)
            {
                if (p.Name != portName) continue;
                if (p.IsIn != wantIn)
                    problems.Add("link " + side + " port '" + QualifiedId(doc.Scope, cid) + "."
                        + portName + "', which is not " + kind);
                return;
            }
            problems.Add("link " + side + " port '" + QualifiedId(doc.Scope, cid) + "."
                + portName + "', which does not exist");
        }

        private static int RequireOwner(Dictionary<string, int> ownerDoc, string guid, string side)
        {
            int index;
            if (guid == null || !ownerDoc.TryGetValue(guid, out index))
                throw new WentaException("MultiDrawing.Merge: link '" + side + "' guid '"
                    + (guid ?? "null") + "' is not owned by any drawing document");
            return index;
        }

        private static string PortSuffix(string portName)
        {
            return string.IsNullOrEmpty(portName) ? "" : "." + portName;
        }

        private static string GuidOf(DrawingDocument doc, string cid)
        {
            ComponentMeta m;
            if (doc.Meta == null || !doc.Meta.TryGetValue(cid, out m) || m == null) return null;
            return m.Guid;
        }

        private static string ScopeOf(DrawingDocument doc, string cid)
        {
            ComponentMeta m;
            if (doc.Meta == null || !doc.Meta.TryGetValue(cid, out m) || m == null || string.IsNullOrEmpty(m.DrawingScope))
                return doc.Scope;
            return m.DrawingScope;
        }
    }
}
