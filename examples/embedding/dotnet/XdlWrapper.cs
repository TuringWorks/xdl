using System;
using System.Runtime.InteropServices;
using System.Collections.Generic;

namespace XdlSharp
{
    /// <summary>
    /// Exception thrown when XDL operations fail
    /// </summary>
    public class XdlException : Exception
    {
        public XdlException(string message) : base(message) { }
    }

    /// <summary>
    /// XDL context for managing library state
    /// </summary>
    public class XdlContext : IDisposable
    {
        private IntPtr _context;
        private bool _disposed = false;

        /// <summary>
        /// Initialize a new XDL context
        /// </summary>
        /// <param name="libraryPath">Path to XDL library (optional)</param>
        public XdlContext(string libraryPath = null)
        {
            if (!string.IsNullOrEmpty(libraryPath))
            {
                // Load the library explicitly if path provided
                IntPtr handle = NativeMethods.LoadLibrary(libraryPath);
                if (handle == IntPtr.Zero)
                {
                    throw new XdlException($"Failed to load XDL library from: {libraryPath}");
                }
            }

            _context = NativeMethods.xdl_init();
            if (_context == IntPtr.Zero)
            {
                throw new XdlException("Failed to initialize XDL context");
            }
        }

        /// <summary>
        /// Call an XDL function with scalar arguments
        /// </summary>
        public double CallFunction(string functionName, params double[] args)
        {
            if (_disposed)
                throw new ObjectDisposedException(nameof(XdlContext));

            IntPtr funcNamePtr = Marshal.StringToHGlobalAnsi(functionName);
            try
            {
                double[] argsArray = args ?? new double[0];
                double result = NativeMethods.xdl_call_function(
                    _context, funcNamePtr, argsArray, argsArray.Length);
                return result;
            }
            finally
            {
                Marshal.FreeHGlobal(funcNamePtr);
            }
        }

        /// <summary>
        /// Dispose the context and free resources
        /// </summary>
        public void Dispose()
        {
            if (!_disposed)
            {
                if (_context != IntPtr.Zero)
                {
                    NativeMethods.xdl_cleanup(_context);
                    _context = IntPtr.Zero;
                }
                _disposed = true;
            }
        }

        ~XdlContext()
        {
            Dispose();
        }
    }

    /// <summary>
    /// Convenience methods for common XDL functions
    /// </summary>
    public static class XdlMath
    {
        private static XdlContext _defaultContext;

        private static XdlContext DefaultContext
        {
            get
            {
                if (_defaultContext == null)
                    _defaultContext = new XdlContext();
                return _defaultContext;
            }
        }

        /// <summary>Compute sine</summary>
        public static double Sin(double x) => DefaultContext.CallFunction("sin", x);

        /// <summary>Compute cosine</summary>
        public static double Cos(double x) => DefaultContext.CallFunction("cos", x);

        /// <summary>Compute square root</summary>
        public static double Sqrt(double x) => DefaultContext.CallFunction("sqrt", x);

        /// <summary>Compute exponential</summary>
        public static double Exp(double x) => DefaultContext.CallFunction("exp", x);

        /// <summary>Compute natural logarithm</summary>
        public static double Log(double x) => DefaultContext.CallFunction("log", x);
    }

    /// <summary>
    /// P/Invoke declarations for XDL C API
    /// </summary>
    internal static class NativeMethods
    {
        private const string LibraryName = "xdl_ffi";

        [DllImport("kernel32.dll", SetLastError = true)]
        public static extern IntPtr LoadLibrary(string dllToLoad);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr xdl_init();

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void xdl_cleanup(IntPtr context);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern double xdl_call_function(
            IntPtr context,
            IntPtr functionName,
            [In] double[] args,
            int nargs);
    }
}
