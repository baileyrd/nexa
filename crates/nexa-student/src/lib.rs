//! Governed, deterministic learner evidence and derived mastery projections.
#![forbid(unsafe_code)]

use nexa_domain::{
    AssessmentId, AttemptId, CompetencyId, Confidence, EvidenceId, LearningObjectiveId,
    MasteryScore, ProtocolVersion, StudentId, Timestamp,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq)]
pub enum StudentModelError {
    #[error("{field} must contain between 1 and {max} characters")]
    InvalidText { field: &'static str, max: usize },
    #[error("competency must reference at least one objective")]
    MissingObjective,
    #[error("attempt completion precedes its start")]
    InvalidAttemptTime,
    #[error("evidence {0} already exists with different content")]
    ConflictingDuplicate(EvidenceId),
    #[error("evidence belongs to another student or competency")]
    ProjectionMismatch,
    #[error("repository operation failed: {message}")]
    Repository { message: String },
}

fn valid_text(value: &str, field: &'static str, max: usize) -> Result<(), StudentModelError> {
    if value.trim().is_empty() || value.chars().count() > max {
        Err(StudentModelError::InvalidText { field, max })
    } else {
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "StudentWire")]
pub struct Student {
    pub id: StudentId,
    pub display_name: String,
    pub created_at: Timestamp,
}
#[derive(Deserialize)]
struct StudentWire {
    id: StudentId,
    display_name: String,
    created_at: Timestamp,
}
impl TryFrom<StudentWire> for Student {
    type Error = StudentModelError;
    fn try_from(v: StudentWire) -> Result<Self, Self::Error> {
        valid_text(&v.display_name, "display_name", 200)?;
        Ok(Self {
            id: v.id,
            display_name: v.display_name,
            created_at: v.created_at,
        })
    }
}
impl Student {
    pub fn new(
        id: StudentId,
        display_name: impl Into<String>,
        created_at: Timestamp,
    ) -> Result<Self, StudentModelError> {
        Self::try_from(StudentWire {
            id,
            display_name: display_name.into(),
            created_at,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "ObjectiveWire")]
pub struct LearningObjective {
    pub id: LearningObjectiveId,
    pub title: String,
}
#[derive(Deserialize)]
struct ObjectiveWire {
    id: LearningObjectiveId,
    title: String,
}
impl TryFrom<ObjectiveWire> for LearningObjective {
    type Error = StudentModelError;
    fn try_from(v: ObjectiveWire) -> Result<Self, Self::Error> {
        valid_text(&v.title, "objective.title", 300)?;
        Ok(Self {
            id: v.id,
            title: v.title,
        })
    }
}
impl LearningObjective {
    pub fn new(
        id: LearningObjectiveId,
        title: impl Into<String>,
    ) -> Result<Self, StudentModelError> {
        Self::try_from(ObjectiveWire {
            id,
            title: title.into(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "CompetencyWire")]
pub struct Competency {
    pub id: CompetencyId,
    pub title: String,
    pub objective_ids: Vec<LearningObjectiveId>,
}
#[derive(Deserialize)]
struct CompetencyWire {
    id: CompetencyId,
    title: String,
    objective_ids: Vec<LearningObjectiveId>,
}
impl TryFrom<CompetencyWire> for Competency {
    type Error = StudentModelError;
    fn try_from(mut v: CompetencyWire) -> Result<Self, Self::Error> {
        valid_text(&v.title, "competency.title", 300)?;
        v.objective_ids.sort();
        v.objective_ids.dedup();
        if v.objective_ids.is_empty() {
            return Err(StudentModelError::MissingObjective);
        }
        Ok(Self {
            id: v.id,
            title: v.title,
            objective_ids: v.objective_ids,
        })
    }
}
impl Competency {
    pub fn new(
        id: CompetencyId,
        title: impl Into<String>,
        objective_ids: Vec<LearningObjectiveId>,
    ) -> Result<Self, StudentModelError> {
        Self::try_from(CompetencyWire {
            id,
            title: title.into(),
            objective_ids,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    Recognition,
    Recall,
    Explanation,
    Application,
    Demonstration,
    Debugging,
    Transfer,
    Retention,
    LabPerformance,
    Assessment,
    InstructorObservation,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceOutcome {
    Success,
    PartialSuccess,
    Failure,
    Ambiguous,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceDifficulty {
    VeryEasy,
    Easy,
    Moderate,
    Challenging,
    Advanced,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndependenceLevel {
    Independent,
    MinorHint,
    ModerateHint,
    HeavyGuidance,
    SolutionExposed,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "id")]
pub enum EvidenceSource {
    Assessment(AttemptId),
    Lesson(LearningObjectiveId),
    Instructor,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LearningEvidence {
    pub id: EvidenceId,
    pub student_id: StudentId,
    pub competency_id: CompetencyId,
    pub evidence_type: EvidenceType,
    pub outcome: EvidenceOutcome,
    pub difficulty: EvidenceDifficulty,
    pub independence: IndependenceLevel,
    pub confidence: Option<Confidence>,
    pub source: EvidenceSource,
    pub observed_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "AttemptWire")]
pub struct AssessmentAttempt {
    pub id: AttemptId,
    pub assessment_id: AssessmentId,
    pub student_id: StudentId,
    pub started_at: Timestamp,
    pub completed_at: Option<Timestamp>,
}
#[derive(Deserialize)]
struct AttemptWire {
    id: AttemptId,
    assessment_id: AssessmentId,
    student_id: StudentId,
    started_at: Timestamp,
    completed_at: Option<Timestamp>,
}
impl TryFrom<AttemptWire> for AssessmentAttempt {
    type Error = StudentModelError;
    fn try_from(v: AttemptWire) -> Result<Self, Self::Error> {
        if v.completed_at.is_some_and(|t| t < v.started_at) {
            return Err(StudentModelError::InvalidAttemptTime);
        }
        Ok(Self {
            id: v.id,
            assessment_id: v.assessment_id,
            student_id: v.student_id,
            started_at: v.started_at,
            completed_at: v.completed_at,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompetencyStatus {
    Unestablished,
    Emerging,
    Developing,
    Functional,
    Proficient,
    Mastered,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MasteryState {
    pub student_id: StudentId,
    pub competency_id: CompetencyId,
    pub mastery: MasteryScore,
    pub model_confidence: Confidence,
    pub status: CompetencyStatus,
    pub evidence_count: u32,
    pub last_evidence_at: Option<Timestamp>,
    pub policy_version: ProtocolVersion,
}
impl MasteryState {
    pub fn empty(
        student_id: StudentId,
        competency_id: CompetencyId,
        policy_version: ProtocolVersion,
    ) -> Self {
        Self {
            student_id,
            competency_id,
            mastery: MasteryScore::new(0.0).expect("constant valid"),
            model_confidence: Confidence::new(0.0).expect("constant valid"),
            status: CompetencyStatus::Unestablished,
            evidence_count: 0,
            last_evidence_at: None,
            policy_version,
        }
    }
}

pub trait MasteryUpdatePolicy {
    fn version(&self) -> ProtocolVersion;
    fn update(
        &self,
        previous: &MasteryState,
        evidence: &LearningEvidence,
    ) -> Result<MasteryState, StudentModelError>;
}
/// Phase 3 v1 bounded weighted update. Coefficients are part of version 1.0.
#[derive(Clone, Copy, Debug, Default)]
pub struct BoundedWeightedV1;
impl MasteryUpdatePolicy for BoundedWeightedV1 {
    fn version(&self) -> ProtocolVersion {
        ProtocolVersion::new(1, 0)
    }
    fn update(
        &self,
        p: &MasteryState,
        e: &LearningEvidence,
    ) -> Result<MasteryState, StudentModelError> {
        if p.student_id != e.student_id || p.competency_id != e.competency_id {
            return Err(StudentModelError::ProjectionMismatch);
        }
        let observed = match e.outcome {
            EvidenceOutcome::Success => 1.0,
            EvidenceOutcome::PartialSuccess => 0.5,
            EvidenceOutcome::Failure => 0.0,
            EvidenceOutcome::Ambiguous => p.mastery.get(),
        };
        let difficulty = match e.difficulty {
            EvidenceDifficulty::VeryEasy => 0.6,
            EvidenceDifficulty::Easy => 0.8,
            EvidenceDifficulty::Moderate => 1.0,
            EvidenceDifficulty::Challenging => 1.1,
            EvidenceDifficulty::Advanced => 1.2,
        };
        let independence = match e.independence {
            IndependenceLevel::Independent => 1.0,
            IndependenceLevel::MinorHint => 0.85,
            IndependenceLevel::ModerateHint => 0.65,
            IndependenceLevel::HeavyGuidance => 0.35,
            IndependenceLevel::SolutionExposed => 0.1,
        };
        let next = (p.mastery.get()
            + 0.25 * difficulty * independence * (observed - p.mastery.get()))
        .clamp(0.0, 1.0);
        let count = p.evidence_count.saturating_add(1);
        let confidence = (count as f64 / 5.0).min(1.0);
        let status = match next {
            v if count >= 5 && v >= 0.85 => CompetencyStatus::Mastered,
            v if v >= 0.7 => CompetencyStatus::Proficient,
            v if v >= 0.5 => CompetencyStatus::Functional,
            v if v >= 0.3 => CompetencyStatus::Developing,
            v if v > 0.0 => CompetencyStatus::Emerging,
            _ => CompetencyStatus::Unestablished,
        };
        Ok(MasteryState {
            student_id: p.student_id,
            competency_id: p.competency_id,
            mastery: MasteryScore::new(next).map_err(|x| StudentModelError::Repository {
                message: x.to_string(),
            })?,
            model_confidence: Confidence::new(confidence).map_err(|x| {
                StudentModelError::Repository {
                    message: x.to_string(),
                }
            })?,
            status,
            evidence_count: count,
            last_evidence_at: Some(e.observed_at),
            policy_version: self.version(),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppendOutcome {
    Appended,
    Duplicate,
}
pub trait EvidenceRepository {
    fn append(&mut self, evidence: LearningEvidence) -> Result<AppendOutcome, StudentModelError>;
    fn all(&self) -> Result<Vec<LearningEvidence>, StudentModelError>;
}
pub trait MasteryRepository {
    fn put(&mut self, state: MasteryState) -> Result<(), StudentModelError>;
    fn get(
        &self,
        student: StudentId,
        competency: CompetencyId,
    ) -> Result<Option<MasteryState>, StudentModelError>;
}

#[derive(Clone, Debug, Default)]
pub struct InMemoryEvidenceRepository {
    entries: Vec<LearningEvidence>,
    ids: BTreeMap<EvidenceId, usize>,
}
impl EvidenceRepository for InMemoryEvidenceRepository {
    fn append(&mut self, e: LearningEvidence) -> Result<AppendOutcome, StudentModelError> {
        if let Some(i) = self.ids.get(&e.id) {
            return if self.entries[*i] == e {
                Ok(AppendOutcome::Duplicate)
            } else {
                Err(StudentModelError::ConflictingDuplicate(e.id))
            };
        }
        self.ids.insert(e.id, self.entries.len());
        self.entries.push(e);
        Ok(AppendOutcome::Appended)
    }
    fn all(&self) -> Result<Vec<LearningEvidence>, StudentModelError> {
        Ok(self.entries.clone())
    }
}
#[derive(Clone, Debug, Default)]
pub struct InMemoryMasteryRepository(BTreeMap<(StudentId, CompetencyId), MasteryState>);
impl MasteryRepository for InMemoryMasteryRepository {
    fn put(&mut self, s: MasteryState) -> Result<(), StudentModelError> {
        self.0.insert((s.student_id, s.competency_id), s);
        Ok(())
    }
    fn get(
        &self,
        a: StudentId,
        b: CompetencyId,
    ) -> Result<Option<MasteryState>, StudentModelError> {
        Ok(self.0.get(&(a, b)).cloned())
    }
}

pub fn replay<P: MasteryUpdatePolicy>(
    evidence: &[LearningEvidence],
    policy: &P,
) -> Result<Vec<MasteryState>, StudentModelError> {
    let mut ordered = evidence.to_vec();
    ordered.sort_by_key(|e| (e.observed_at, e.id));
    let mut seen = BTreeSet::new();
    let mut states: BTreeMap<(StudentId, CompetencyId), MasteryState> = BTreeMap::new();
    for e in ordered {
        if !seen.insert(e.id) {
            continue;
        }
        let key = (e.student_id, e.competency_id);
        let previous = states.remove(&key).unwrap_or_else(|| {
            MasteryState::empty(e.student_id, e.competency_id, policy.version())
        });
        states.insert(key, policy.update(&previous, &e)?);
    }
    Ok(states.into_values().collect())
}

pub use nexa_events::{CompetencyEvidenceAdded, CompetencyUpdated};
