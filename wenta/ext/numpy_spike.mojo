"""Spike: read/write a numpy ndarray via UnsafePointer (zero-copy)."""

from std.os import abort
from std.python import Python, PythonObject
from std.python.bindings import PythonModuleBuilder


def sum_ndarray(arr: PythonObject) raises -> PythonObject:
    """Sum a contiguous float64 ndarray via raw pointer."""
    var n = Int(py=arr.shape[0])
    var addr = Int(py=arr.ctypes.data)
    var ptr = UnsafePointer[Float64, MutExternalOrigin](unsafe_from_address=addr)
    var s: Float64 = 0.0
    for i in range(n):
        s += ptr[i]
    return PythonObject(s)


def fill_ndarray(arr: PythonObject, scale: PythonObject) raises -> PythonObject:
    """Write scale*i into arr[i] via raw pointer (in-place)."""
    var n = Int(py=arr.shape[0])
    var addr = Int(py=arr.ctypes.data)
    var ptr = UnsafePointer[Float64, MutExternalOrigin](unsafe_from_address=addr)
    var s = Float64(py=scale)
    for i in range(n):
        ptr[i] = s * Float64(i)
    return Python.none()


@export
def PyInit_numpy_spike() -> PythonObject:
    try:
        var m = PythonModuleBuilder("numpy_spike")
        m.def_function[sum_ndarray]("sum_ndarray")
        m.def_function[fill_ndarray]("fill_ndarray")
        return m.finalize()
    except e:
        abort(String("failed to create numpy_spike: ", e))
        return PythonObject()
