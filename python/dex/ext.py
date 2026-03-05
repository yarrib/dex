"""Extension API for dex.

Provides utilities for registering custom templates, hooks, and commands.
This module is the public API for building on top of dex.
"""

from __future__ import annotations

from dex.cli import DexGroup, create_cli, passthrough
from dex.config import DexConfig, RemoteSource, load_config
from dex.passthrough import PassthroughSpec

__all__ = [
    "DexConfig",
    "DexGroup",
    "PassthroughSpec",
    "RemoteSource",
    "create_cli",
    "load_config",
    "passthrough",
]
