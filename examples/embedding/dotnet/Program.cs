using System;
using System.Linq;
using XdlSharp;

namespace XdlDemo
{
    class Program
    {
        static void Main(string[] args)
        {
            Console.WriteLine("XDL .NET Bindings Demo");
            Console.WriteLine("======================");

            try
            {
                // Test basic mathematical functions
                Console.WriteLine("\n1. Mathematical Functions:");
                Console.WriteLine($"sin(π/2) = {XdlMath.Sin(Math.PI / 2):F4}");
                Console.WriteLine($"cos(0) = {XdlMath.Cos(0):F4}");
                Console.WriteLine($"sqrt(16) = {XdlMath.Sqrt(16):F4}");
                Console.WriteLine($"exp(1) = {XdlMath.Exp(1):F4}");
                Console.WriteLine($"log(e) = {XdlMath.Log(Math.E):F4}");

                // Test with context
                Console.WriteLine("\n2. Using XDL Context:");
                using (var ctx = new XdlContext())
                {
                    double[] testValues = { 0, Math.PI / 6, Math.PI / 4, Math.PI / 3, Math.PI / 2 };

                    Console.WriteLine("Angle (rad) | sin(x) | cos(x)");
                    Console.WriteLine("------------|--------|--------");

                    foreach (double x in testValues)
                    {
                        double sinVal = ctx.CallFunction("sin", x);
                        double cosVal = ctx.CallFunction("cos", x);
                        Console.WriteLine($"{x,10:F4} | {sinVal,6:F4} | {cosVal,6:F4}");
                    }
                }

                // Test performance comparison
                Console.WriteLine("\n3. Performance Test:");
                RunPerformanceTest();

                Console.WriteLine("\n✓ All tests completed successfully!");

            }
            catch (Exception ex)
            {
                Console.WriteLine($"✗ Error: {ex.Message}");
                Console.WriteLine("Make sure the XDL library is available in your PATH or specify the full path.");
            }
        }

        static void RunPerformanceTest()
        {
            const int iterations = 100000;
            var random = new Random(42);
            var testData = Enumerable.Range(0, iterations)
                                    .Select(_ => random.NextDouble() * Math.PI * 2)
                                    .ToArray();

            // Test XDL performance
            var xdlStart = DateTime.Now;
            double xdlSum = 0;
            using (var ctx = new XdlContext())
            {
                foreach (double x in testData)
                {
                    xdlSum += ctx.CallFunction("sin", x);
                }
            }
            var xdlTime = DateTime.Now - xdlStart;

            // Test .NET performance
            var dotnetStart = DateTime.Now;
            double dotnetSum = 0;
            foreach (double x in testData)
            {
                dotnetSum += Math.Sin(x);
            }
            var dotnetTime = DateTime.Now - dotnetStart;

            Console.WriteLine($"XDL time:     {xdlTime.TotalMilliseconds:F2}ms");
            Console.WriteLine($".NET time:    {dotnetTime.TotalMilliseconds:F2}ms");
            Console.WriteLine($"Speedup:      {dotnetTime.TotalMilliseconds / xdlTime.TotalMilliseconds:F2}x");
            Console.WriteLine($"Results match: {Math.Abs(xdlSum - dotnetSum) < 1e-10}");
        }
    }
}
