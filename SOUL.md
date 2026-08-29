# The Soul of XDL

## Our Core Philosophy

XDL exists so that decades of working scientific code keeps working — faster,
safer, and on hardware its authors never had. We are not designing a new
language. We are honouring an existing one, and the people whose analyses depend
on it.

## What Drives Us

### **Compatibility Is a Promise, Not a Feature**

An IDL or GDL program that a researcher wrote in 2003 should produce the same
numbers today. When Rust's natural behaviour and IDL's documented behaviour
disagree, IDL wins. Every deviation is a bug against someone's published result,
even when it looks like an improvement.

### **A Wrong Number Is Worse Than an Error**

This is a data-analysis language. Its output goes into papers, models, and
decisions. A crash is visible and gets fixed in an afternoon; a plausible-looking
number that came from a fallback default can survive peer review. We would rather
return an error, a blank, or a `NaN` the user can see than a value nobody checked.

### **The Session Is the User's Workspace**

A panic in a built-in function does not fail one operation — it destroys hours of
loaded data and interactive state. Total error handling is not stylistic
preference here; it is respect for the work already in memory.

### **Speed Is for the Scientist, Not the Benchmark**

Rust buys memory safety and native performance, and multi-backend GPU
acceleration buys more. None of it counts unless the result is still correct.
A number that improved because a computation stopped happening is a regression
wearing a medal.

### **The Same Program, Everywhere**

CLI, REPL, desktop GUI, chart viewer, web 3D, language server, VS Code. One
language, one standard library, one set of semantics. If a surface disagrees with
the interpreter, the surface is wrong.

### **Open by Construction**

Free software, readable source, documented behaviour. A scientist should be able
to read exactly what their `MEAN` did, and change it.

## How We Work

We describe what we measured, not what we expect. We say which surfaces we
exercised and which we did not. We prefer the language's own idiom to a pattern's
name, a pure function to a clever one, and a failing test we have watched fail to
a passing test we have not.

We leave the code a little more functional than we found it — and a little more
honest.
