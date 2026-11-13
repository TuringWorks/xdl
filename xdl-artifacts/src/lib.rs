// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! Versioned contracts for reproducible XDL scientific-computing artifacts.
//!
//! This crate intentionally contains data contracts and validation only. It
//! does not execute scripts, hash files, or claim that a run is reproducible;
//! callers must collect and verify those facts at the workflow boundary.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Current version of the serialized run-manifest contract.
pub const RUN_MANIFEST_SCHEMA_VERSION: u32 = 1;

/// Current version of the serialized compatibility-inventory contract.
pub const COMPATIBILITY_SCHEMA_VERSION: u32 = 1;

/// Lifecycle/result state recorded in a run manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    InProgress,
    Passed,
    Failed,
}

/// Summary of discovered script tests for a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestSummary {
    pub discovered: usize,
    pub passed: usize,
    pub failed: usize,
}

impl TestSummary {
    pub fn validate(&self) -> Result<(), ArtifactError> {
        if self.passed.saturating_add(self.failed) != self.discovered {
            return Err(ArtifactError::InvalidTestSummary {
                discovered: self.discovered,
                passed: self.passed,
                failed: self.failed,
            });
        }
        Ok(())
    }
}

/// A capability's observed implementation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityStatus {
    /// Behavior is covered by executable tests for the stated scope.
    Verified,
    /// Some behavior is implemented, but scope or semantics are incomplete.
    Partial,
    /// The path exists for experiments and requires capability-specific
    /// validation before production use.
    Experimental,
    /// The capability is provided by a configured external runtime or tool.
    Delegated,
    /// The capability is explicitly unavailable.
    Unsupported,
    /// An attempted compatibility check produced evidence that did not match.
    Failed,
}

/// One machine-readable compatibility claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompatibilityEntry {
    pub id: String,
    pub surface: String,
    pub capability: String,
    pub status: CompatibilityStatus,
    #[serde(default)]
    pub evidence: Vec<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

impl CompatibilityEntry {
    /// Validate the fields that are required for a useful status report.
    pub fn validate(&self) -> Result<(), ArtifactError> {
        require_non_empty("compatibility id", &self.id)?;
        require_non_empty("compatibility surface", &self.surface)?;
        require_non_empty("compatibility capability", &self.capability)
    }
}

/// A versioned collection of compatibility claims.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompatibilityInventory {
    pub schema_version: u32,
    pub generated_by: String,
    pub entries: Vec<CompatibilityEntry>,
}

impl CompatibilityInventory {
    pub fn new(generated_by: impl Into<String>, entries: Vec<CompatibilityEntry>) -> Self {
        Self {
            schema_version: COMPATIBILITY_SCHEMA_VERSION,
            generated_by: generated_by.into(),
            entries,
        }
    }

    pub fn validate(&self) -> Result<(), ArtifactError> {
        if self.schema_version != COMPATIBILITY_SCHEMA_VERSION {
            return Err(ArtifactError::UnsupportedSchemaVersion {
                kind: "compatibility inventory",
                version: self.schema_version,
            });
        }
        require_non_empty("inventory generator", &self.generated_by)?;
        self.entries
            .iter()
            .try_for_each(CompatibilityEntry::validate)
    }

    pub fn to_json_pretty(&self) -> Result<String, ArtifactError> {
        self.validate()?;
        serde_json::to_string_pretty(self).map_err(ArtifactError::Serialization)
    }

    pub fn from_json(json: &str) -> Result<Self, ArtifactError> {
        let inventory: Self = serde_json::from_str(json).map_err(ArtifactError::Serialization)?;
        inventory.validate()?;
        Ok(inventory)
    }
}

/// A produced or consumed artifact associated with a scientific run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunArtifact {
    pub kind: String,
    pub path: String,
    #[serde(default)]
    pub sha256: Option<String>,
}

impl RunArtifact {
    pub fn validate(&self) -> Result<(), ArtifactError> {
        require_non_empty("artifact kind", &self.kind)?;
        require_non_empty("artifact path", &self.path)
    }
}

/// The reproducibility metadata for one XDL execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunManifest {
    pub schema_version: u32,
    #[serde(default = "default_run_status")]
    pub status: RunStatus,
    pub run_id: String,
    pub project: String,
    pub entrypoint: String,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub source_hash: Option<String>,
    #[serde(default)]
    pub model_hash: Option<String>,
    #[serde(default)]
    pub input_hashes: BTreeMap<String, String>,
    #[serde(default)]
    pub configuration: BTreeMap<String, String>,
    #[serde(default)]
    pub seed: Option<u64>,
    #[serde(default)]
    pub backend: Option<String>,
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
    #[serde(default)]
    pub solver: Option<String>,
    #[serde(default)]
    pub tolerances: BTreeMap<String, f64>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub test_summary: Option<TestSummary>,
    #[serde(default)]
    pub inputs: Vec<RunArtifact>,
    #[serde(default)]
    pub outputs: Vec<RunArtifact>,
}

impl RunManifest {
    pub fn new(
        run_id: impl Into<String>,
        project: impl Into<String>,
        entrypoint: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: RUN_MANIFEST_SCHEMA_VERSION,
            status: RunStatus::InProgress,
            run_id: run_id.into(),
            project: project.into(),
            entrypoint: entrypoint.into(),
            started_at: None,
            source_hash: None,
            model_hash: None,
            input_hashes: BTreeMap::new(),
            configuration: BTreeMap::new(),
            seed: None,
            backend: None,
            environment: BTreeMap::new(),
            solver: None,
            tolerances: BTreeMap::new(),
            warnings: Vec::new(),
            test_summary: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn validate(&self) -> Result<(), ArtifactError> {
        if self.schema_version != RUN_MANIFEST_SCHEMA_VERSION {
            return Err(ArtifactError::UnsupportedSchemaVersion {
                kind: "run manifest",
                version: self.schema_version,
            });
        }
        require_non_empty("run id", &self.run_id)?;
        require_non_empty("project", &self.project)?;
        require_non_empty("entrypoint", &self.entrypoint)?;
        if let Some(summary) = self.test_summary {
            summary.validate()?;
            if self.status == RunStatus::Passed && summary.failed != 0 {
                return Err(ArtifactError::InvalidStatusForTestSummary);
            }
            if self.status == RunStatus::Failed && summary.failed == 0 {
                return Err(ArtifactError::InvalidStatusForTestSummary);
            }
        }
        self.inputs
            .iter()
            .chain(self.outputs.iter())
            .try_for_each(RunArtifact::validate)
    }

    pub fn to_json_pretty(&self) -> Result<String, ArtifactError> {
        self.validate()?;
        serde_json::to_string_pretty(self).map_err(ArtifactError::Serialization)
    }

    pub fn from_json(json: &str) -> Result<Self, ArtifactError> {
        let manifest: Self = serde_json::from_str(json).map_err(ArtifactError::Serialization)?;
        manifest.validate()?;
        Ok(manifest)
    }
}

#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("{field} must not be empty")]
    EmptyField { field: &'static str },
    #[error("unsupported {kind} schema version: {version}")]
    UnsupportedSchemaVersion { kind: &'static str, version: u32 },
    #[error("invalid test summary: discovered={discovered}, passed={passed}, failed={failed}")]
    InvalidTestSummary {
        discovered: usize,
        passed: usize,
        failed: usize,
    },
    #[error("run status does not agree with its test summary")]
    InvalidStatusForTestSummary,
    #[error("artifact serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), ArtifactError> {
    if value.trim().is_empty() {
        Err(ArtifactError::EmptyField { field })
    } else {
        Ok(())
    }
}

const fn default_run_status() -> RunStatus {
    RunStatus::InProgress
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_manifest_round_trips_and_keeps_ordered_metadata() {
        let mut manifest = RunManifest::new("run-001", "gnc-demo", "examples/demo.xdl");
        manifest.seed = Some(42);
        manifest.configuration.insert("mode".into(), "sil".into());
        manifest.tolerances.insert("position_m".into(), 1e-6);
        manifest.outputs.push(RunArtifact {
            kind: "plot".into(),
            path: "runs/run-001/trajectory.png".into(),
            sha256: Some("abc123".into()),
        });

        let json = manifest
            .to_json_pretty()
            .expect("valid manifest serializes");
        let decoded = RunManifest::from_json(&json).expect("manifest round trips");

        assert_eq!(decoded, manifest);
        assert!(json.contains("\"schema_version\": 1"));
        assert!(json.find("\"configuration\"").is_some());
    }

    #[test]
    fn invalid_manifest_is_rejected_before_serialization() {
        let manifest = RunManifest::new("", "project", "entrypoint.xdl");

        assert!(matches!(
            manifest.to_json_pretty(),
            Err(ArtifactError::EmptyField { field: "run id" })
        ));
    }

    #[test]
    fn compatibility_inventory_requires_explicit_status_entries() {
        let inventory = CompatibilityInventory::new(
            "xdl-compat-report",
            vec![CompatibilityEntry {
                id: "matlab.array-literal".into(),
                surface: "matlab".into(),
                capability: "array literals".into(),
                status: CompatibilityStatus::Partial,
                evidence: vec!["xdl-matlab::tests::test_array_literal".into()],
                notes: Some("multidimensional edge cases remain".into()),
            }],
        );

        let decoded = CompatibilityInventory::from_json(
            &inventory
                .to_json_pretty()
                .expect("valid inventory serializes"),
        )
        .expect("inventory round trips");

        assert_eq!(decoded, inventory);
    }

    #[test]
    fn future_schema_versions_fail_closed() {
        let mut manifest = RunManifest::new("run-001", "project", "entrypoint.xdl");
        manifest.schema_version += 1;

        assert!(matches!(
            manifest.validate(),
            Err(ArtifactError::UnsupportedSchemaVersion {
                kind: "run manifest",
                ..
            })
        ));
    }

    #[test]
    fn test_summary_must_match_run_status_and_counts() {
        let mut manifest = RunManifest::new("run-001", "project", "tests");
        manifest.status = RunStatus::Passed;
        manifest.test_summary = Some(TestSummary {
            discovered: 2,
            passed: 1,
            failed: 1,
        });

        assert!(matches!(
            manifest.validate(),
            Err(ArtifactError::InvalidStatusForTestSummary)
        ));
    }
}
