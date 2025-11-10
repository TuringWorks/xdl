import java.util.Arrays;

/**
 * Java wrapper for XDL scientific computing library
 */
public class XdlWrapper {
    // Load the native library
    static {
        try {
            // Try to load from common locations
            String[] possibleNames = {
                "xdl_ffi",           // Direct name
                "libxdl_ffi.so",     // Linux
                "libxdl_ffi.dylib",  // macOS
                "xdl_ffi.dll"        // Windows
            };

            boolean loaded = false;
            for (String libName : possibleNames) {
                try {
                    System.loadLibrary(libName.replaceAll("\\.(so|dylib|dll)$", ""));
                    loaded = true;
                    break;
                } catch (UnsatisfiedLinkError e) {
                    // Try next name
                }
            }

            if (!loaded) {
                // Try loading from current directory or full path
                String[] possiblePaths = {
                    "./libxdl_ffi.so",
                    "./libxdl_ffi.dylib",
                    "./xdl_ffi.dll",
                    "../target/release/libxdl_ffi.so",
                    "../target/release/libxdl_ffi.dylib",
                    "../target/release/xdl_ffi.dll"
                };

                for (String path : possiblePaths) {
                    try {
                        System.load(path);
                        loaded = true;
                        break;
                    } catch (UnsatisfiedLinkError e) {
                        // Try next path
                    }
                }
            }

            if (!loaded) {
                throw new RuntimeException("Could not load XDL native library. " +
                    "Make sure libxdl_ffi is built and available.");
            }

        } catch (Exception e) {
            throw new RuntimeException("Failed to load XDL library: " + e.getMessage(), e);
        }
    }

    // Native method declarations
    private static native long xdl_init();
    private static native void xdl_cleanup(long context);
    private static native double xdl_call_function(long context, String functionName,
                                                  double[] args, int nargs);

    private long context;

    /**
     * Create a new XDL context
     */
    public XdlWrapper() {
        this.context = xdl_init();
        if (context == 0) {
            throw new RuntimeException("Failed to initialize XDL context");
        }
    }

    /**
     * Call an XDL function with scalar arguments
     */
    public double callFunction(String functionName, double... args) {
        if (context == 0) {
            throw new IllegalStateException("XDL context not initialized");
        }
        return xdl_call_function(context, functionName, args, args.length);
    }

    /**
     * Clean up the XDL context
     */
    public void close() {
        if (context != 0) {
            xdl_cleanup(context);
            context = 0;
        }
    }

    // Convenience methods for common functions
    public double sin(double x) { return callFunction("sin", x); }
    public double cos(double x) { return callFunction("cos", x); }
    public double sqrt(double x) { return callFunction("sqrt", x); }
    public double exp(double x) { return callFunction("exp", x); }
    public double log(double x) { return callFunction("log", x); }

    @Override
    protected void finalize() throws Throwable {
        close();
        super.finalize();
    }

    /**
     * Static convenience methods using shared context
     */
    private static XdlWrapper sharedInstance;

    private static XdlWrapper getSharedInstance() {
        if (sharedInstance == null) {
            sharedInstance = new XdlWrapper();
        }
        return sharedInstance;
    }

    public static double Sin(double x) { return getSharedInstance().sin(x); }
    public static double Cos(double x) { return getSharedInstance().cos(x); }
    public static double Sqrt(double x) { return getSharedInstance().sqrt(x); }
    public static double Exp(double x) { return getSharedInstance().exp(x); }
    public static double Log(double x) { return getSharedInstance().log(x); }
}
