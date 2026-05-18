"""Internal helpers shared by the Mojo-backed Python shims.

The Mojo ext modules raise plain ``Error`` which Python sees as a generic
``Exception``. ``translate_error`` re-raises those as the more specific
exception type the public API contract documents (typically
``ValueError`` for validation failures).
"""

from __future__ import annotations

from typing import Any, Callable, TypeVar

T = TypeVar("T")


def translate_error(fn: Callable[..., T], *args: Any, **kwargs: Any) -> T:
    """Call ``fn(*args, **kwargs)``; turn any ``Exception`` into ``ValueError``.

    The Mojo side raises a generic ``Error`` that PyMojo surfaces as a bare
    ``Exception`` in Python. Most of our shimmed math contracts ``ValueError``
    on invalid inputs — this helper keeps that contract without each shim
    re-rolling the same try/except.
    """
    try:
        return fn(*args, **kwargs)
    except Exception as e:
        raise ValueError(str(e)) from e
