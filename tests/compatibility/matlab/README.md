# MATLAB compatibility corpus

This is a deliberately small checked-in corpus for the compatibility-report
workflow. It is evidence for the specific cases here, not a language or
toolbox coverage claim.

Run it with:

```bash
xdl compatibility-report \
  --corpus tests/compatibility/matlab \
  --execute \
  --format json
```

The sidecars demonstrate two narrow reference types:

- `scalar_output.m.expected` checks normalized observed stdout;
- `vector_values.m.expected.json` checks named numeric values with an absolute
  tolerance and records their provenance.

`no_reference.m` is intentionally expected to remain `partial`.
