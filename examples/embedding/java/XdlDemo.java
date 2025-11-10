/**
 * XDL Java Bindings Demo
 */
public class XdlDemo {
    public static void main(String[] args) {
        System.out.println("XDL Java Bindings Demo");
        System.out.println("======================");

        try {
            // Test basic mathematical functions using static methods
            System.out.println("\n1. Mathematical Functions (Static):");
            System.out.printf("sin(π/2) = %.4f%n", XdlWrapper.Sin(Math.PI / 2));
            System.out.printf("cos(0) = %.4f%n", XdlWrapper.Cos(0));
            System.out.printf("sqrt(16) = %.4f%n", XdlWrapper.Sqrt(16));
            System.out.printf("exp(1) = %.4f%n", XdlWrapper.Exp(1));
            System.out.printf("log(e) = %.4f%n", XdlWrapper.Log(Math.E));

            // Test with instance
            System.out.println("\n2. Using XDL Instance:");
            XdlWrapper xdl = new XdlWrapper();

            double[] testAngles = {0, Math.PI/6, Math.PI/4, Math.PI/3, Math.PI/2};

            System.out.println("Angle (rad) | sin(x) | cos(x)");
            System.out.println("------------|--------|--------");

            for (double angle : testAngles) {
                double sinVal = xdl.sin(angle);
                double cosVal = xdl.cos(angle);
                System.out.printf("%.4f      | %.4f | %.4f%n", angle, sinVal, cosVal);
            }

            // Test performance comparison
            System.out.println("\n3. Performance Test:");
            runPerformanceTest(xdl);

            // Clean up
            xdl.close();

            System.out.println("\n✓ All tests completed successfully!");

        } catch (Exception e) {
            System.err.println("✗ Error: " + e.getMessage());
            e.printStackTrace();
            System.err.println("Make sure the XDL native library is available.");
        }
    }

    private static void runPerformanceTest(XdlWrapper xdl) {
        final int iterations = 100000;

        // Generate test data
        double[] testData = new double[iterations];
        java.util.Random random = new java.util.Random(42);
        for (int i = 0; i < iterations; i++) {
            testData[i] = random.nextDouble() * Math.PI * 2;
        }

        // Test XDL performance
        long xdlStart = System.nanoTime();
        double xdlSum = 0;
        for (double x : testData) {
            xdlSum += xdl.callFunction("sin", x);
        }
        long xdlTime = System.nanoTime() - xdlStart;

        // Test Java performance
        long javaStart = System.nanoTime();
        double javaSum = 0;
        for (double x : testData) {
            javaSum += Math.sin(x);
        }
        long javaTime = System.nanoTime() - javaStart;

        double xdlTimeMs = xdlTime / 1_000_000.0;
        double javaTimeMs = javaTime / 1_000_000.0;

        System.out.printf("XDL time:     %.2fms%n", xdlTimeMs);
        System.out.printf("Java time:    %.2fms%n", javaTimeMs);
        System.out.printf("Speedup:      %.2fx%n", javaTimeMs / xdlTimeMs);
        System.out.printf("Results match: %b%n", Math.abs(xdlSum - javaSum) < 1e-10);
    }
}
