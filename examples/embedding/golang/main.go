package main

import (
	"fmt"
	"math"
	"math/rand"
	"time"

	"./xdl" // Import the XDL package
)

func main() {
	fmt.Println("XDL Go Bindings Demo")
	fmt.Println("====================")

	// Test basic mathematical functions
	fmt.Println("\n1. Mathematical Functions:")
	fmt.Printf("sin(π/2) = %.4f\n", xdl.Sin(math.Pi/2))
	fmt.Printf("cos(0) = %.4f\n", xdl.Cos(0))
	fmt.Printf("sqrt(16) = %.4f\n", xdl.Sqrt(16))
	fmt.Printf("exp(1) = %.4f\n", xdl.Exp(1))
	fmt.Printf("log(e) = %.4f\n", xdl.Log(math.E))

	// Test with context
	fmt.Println("\n2. Using XDL Context:")
	ctx, err := xdl.NewContext()
	if err != nil {
		fmt.Printf("Error creating context: %v\n", err)
		return
	}
	defer ctx.Close()

	testAngles := []float64{0, math.Pi / 6, math.Pi / 4, math.Pi / 3, math.Pi / 2}

	fmt.Println("Angle (rad) | sin(x) | cos(x)")
	fmt.Println("------------|--------|--------")

	for _, angle := range testAngles {
		sinVal := ctx.Sin(angle)
		cosVal := ctx.Cos(angle)
		fmt.Printf("%.4f      | %.4f | %.4f\n", angle, sinVal, cosVal)
	}

	// Test performance comparison
	fmt.Println("\n3. Performance Test:")
	runPerformanceTest(ctx)

	fmt.Println("\n✓ All tests completed successfully!")
}

func runPerformanceTest(ctx *xdl.Context) {
	const iterations = 100000

	// Generate test data
	rand.Seed(42)
	testData := make([]float64, iterations)
	for i := range testData {
		testData[i] = rand.Float64() * math.Pi * 2
	}

	// Test XDL performance
	xdlStart := time.Now()
	xdlSum := 0.0
	for _, x := range testData {
		xdlSum += ctx.CallFunction("sin", x)
	}
	xdlTime := time.Since(xdlStart)

	// Test Go performance
	goStart := time.Now()
	goSum := 0.0
	for _, x := range testData {
		goSum += math.Sin(x)
	}
	goTime := time.Since(goStart)

	fmt.Printf("XDL time:     %.2fms\n", float64(xdlTime.Nanoseconds())/1e6)
	fmt.Printf("Go time:      %.2fms\n", float64(goTime.Nanoseconds())/1e6)
	fmt.Printf("Speedup:      %.2fx\n", float64(goTime.Nanoseconds())/float64(xdlTime.Nanoseconds()))
	fmt.Printf("Results match: %t\n", math.Abs(xdlSum-goSum) < 1e-10)
}
