// Package xdl provides Go bindings for the XDL scientific computing library
package xdl

/*
#cgo LDFLAGS: -L. -lxdl_ffi -lm
#cgo darwin LDFLAGS: -L. -lxdl_ffi
#cgo linux LDFLAGS: -L. -lxdl_ffi -lm
#cgo windows LDFLAGS: -L. -lxdl_ffi

#include <stdlib.h>

// XDL C API declarations
extern void* xdl_init();
extern void xdl_cleanup(void* context);
extern double xdl_call_function(void* context, const char* functionName, double* args, int nargs);
*/
import "C"
import (
	"errors"
	"unsafe"
)

// Context represents an XDL execution context
type Context struct {
	ptr unsafe.Pointer
}

// NewContext creates a new XDL context
func NewContext() (*Context, error) {
	ptr := C.xdl_init()
	if ptr == nil {
		return nil, errors.New("failed to initialize XDL context")
	}
	return &Context{ptr: ptr}, nil
}

// Close cleans up the XDL context
func (c *Context) Close() {
	if c.ptr != nil {
		C.xdl_cleanup(c.ptr)
		c.ptr = nil
	}
}

// CallFunction calls an XDL function with scalar arguments
func (c *Context) CallFunction(functionName string, args ...float64) float64 {
	if c.ptr == nil {
		panic("XDL context is closed")
	}

	funcNameC := C.CString(functionName)
	defer C.free(unsafe.Pointer(funcNameC))

	var argsPtr *C.double
	if len(args) > 0 {
		argsPtr = (*C.double)(unsafe.Pointer(&args[0]))
	}

	result := C.xdl_call_function(c.ptr, funcNameC, argsPtr, C.int(len(args)))
	return float64(result)
}

// Convenience methods for common mathematical functions
func (c *Context) Sin(x float64) float64  { return c.CallFunction("sin", x) }
func (c *Context) Cos(x float64) float64  { return c.CallFunction("cos", x) }
func (c *Context) Sqrt(x float64) float64 { return c.CallFunction("sqrt", x) }
func (c *Context) Exp(x float64) float64  { return c.CallFunction("exp", x) }
func (c *Context) Log(x float64) float64  { return c.CallFunction("log", x) }

// Global context for convenience functions
var defaultContext *Context

func init() {
	var err error
	defaultContext, err = NewContext()
	if err != nil {
		panic("Failed to initialize default XDL context: " + err.Error())
	}
}

// Global convenience functions
func Sin(x float64) float64  { return defaultContext.Sin(x) }
func Cos(x float64) float64  { return defaultContext.Cos(x) }
func Sqrt(x float64) float64 { return defaultContext.Sqrt(x) }
func Exp(x float64) float64  { return defaultContext.Exp(x) }
func Log(x float64) float64  { return defaultContext.Log(x) }

// CallFunction is a convenience function using the default context
func CallFunction(functionName string, args ...float64) float64 {
	return defaultContext.CallFunction(functionName, args...)
}
