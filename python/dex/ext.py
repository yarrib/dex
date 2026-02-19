"""Extension API for dex.

Provides utilities for registering custom templates, hooks, and commands.
This module is the public API for building on top of dex.
"""

from __future__ import annotations

from dex.cli import DexGroup, create_cli
from dex.passthrough import PassthroughSpec

__all__ = [
    "DexGroup",
    "PassthroughSpec",
    "create_cli",
]
