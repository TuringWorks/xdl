#!/usr/bin/env python3
"""
XDL Python Bindings

This module provides Python bindings for the XDL scientific computing library.
It uses ctypes to interface with the XDL C API.
"""

import ctypes
import os
import platform
from typing import Optional


class XdlError(Exception):
    """Exception raised for XDL errors"""
    pass


class XdlContext:
    """XDL execution context"""

    def __init__(self, lib_path: Optional[str] = None):
        """Initialize XDL context

        Args:
            lib_path: Path to XDL shared library. If None, tries to find it automatically.
        """
        self.lib = self._load_library(lib_path)
        self._context = None
        self._initialize()

    def _load_library(self, lib_path: Optional[str]) -> ctypes.CDLL:
        """Load the XDL shared library"""
        if lib_path is None:
            # Try to find the library in common locations
            system = platform.system().lower()
            if system == "darwin":  # macOS
                lib_name = "libxdl_ffi.dylib"
            elif system == "linux":
                lib_name = "libxdl_ffi.so"
            elif system == "windows":
                lib_name = "xdl_ffi.dll"
            else:
                raise RuntimeError(f"Unsupported platform: {system}")

            # Try relative paths
            possible_paths = [
                os.path.join(os.path.dirname(__file__), "..", "..", "..", "target", "debug", lib_name),
                os.path.join(os.path.dirname(__file__), "..", "..", "..", "target", "release", lib_name),
                f"./{lib_name}",
                lib_name,
            ]

            for path in possible_paths:
                if os.path.exists(path):
                    lib_path = path
                    break
            else:
                raise RuntimeError(f"Could not find XDL library. Tried: {possible_paths}")

        lib = ctypes.CDLL(lib_path)

        # Define function signatures
        lib.xdl_init.restype = ctypes.c_void_p
        lib.xdl_cleanup.argtypes = [ctypes.c_void_p]
        lib.xdl_call_function.argtypes = [
            ctypes.c_void_p, ctypes.c_char_p,
            ctypes.POINTER(ctypes.c_double), ctypes.c_int
        ]
        lib.xdl_call_function.restype = ctypes.c_double

        return lib

    def _initialize(self):
        """Initialize the XDL context"""
        self._context = self.lib.xdl_init()
        if self._context is None:
            raise XdlError("Failed to initialize XDL context")

    def __del__(self):
        """Clean up the XDL context"""
        if hasattr(self, '_context') and self._context is not None:
            self.lib.xdl_cleanup(self._context)

    def call_function(self, name: str, *args) -> float:
        """Call an XDL function

        Args:
            name: Function name
            *args: Function arguments (scalars)

        Returns:
            Function result as float
        """
        if not args:
            c_args = None
            nargs = 0
        else:
            c_args = (ctypes.c_double * len(args))(*args)
            nargs = len(args)

        name_bytes = name.encode('utf-8')
        result = self.lib.xdl_call_function(
            self._context, name_bytes,
            c_args, nargs
        )

        return float(result)


# Convenience functions

def sin(x: float) -> float:
    """Compute sine"""
    ctx = XdlContext()
    return ctx.call_function("sin", x)

def cos(x: float) -> float:
    """Compute cosine"""
    ctx = XdlContext()
    return ctx.call_function("cos", x)

def sqrt(x: float) -> float:
    """Compute square root"""
    ctx = XdlContext()
    return ctx.call_function("sqrt", x)


if __name__ == "__main__":
    # Example usage
    print("XDL Python Bindings Demo")
    print("=" * 30)

    try:
        # Test basic functions
        print(f"sin(1.57) = {sin(1.57):.4f}")
        print(f"cos(0) = {cos(0):.4f}")
        print(f"sqrt(16) = {sqrt(16):.4f}")

        print("\nAll tests passed!")

    except Exception as e:
        print(f"Error: {e}")
        print("Make sure the XDL library is built and available.")
