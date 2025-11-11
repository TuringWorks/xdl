---
layout: default
title: Quick Start
parent: Getting Started
nav_order: 1
---

# Quick Start Guide

Get up and running with XDL in minutes.

## Installation

```bash
git clone https://github.com/TuringWorks/xdl
cd xdl
cargo build --release
cargo install --path xdl-cli
```

## Your First XDL Program

Create a file `hello.xdl`:

```xdl
; Hello World in XDL
print, 'Hello, XDL!'

; Basic arithmetic
x = 10
y = 20
print, 'x + y =', x + y
```

Run it:

```bash
xdl hello.xdl
```

## Interactive REPL

Launch the interactive REPL:

```bash
xdl
```

Try some commands:

```xdl
XDL> x = findgen(100)
XDL> y = sin(x * !pi / 50)
XDL> print, 'Min:', min(y), 'Max:', max(y)
XDL> .quit
```

## Next Steps

- [Installation Guide](installation) - Detailed installation
- [Examples](../README) - More sample code
- [Graphics Quick Start](../QUICKSTART_GRAPHICS) - Learn plotting
- [Core Features](../core) - Language reference

For the complete quick start documentation, see [QUICK_START](../QUICK_START).
