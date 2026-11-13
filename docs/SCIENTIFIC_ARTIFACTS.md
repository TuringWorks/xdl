# Scientific artifacts and compatibility contracts

`xdl-artifacts` is the first P0 foundation for reproducible scientific
workflows. It defines data contracts; it does not yet execute tests, hash files,
collect plots, or guarantee reproducibility by itself.

## Run manifests

A `RunManifest` records the metadata needed to reproduce and review one run:

- versioned schema and run ID;
- project and entrypoint;
- optional source and model hashes;
- input hashes and named configuration;
- random seed, backend, environment, and solver;
- declared numerical tolerances;
- warnings and input/output artifact references.

The contract validates required identity fields and rejects unknown future schema
versions. Callers remain responsible for calculating hashes, capturing the
environment, and verifying that referenced files exist.

Example shape:

```json
{
  "schema_version": 1,
  "run_id": "run-001",
  "project": "gnc-demo",
  "entrypoint": "examples/demo.xdl",
  "seed": 42,
  "backend": "cpu",
  "solver": "rk4",
  "tolerances": { "position_m": 0.000001 },
  "warnings": [],
  "inputs": [],
  "outputs": []
}
```

## Compatibility inventory

`CompatibilityInventory` provides explicit statuses instead of aggregate claims:

- `verified` — executable tests cover the stated scope;
- `partial` — behavior exists but scope or semantics are incomplete;
- `experimental` — useful for evaluation, not yet generally validated;
- `delegated` — provided by a configured external runtime/tool;
- `unsupported` — explicitly unavailable.

Every entry must identify its surface, capability, evidence, and optional notes.
This is intended to become the generated source for MATLAB/Octave, IDL/GDL,
scientific-I/O, GPU, and model-runtime status reports.

## Current boundary

The first implementation is intentionally small:

- serialization is JSON;
- schema version is currently `1`;
- validation checks identity, status-entry, artifact, and schema fields;
- no CLI command consumes manifests yet;
- no VibeCoder panel consumes manifests yet;
- no file hashing or clean-machine replay is implemented yet.

The next step is to make `xdl test` produce these manifests from an explicit,
reviewable collection policy, then expose them through VibeCoder artifacts.

## Verification

The crate has tests for:

- manifest round-trip serialization;
- invalid identity rejection;
- compatibility inventory round trips;
- fail-closed behavior for future schema versions.

Run:

```bash
cargo test -p xdl-artifacts
```
