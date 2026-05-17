"""Native-Mojo solver kernel — critical-path DP on flat int-indexed arrays.

The Python solver projects its NetworkX graph into three flat structures:

* ``topo``  — ``List[Int]``: node indices in topological order (length N)
* ``preds`` — ``List[List[Int]]``: per-node predecessor index lists
* ``dp``    — ``List[Float64]``: per-node pressure-drop weight (length N)

This kernel walks ``topo`` once, accumulating the maximum weighted path
ending at each node, and returns the global maximum — i.e. the
critical-path pressure drop. O(V + E).
"""


def propagate_flows(
    topo: List[Int], preds: List[List[Int]], var flows: List[Float64]
) raises -> List[Float64]:
    """Reverse-topo flow walk on flat arrays.

    ``flows`` is pre-seeded with terminal demands at the terminal in-port
    indices and zeros everywhere else. The walk visits nodes in *reverse*
    topological order; each node's accumulated flow is pushed onto every
    predecessor. Returns the same list (mutated).

    NOTE: The Python solver does **not** currently call this — per-element
    PythonObject conversion of an N-float list across the boundary makes
    the round-trip more expensive than the original pure-Python walk for
    the network sizes we've benchmarked (≤ 500 nodes). Kept as the
    reference kernel for a future combined-pass solver that crosses the
    boundary exactly once for both flow propagation and critical-path DP.
    """
    for i in range(len(topo) - 1, -1, -1):
        var node = topo[i]
        var f = flows[node]
        if f != 0.0:
            var k = len(preds[node])
            for j in range(k):
                flows[preds[node][j]] += f
    return flows^


def critical_path_sum(
    topo: List[Int], preds: List[List[Int]], dp: List[Float64]
) raises -> Float64:
    """Return the longest weighted path's total (critical-path pressure drop).

    Inputs must be consistent: ``topo`` enumerates all nodes once,
    ``preds[i]`` lists node indices with edges into ``i``, and ``dp`` is
    indexed by the same node index.
    """
    var n = len(dp)
    var dist = List[Float64](length=n, fill=0.0)
    var max_dist: Float64 = 0.0
    for i in range(len(topo)):
        var node = topo[i]
        var best: Float64 = 0.0
        var k = len(preds[node])
        for j in range(k):
            var pd = dist[preds[node][j]]
            if pd > best:
                best = pd
        var d = best + dp[node]
        dist[node] = d
        if d > max_dist:
            max_dist = d
    return max_dist
