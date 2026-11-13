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
- `failed` — an attempted check produced evidence that did not match.

Every entry must identify its surface, capability, evidence, and optional notes.
This is intended to become the generated source for MATLAB/Octave, IDL/GDL,
scientific-I/O, GPU, and model-runtime status reports.

The current CLI exposes a curated baseline:

```bash
xdl compatibility-report
xdl compatibility-report --format json
xdl compatibility-report --corpus examples/matlab --format json
xdl compatibility-report --corpus examples/matlab --execute --format json
```

This baseline is intentionally not generated from the full corpus yet. Its
purpose is to make current claims explicit and machine-readable while the
conformance harness is built. Corpus mode measures transpilation only unless
`--execute` is supplied. Even with `--execute`, a successful entry remains
`partial` until independent numerical equivalence tests are attached.

For a narrow observed-output reference, place a sidecar next to a corpus file:

```text
example.m
example.m.expected
```

With `--execute`, matching normalized stdout is reported as `verified` for
that observed-output case. A mismatch is reported as `failed`; this does not
prove or disprove broader array, runtime, or numerical equivalence.

## Current boundary

The first implementation is intentionally small:

- serialization is JSON;
- schema version is currently `1`;
- validation checks identity, status-entry, artifact, and schema fields;
- `xdl test --manifest PATH` emits test counts and status;
- the compatibility report is a curated baseline, not corpus-generated;
- no VibeCoder panel consumes manifests yet;
- no file hashing or clean-machine replay is implemented yet.

The basic runner can now emit a truthful summary manifest:

```bash
xdl test tests --manifest runs/test-summary.json
```

The next step is to add an explicit, reviewable collection policy for file
hashes, outputs, timeouts, numerical evidence, and clean-machine replay, then
expose those richer artifacts through VibeCoder.

## Verification

The crate has tests for:

- manifest round-trip serialization;
- invalid identity rejection;
- compatibility inventory round trips;
- fail-closed behavior for future schema versions.

Run:

```bash
cargo test -p xdl-artifacts
cargo test -p xdl-cli --test cli_tests
```
