"""Native-Mojo solver kernel — critical-path DP on flat int-indexed arrays.

The Python solver projects its NetworkX graph into three flat structures:

* ``topo``  — ``List[Int]``: node indices in topological order (length N)
* ``preds`` — ``List[List[Int]]``: per-node predecessor index lists
* ``dp``    — ``List[Float64]``: per-node pressure-drop weight (length N)

This kernel walks ``topo`` once, accumulating the maximum weighted path
ending at each node, and returns the global maximum — i.e. the
critical-path pressure drop. O(V + E).
"""


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
