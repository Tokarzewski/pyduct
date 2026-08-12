//! Clash detection — find overlapping / near-intersecting duct segments.
//!
//! Given the [`crate::topology::Segment`] set produced by
//! [`crate::topology::TracedSystem::flatten`], [`find_clashes`] reports every
//! distinct pair whose minimum 2D centreline distance is smaller than the sum
//! of the two radii plus a user-supplied clearance margin.

use crate::Result;

/// A detected clash between two duct segments.
///
/// `a` and `b` are the [`crate::topology::Segment::component_id`]s of the two
/// segments, ordered lexicographically so every pair is reported exactly once.
/// `distance_m` is the exact minimum distance between the two 2D centreline
/// segments [m].
#[derive(Debug, Clone, PartialEq)]
pub struct Clash {
    pub a: String,
    pub b: String,
    pub distance_m: f64,
}

/// A point in the plane.
type P = (f64, f64);

/// Squared norm of a vector.
#[inline]
fn norm2(v: (f64, f64)) -> f64 {
    v.0 * v.0 + v.1 * v.1
}

/// Dot product of two vectors.
#[inline]
fn dot(u: (f64, f64), v: (f64, f64)) -> f64 {
    u.0 * v.0 + u.1 * v.1
}

/// `b - a`.
#[inline]
fn sub(a: P, b: P) -> P {
    (b.0 - a.0, b.1 - a.1)
}

/// Distance between two points.
fn point_point(a: P, b: P) -> f64 {
    norm2(sub(a, b)).sqrt()
}

/// Distance from point `p` to segment `a`..`b`.
fn point_segment(p: P, a: P, b: P) -> f64 {
    let ab = sub(a, b);
    let len2 = norm2(ab);
    // Degenerate segment: distance to a single point.
    if len2 == 0.0 {
        return point_point(p, a);
    }
    let ap = sub(a, p);
    let t = (dot(ap, ab) / len2).clamp(0.0, 1.0);
    point_point(p, (a.0 + t * ab.0, a.1 + t * ab.1))
}

/// Exact minimum distance between two 2D line segments.
///
/// Handles the general (skew-like) case, parallel / collinear and degenerate
/// (zero-length) segments via the closest-feet / endpoint fall-backs.
pub(crate) fn segment_distance(p0: P, p1: P, q0: P, q1: P) -> f64 {
    let u = sub(p0, p1);
    let v = sub(q0, q1);
    let w = (p0.0 - q0.0, p0.1 - q0.1);

    let a = dot(u, u);
    let b = dot(u, v);
    let c = dot(v, v);
    let d = dot(u, w);
    let e = dot(v, w);

    let denom = a * c - b * b;

    // Parallel (or any degenerate) segments — reduce to a sequence of
    // point-segment distances, which is exact and covers collinear overlap.
    if denom <= f64::EPSILON {
        let mut best = f64::INFINITY;
        for d in [
            point_segment(p0, q0, q1),
            point_segment(p1, q0, q1),
            point_segment(q0, p0, p1),
            point_segment(q1, p0, p1),
        ] {
            if d < best {
                best = d;
            }
        }
        return best;
    }

    // Non-parallel: solve for the closest points on the two (infinite) lines,
    // clamping to the segment ranges.
    let s_c = (b * e - c * d) / denom;
    let t_c = (a * e - b * d) / denom;
    let s = s_c.clamp(0.0, 1.0);
    let t = t_c.clamp(0.0, 1.0);

    let p = (p0.0 + s * u.0, p0.1 + s * u.1);
    let q = (q0.0 + t * v.0, q0.1 + t * v.1);

    // If the un-clamped optimum lay inside both ranges this is exact;
    // otherwise fall back to the boundary closest-feet which is still exact
    // for segment-to-segment distance.
    if (0.0..=1.0).contains(&s_c) && (0.0..=1.0).contains(&t_c) {
        return point_point(p, q);
    }

    // Boundary candidate: closest point on each segment to the other segment's
    // endpoints and vice-versa.
    let mut best = f64::INFINITY;
    for d in [
        point_segment(p0, q0, q1),
        point_segment(p1, q0, q1),
        point_segment(q0, p0, p1),
        point_segment(q1, p0, p1),
    ] {
        if d < best {
            best = d;
        }
    }
    best
}

/// Find all duct-segment clashes.
///
/// For every distinct, unordered pair of [`crate::topology::Segment`]s the
/// exact minimum distance between the two 2D centreline segments is computed;
/// a pair is reported as a [`Clash`] when
///
/// ```text
/// distance < (d_a + d_b) / 2 + clearance_m
/// ```
///
/// where `d_a` / `d_b` are the segment diameters and `clearance_m` a required
/// gap between duct surfaces. The pair is ordered (`a` <= `b`) so each clash
/// is reported once.
///
/// # Errors
///
/// Returns an error when `clearance_m < 0`.
///
/// # Examples
///
/// ```
/// use venti::clash::{find_clashes, Clash};
/// use venti::topology::Segment;
///
/// // Two crossing segments: the distance between the centrelines is 0.
/// let segments = vec![
///     Segment { component_id: "A".into(), start: (0.0, -1.0), end: (0.0, 1.0), diameter: 0.2 },
///     Segment { component_id: "B".into(), start: (-1.0, 0.0), end: (1.0, 0.0), diameter: 0.2 },
/// ];
/// let clashes = find_clashes(&segments, 0.0).unwrap();
/// assert_eq!(clashes.len(), 1);
/// assert_eq!(clashes[0].a, "A");
/// assert_eq!(clashes[0].b, "B");
/// assert_eq!(clashes[0].distance_m, 0.0);
/// ```
///
/// ```
/// use venti::clash::{find_clashes, clash_count};
/// use venti::topology::Segment;
///
/// // Far-apart parallel segments: well beyond any combined radius + margin.
/// let segments = vec![
///     Segment { component_id: "x".into(), start: (0.0, 0.0), end: (10.0, 0.0), diameter: 0.1 },
///     Segment { component_id: "y".into(), start: (0.0, 5.0), end: (10.0, 5.0), diameter: 0.1 },
/// ];
/// assert_eq!(clash_count(&find_clashes(&segments, 0.0).unwrap()), 0);
/// ```
pub fn find_clashes(segments: &[crate::topology::Segment], clearance_m: f64) -> Result<Vec<Clash>> {
    if clearance_m < 0.0 {
        return Err("clearance_m must be >= 0".into());
    }

    let mut clashes = Vec::new();
    for i in 0..segments.len() {
        for j in (i + 1)..segments.len() {
            let a = &segments[i];
            let b = &segments[j];
            let dist = segment_distance(a.start, a.end, b.start, b.end);
            let threshold = (a.diameter + b.diameter) / 2.0 + clearance_m;
            if dist < threshold {
                // Normalise the pair ordering so a <= b (lexicographic).
                let (name_a, name_b) = if a.component_id <= b.component_id {
                    (&a.component_id, &b.component_id)
                } else {
                    (&b.component_id, &a.component_id)
                };
                clashes.push(Clash {
                    a: name_a.clone(),
                    b: name_b.clone(),
                    distance_m: dist,
                });
            }
        }
    }
    Ok(clashes)
}

/// Number of clashes in a clash list.
pub fn clash_count(clashes: &[Clash]) -> usize {
    clashes.len()
}

/// Render clashes as CSV with a `a,b,distance_m` header.
///
/// ```
/// use venti::clash::{Clash, clashes_as_csv};
///
/// let clashes = vec![
///     Clash { a: "A".into(), b: "B".into(), distance_m: 0.0 },
///     Clash { a: "C".into(), b: "D".into(), distance_m: 0.05 },
/// ];
/// let csv = clashes_as_csv(&clashes);
/// assert_eq!(csv, "a,b,distance_m\nA,B,0\nC,D,0.05\n");
/// ```
pub fn clashes_as_csv(clashes: &[Clash]) -> String {
    let mut out = String::from("a,b,distance_m\n");
    for c in clashes {
        out.push_str(&format!("{},{},{}\n", c.a, c.b, c.distance_m));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(id: &str, p0: P, p1: P, diameter: f64) -> crate::topology::Segment {
        crate::topology::Segment {
            component_id: id.into(),
            start: p0,
            end: p1,
            diameter,
        }
    }

    #[test]
    fn crossing_segments_clash() {
        let segments = vec![
            seg("A", (0.0, -1.0), (0.0, 1.0), 0.2),
            seg("B", (-1.0, 0.0), (1.0, 0.0), 0.2),
        ];
        let clashes = find_clashes(&segments, 0.0).unwrap();
        assert_eq!(clashes.len(), 1);
        assert_eq!(clashes[0].distance_m, 0.0);
        assert_eq!((clashes[0].a.as_str(), clashes[0].b.as_str()), ("A", "B"));
    }

    #[test]
    fn far_parallel_segments_do_not_clash() {
        let segments = vec![
            seg("x", (0.0, 0.0), (10.0, 0.0), 0.1),
            seg("y", (0.0, 5.0), (10.0, 5.0), 0.1),
        ];
        assert_eq!(clash_count(&find_clashes(&segments, 0.0).unwrap()), 0);
    }

    #[test]
    fn clearance_margin_shifts_threshold() {
        // Just-touching surfaces: distance == radius sum, no clash at 0 clearance.
        let segments = vec![
            seg("a", (0.0, 0.0), (10.0, 0.0), 0.2),
            seg("b", (0.0, 0.2), (10.0, 0.2), 0.2),
        ];
        // distance 0.2 == (0.2+0.2)/2 = 0.2 -> not < threshold, no clash.
        assert_eq!(clash_count(&find_clashes(&segments, 0.0).unwrap()), 0);
        // With clearance, 0.2 < 0.2 + margin -> clash.
        assert_eq!(clash_count(&find_clashes(&segments, 0.1).unwrap()), 1);
        // With enough clearance the same margin still clashes; a big margin
        // also catches previously-far pairs.
        let far = vec![
            seg("a", (0.0, 0.0), (10.0, 0.0), 0.2),
            seg("b", (0.0, 3.0), (10.0, 3.0), 0.2),
        ];
        assert_eq!(clash_count(&find_clashes(&far, 0.0).unwrap()), 0);
        assert_eq!(clash_count(&find_clashes(&far, 3.0).unwrap()), 1);
    }

    #[test]
    fn identical_and_overlapping_segments_clash() {
        // Identical centreline -> distance 0.
        let identical = vec![
            seg("s1", (0.0, 0.0), (5.0, 0.0), 0.3),
            seg("s2", (0.0, 0.0), (5.0, 0.0), 0.3),
        ];
        let c = find_clashes(&identical, 0.0).unwrap();
        assert_eq!(clash_count(&c), 1);
        assert_eq!(c[0].distance_m, 0.0);

        // Partially overlapping collinear segments -> distance 0.
        let overlap = vec![
            seg("s1", (0.0, 0.0), (5.0, 0.0), 0.3),
            seg("s2", (3.0, 0.0), (8.0, 0.0), 0.3),
        ];
        let c = find_clashes(&overlap, 0.0).unwrap();
        assert_eq!(clash_count(&c), 1);
        assert_eq!(c[0].distance_m, 0.0);
    }

    #[test]
    fn collinear_overlap() {
        // Collinear segments sharing a stretch of centreline: distance 0,
        // reported as a clash at zero clearance.
        let segments = vec![
            seg("p", (0.0, 0.0), (5.0, 0.0), 0.2),
            seg("q", (4.0, 0.0), (9.0, 0.0), 0.2),
        ];
        let c = find_clashes(&segments, 0.0).unwrap();
        assert_eq!(clash_count(&c), 1);
        assert_eq!(c[0].distance_m, 0.0);

        // Collinear segments with a gap measure the exact gap distance and
        // only clash when it falls under the threshold.
        let gapped = vec![
            seg("p", (0.0, 0.0), (4.0, 0.0), 0.2),
            seg("q", (6.0, 0.0), (10.0, 0.0), 0.2),
        ];
        assert_eq!(clash_count(&find_clashes(&gapped, 0.0).unwrap()), 0);
        let c = find_clashes(&gapped, 2.5).unwrap();
        assert_eq!(clash_count(&c), 1);
        assert!((c[0].distance_m - 2.0).abs() < 1e-9);
    }

    #[test]
    fn clashes_as_csv_content() {
        let clashes = vec![
            Clash {
                a: "A".into(),
                b: "B".into(),
                distance_m: 0.0,
            },
            Clash {
                a: "C".into(),
                b: "D".into(),
                distance_m: 0.05,
            },
        ];
        assert_eq!(
            clashes_as_csv(&clashes),
            "a,b,distance_m\nA,B,0\nC,D,0.05\n"
        );
        assert_eq!(clashes_as_csv(&[]), "a,b,distance_m\n");
    }

    #[test]
    fn negative_clearance_is_rejected() {
        let segments = vec![seg("a", (0.0, 0.0), (1.0, 0.0), 0.1)];
        assert!(find_clashes(&segments, -1.0).is_err());
    }

    #[test]
    fn degenerate_zero_length_segment() {
        // A zero-length segment behaves like the single point it is.
        let segments = vec![
            seg("a", (0.0, 0.0), (0.0, 0.0), 0.2),
            seg("b", (0.1, 0.0), (0.1, 1.0), 0.2),
        ];
        let c = find_clashes(&segments, 0.0).unwrap();
        assert_eq!(clash_count(&c), 1);
        assert!((c[0].distance_m - 0.1).abs() < 1e-9);

        // Outside the combined radii + clearance: no clash.
        let far = vec![
            seg("a", (0.0, 0.0), (0.0, 0.0), 0.2),
            seg("b", (1.0, 0.0), (1.0, 1.0), 0.2),
        ];
        assert_eq!(clash_count(&find_clashes(&far, 0.0).unwrap()), 0);
    }
}
