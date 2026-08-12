//! Host-agnostic geometry & topology — the M3 "trace" and "draw" core.
//!
//! Turns 2D duct centreline polylines into a `venti::Network` and back, with
//! **no CAD dependency** (works headless). A thin CAD adapter later maps ZWCAD
//! entities ↔ polylines/segments; all topology math lives here.
//!
//! * [`trace`] — coalesce polylines into a graph, split at junctions/endpoints,
//!   and build a `Network` (Source / RigidDuct / Tee / Terminal) with flow
//!   rooted at one source endpoint. (Advances issue #19.)
//! * [`TracedSystem::flatten`] — project back into drawable [`Segment`]
//!   primitives. (Advances issue #20.)
//!
//! Scope: round ducts, one source, no closed loops (a tree). We support:
//! degree-1 endpoints (source / terminal) and degree-3 tees. Degree ≥ 4 and
//! cycles are rejected. Tee "straight/branch" legs are assigned by traversal
//! order (geometry-accurate alignment can be layered on later); connectivity —
//! and therefore the solved pressure drops — is exact.

use crate::Result;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::components::duct::RigidDuct;
use crate::components::fitting::{Source, Tee, Terminal};
use crate::core::geometry::Round;
use crate::network::{ComponentEnum, Network};

/// A polyline duct centreline (metres). Consecutive points are joined.
#[derive(Debug, Clone)]
pub struct Polyline {
    pub points: Vec<(f64, f64)>,
}

impl Polyline {
    pub fn new(points: Vec<(f64, f64)>) -> Self {
        Polyline { points }
    }
}

/// A drawable duct primitive returned by [`TracedSystem::flatten`].
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub component_id: String,
    pub start: (f64, f64),
    pub end: (f64, f64),
    /// Diameter [m] for round ducts.
    pub diameter: f64,
}

/// Options controlling [`trace`].
#[derive(Debug, Clone)]
pub struct TraceOptions {
    /// Coalescing tolerance for shared endpoints [m].
    pub snap: f64,
    /// Default round duct diameter [m].
    pub default_diameter: f64,
    /// Diameter [m] per chain id (overrides default).
    pub diameters: HashMap<String, f64>,
    /// Terminal flowrates [m³/s] per terminal id (0 if absent).
    pub flows: HashMap<String, f64>,
}

impl Default for TraceOptions {
    fn default() -> Self {
        TraceOptions {
            snap: 1e-4,
            default_diameter: 0.2,
            diameters: HashMap::new(),
            flows: HashMap::new(),
        }
    }
}

/// A maximal straight duct chain and its geometry.
#[derive(Debug, Clone)]
pub struct Chain {
    pub id: String,
    pub points: Vec<(f64, f64)>,
    pub length_m: f64,
    pub diameter: f64,
}

/// The result of tracing: a network plus the geometry needed to draw it.
#[derive(Debug, Clone)]
pub struct TracedSystem {
    pub network: Network,
    pub chains: Vec<Chain>,
}

impl TracedSystem {
    /// Project the traced ducts into drawable [`Segment`] primitives.
    pub fn flatten(&self) -> Vec<Segment> {
        self.chains
            .iter()
            .map(|c| Segment {
                component_id: c.id.clone(),
                start: c.points[0],
                end: *c.points.last().unwrap(),
                diameter: c.diameter,
            })
            .collect()
    }

    /// Total traced straight ductwork length [m].
    pub fn total_length_m(&self) -> f64 {
        self.chains.iter().map(|c| c.length_m).sum()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Vid(usize);

#[derive(Debug, Clone)]
struct Vertex {
    point: (f64, f64),
    degree: usize,
}

fn dist2(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)
}

/// Trace 2D polylines into a `TracedSystem`.
pub fn trace(polylines: &[Polyline], opts: &TraceOptions) -> Result<TracedSystem> {
    if polylines.is_empty() || polylines.iter().all(|p| p.points.len() < 2) {
        return Err("no usable polylines".into());
    }

    // 1. Coalesce vertices and build an undirected adjacency.
    let mut verts: Vec<Vertex> = Vec::new();
    let mut cache: HashMap<(i64, i64), Vid> = HashMap::new();
    let mut edges: Vec<(Vid, Vid)> = Vec::new();

    let snap_vert =
        |p: (f64, f64), verts: &mut Vec<Vertex>, cache: &mut HashMap<(i64, i64), Vid>| -> Vid {
            let key = ((p.0 / opts.snap) as i64, (p.1 / opts.snap) as i64);
            for dx in -1..=1i64 {
                for dy in -1..=1i64 {
                    if let Some(&v) = cache.get(&(key.0 + dx, key.1 + dy)) {
                        if dist2(verts[v.0].point, p) <= opts.snap * opts.snap {
                            return v;
                        }
                    }
                }
            }
            let id = Vid(verts.len());
            verts.push(Vertex {
                point: p,
                degree: 0,
            });
            cache.insert(key, id);
            id
        };

    for poly in polylines {
        if poly.points.len() < 2 {
            continue;
        }
        let mut prev = snap_vert(poly.points[0], &mut verts, &mut cache);
        for &p in &poly.points[1..] {
            let cur = snap_vert(p, &mut verts, &mut cache);
            if cur != prev {
                edges.push((prev, cur));
            }
            prev = cur;
        }
    }
    if verts.is_empty() {
        return Err("no usable geometry".into());
    }

    // 2. Degrees + adjacency.
    let n = verts.len();
    let mut deg = vec![0usize; n];
    let mut adj: Vec<Vec<Vid>> = vec![Vec::new(); n];
    for &(a, b) in &edges {
        deg[a.0] += 1;
        deg[b.0] += 1;
        adj[a.0].push(b);
        adj[b.0].push(a);
    }
    for (v, vert) in verts.iter_mut().enumerate() {
        vert.degree = deg[v];
    }

    // Reject unsupported geometry.
    for (i, v) in verts.iter().enumerate() {
        if v.degree >= 4 {
            return Err((format!(
                "vertex {i} has degree {}; only tees (degree 3) are supported",
                v.degree
            ))
            .into());
        }
    }

    // 3. Degree-1 ends; pick one source, the rest are terminals.
    let ends: Vec<Vid> = (0..n).filter(|&i| verts[i].degree == 1).map(Vid).collect();
    if ends.len() < 2 {
        return Err("a network needs at least one source and one terminal end".into());
    }
    let source_v = ends[0];

    // 4. Enumerate maximal chains between junction/end vertices (degree != 2).
    let mut chains: Vec<Chain> = Vec::new();
    let mut chain_ends: Vec<(Vid, Vid)> = Vec::new();
    let mut seen: HashSet<(usize, usize)> = HashSet::new(); // ordered (from,to)
    let mut chain_id = 0usize;

    for (v, _vert) in verts.iter().enumerate() {
        if deg[v] == 2 {
            continue;
        }
        let vv = Vid(v);
        for &nb in &adj[v] {
            if seen.contains(&(vv.0, nb.0)) {
                continue;
            }
            let mut path_pts = vec![verts[v].point];
            let mut cur = vv;
            let mut nxt = nb;
            let mut reached: Option<Vid> = None;
            loop {
                if seen.contains(&(cur.0, nxt.0)) {
                    break;
                }
                seen.insert((cur.0, nxt.0));
                seen.insert((nxt.0, cur.0)); // undirected visit
                path_pts.push(verts[nxt.0].point);
                if verts[nxt.0].degree != 2 {
                    reached = Some(nxt);
                    break;
                }
                // advance through the degree-2 vertex
                let nexts: Vec<Vid> = adj[nxt.0].iter().copied().filter(|&t| t != cur).collect();
                if nexts.is_empty() {
                    break;
                }
                cur = nxt;
                nxt = nexts[0];
            }
            if let Some(end) = reached {
                let id = format!("duct{chain_id}");
                chain_id += 1;
                let diameter = opts
                    .diameters
                    .get(&id)
                    .copied()
                    .unwrap_or(opts.default_diameter);
                let length_m = path_pts.windows(2).map(|w| dist2(w[0], w[1]).sqrt()).sum();
                chains.push(Chain {
                    id,
                    points: path_pts,
                    length_m,
                    diameter,
                });
                chain_ends.push((vv, end));
            }
        }
    }

    // 5. Root a BFS from the source over the junction graph (junctions = degree
    //    != 2 vertices) to assign flow direction and tee leg ports.
    // junction -> [(chain_idx, other_junction_vid)]
    let mut jadj: HashMap<Vid, Vec<(usize, Vid)>> = HashMap::new();
    for (ci, &(a, b)) in chain_ends.iter().enumerate() {
        jadj.entry(a).or_default().push((ci, b));
        jadj.entry(b).or_default().push((ci, a));
    }

    // chain -> (upstream_vid, downstream_vid)
    let mut dir: HashMap<usize, (Vid, Vid)> = HashMap::new();
    // tee_vid -> (chain_idx -> port name among {combined, straight, branch})
    let mut tee_leg: HashMap<Vid, HashMap<usize, String>> = HashMap::new();

    let mut visited: HashSet<Vid> = HashSet::new();
    let mut queue: VecDeque<(Vid, Option<usize>)> = VecDeque::new();
    queue.push_back((source_v, None));
    visited.insert(source_v);
    while let Some((jv, came_in)) = queue.pop_front() {
        let incident: Vec<(usize, Vid)> = jadj.get(&jv).cloned().unwrap_or_default();
        let mut downstream: Vec<(usize, Vid)> = incident
            .into_iter()
            .filter(|(c, _)| came_in != Some(*c))
            .collect();
        downstream.sort_by_key(|(c, _)| *c); // deterministic order
        for (k, (ci, other)) in downstream.iter().enumerate() {
            if visited.contains(other) {
                continue;
            }
            dir.insert(*ci, (jv, *other));
            // if this junction is a tee, classify its legs
            if verts[jv.0].degree == 3 {
                if let Some(came) = came_in {
                    tee_leg
                        .entry(jv)
                        .or_default()
                        .insert(came, "combined".into());
                }
                let port = if k == 0 {
                    "straight".into()
                } else {
                    "branch".into()
                };
                tee_leg.entry(jv).or_default().insert(*ci, port);
            }
            visited.insert(*other);
            queue.push_back((*other, Some(*ci)));
        }
    }

    // Reject unreachable junctions (would indicate a loop / unsupported input).
    for (v, vert) in verts.iter().enumerate() {
        if vert.degree != 2 && !visited.contains(&Vid(v)) {
            return Err((format!(
                "junction vertex {v} is unreachable from the source (loop/unsupported)"
            ))
            .into());
        }
    }

    // 6. Build the network.
    let mut net = Network::new("traced");
    let source_id = "src".to_string();
    net.add(&source_id, ComponentEnum::Source(Source::new("source")))?;

    // terminals: degree-1 ends other than the source
    let mut terminal_of: HashMap<Vid, String> = HashMap::new();
    for &e in &ends {
        if e == source_v {
            continue;
        }
        let tid = format!("term{}", terminal_of.len());
        let flow = opts.flows.get(&tid).copied().unwrap_or(0.0);
        terminal_of.insert(e, tid.clone());
        net.add(
            &tid,
            ComponentEnum::Terminal(Terminal::new(&tid, flow, None, 0.0)),
        )?;
    }

    // tees: degree-3 junctions
    let mut tee_of: HashMap<Vid, String> = HashMap::new();
    for (i, v) in verts.iter().enumerate() {
        if v.degree == 3 {
            let t = format!("tee{}", tee_of.len());
            tee_of.insert(Vid(i), t.clone());
            let r = Round::new(opts.default_diameter)?;
            net.add(&t, ComponentEnum::Tee(Tee::new(&t, r.area, 0.0, 0.5)))?;
        }
    }

    // ducts + connections
    // For each chain, `up` is the junction that feeds it, `down` the junction
    // it feeds. A tee's leg roles:
    //   - the chain feeding INTO a tee connects to its `combined` (In) port;
    //   - chains LEAVING a tee connect from its `straight`/`branch` (Out) legs.
    for (ci, ch) in chains.iter().enumerate() {
        let (up, down) = dir.get(&ci).copied().ok_or("chain direction missing")?;
        let r = Round::new(ch.diameter)?;
        net.add(
            &ch.id,
            ComponentEnum::RigidDuct(RigidDuct::new(
                &ch.id,
                r.area,
                r.hydraulic_diameter,
                ch.length_m,
                0.0001,
            )?),
        )?;

        let duct_in = format!("{}.inlet", ch.id);
        let duct_out = format!("{}.outlet", ch.id);

        // --- upstream leg (what feeds this duct) ---
        if let Some(tid) = terminal_of.get(&up) {
            return Err((format!("upstream end {tid} is a terminal (invalid tree)")).into());
        }
        if let Some(t) = tee_of.get(&up) {
            // this chain leaves the tee via one of its Out legs
            let port = tee_leg
                .get(&up)
                .and_then(|m| m.get(&ci))
                .cloned()
                .ok_or_else(|| format!("tee {t} has no leg assigned for chain {}", ch.id))?;
            net.connect(&format!("{t}.{port}"), &duct_in)?;
        } else {
            // degree-1 that isn't a terminal => the source root
            net.connect(&source_id, &duct_in)?;
        }

        // --- downstream leg (what this duct feeds) ---
        if let Some(tid) = terminal_of.get(&down) {
            net.connect(&duct_out, tid)?;
        } else if let Some(t) = tee_of.get(&down) {
            // this chain feeds INTO the tee's combined (In) leg
            net.connect(&duct_out, &format!("{t}.combined"))?;
        } else {
            return Err("downstream end resolved to the source (invalid tree)".into());
        }
    }

    Ok(TracedSystem {
        network: net,
        chains,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(pts: &[(f64, f64)]) -> Polyline {
        Polyline::new(pts.to_vec())
    }

    #[test]
    fn single_straight_run() {
        let sys = trace(&[run(&[(0.0, 0.0), (5.0, 0.0)])], &TraceOptions::default()).unwrap();
        assert_eq!(sys.chains.len(), 1);
        assert!((sys.total_length_m() - 5.0).abs() < 1e-9);
        // network: 1 source + 1 duct + 1 terminal
        assert_eq!(sys.network.len(), 3);
        assert_eq!(sys.flatten().len(), 1);
        assert!((sys.flatten()[0].diameter - 0.2).abs() < 1e-12);
    }

    #[test]
    fn two_collinear_polylines_make_one_chain() {
        // share the midpoint (degree-2 vertex at (1,0))
        let sys = trace(
            &[
                run(&[(0.0, 0.0), (1.0, 0.0)]),
                run(&[(1.0, 0.0), (3.0, 0.0)]),
            ],
            &TraceOptions::default(),
        )
        .unwrap();
        assert_eq!(sys.chains.len(), 1); // coalesced into one duct through the degree-2 vertex
        assert!((sys.total_length_m() - 3.0).abs() < 1e-9);
    }

    #[test]
    fn tee_split_creates_three_ducts_and_two_terminals() {
        let polylines = vec![
            run(&[(0.0, 1.0), (1.0, 1.0), (2.0, 1.0)]), // trunk, tee at (2,1)
            run(&[(2.0, 1.0), (3.0, 1.0)]),             // branch to term0
            run(&[(2.0, 1.0), (2.0, 0.0)]),             // branch to term1
        ];
        let opts = TraceOptions {
            diameters: [("duct1".into(), 0.3f64)].into_iter().collect(),
            flows: [("term0".into(), 0.06f64), ("term1".into(), 0.04f64)]
                .into_iter()
                .collect(),
            ..Default::default()
        };
        let mut sys = trace(&polylines, &opts).unwrap();
        // 3 ducts, 1 tee, 2 terminals, 1 source
        assert_eq!(sys.chains.len(), 3);
        assert_eq!(sys.network.len(), 7);
        assert_eq!(sys.total_length_m(), 4.0);

        // solve must succeed and give a positive critical-path drop on a trunk
        let fluid = crate::core::fluid::Fluid::new(1.204, 1.825e-5).unwrap();
        let dp = sys.network.solve(Some(&fluid)).unwrap();
        assert!(dp > 0.0, "critical-path {dp}");
        assert_eq!(sys.flatten().len(), 3);
    }

    #[test]
    fn rejects_degree4_junction() {
        // 4-way cross at origin
        let polylines = vec![
            run(&[(-1.0, 0.0), (0.0, 0.0), (1.0, 0.0)]),
            run(&[(0.0, -1.0), (0.0, 0.0), (0.0, 1.0)]),
        ];
        assert!(trace(&polylines, &TraceOptions::default()).is_err());
    }

    #[test]
    fn assigns_terminal_flowrates() {
        let polylines = vec![
            run(&[(0.0, 1.0), (2.0, 1.0)]),
            run(&[(2.0, 1.0), (3.0, 1.0)]),
            run(&[(2.0, 1.0), (2.0, 0.0)]),
        ];
        let flows: HashMap<String, f64> = [("term0".into(), 0.06f64), ("term1".into(), 0.04f64)]
            .into_iter()
            .collect();
        let opts = TraceOptions {
            flows,
            ..Default::default()
        };
        let sys = trace(&polylines, &opts).unwrap();
        // terminals got the given flowrates
        for (id, t) in sys.network.iter_components() {
            if let ComponentEnum::Terminal(tm) = t {
                let _ = tm;
                if id == "term0" {
                    assert!((tm.flowrate_demand - 0.06).abs() < 1e-9);
                }
                if id == "term1" {
                    assert!((tm.flowrate_demand - 0.04).abs() < 1e-9);
                }
            }
        }
    }
}
