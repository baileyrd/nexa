//! Governed authored curriculum and a pure, headless lesson transition policy.
#![forbid(unsafe_code)]

use nexa_domain::{
    CompetencyId, CourseId, CurriculumId, LearningObjectiveId, LessonId, LessonStepId, ModuleId,
    ProtocolVersion, StudentId, Timestamp,
};
use nexa_pedagogy::{InstructionalOption, PedagogyDecision};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const LESSON_POLICY_V1: ProtocolVersion = ProtocolVersion::new(1, 0);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LessonStepKind {
    Introduction,
    Explanation,
    Demonstration,
    Example,
    Practice,
    Question,
    Reflection,
    Review,
    Remediation,
    Challenge,
    Lab,
    Assessment,
    Summary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepRequirement {
    Optional,
    Recommended,
    Required,
    MandatoryForCertification,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ObjectiveMapping {
    pub objective_id: LearningObjectiveId,
    pub competency_ids: BTreeSet<CompetencyId>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LessonStep {
    pub id: LessonStepId,
    pub kind: LessonStepKind,
    pub requirement: StepRequirement,
    pub objective_ids: BTreeSet<LearningObjectiveId>,
    /// Governed routing destinations. An absent option is unavailable, never inferred.
    pub routes: BTreeMap<InstructionalOption, LessonStepId>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Lesson {
    pub id: LessonId,
    pub objective_ids: BTreeSet<LearningObjectiveId>,
    pub prerequisites: BTreeSet<LessonId>,
    pub steps: Vec<LessonStep>,
    pub entry_step_id: LessonStepId,
    pub completion_step_id: LessonStepId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub id: ModuleId,
    pub lesson_ids: Vec<LessonId>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Course {
    pub id: CourseId,
    pub module_ids: Vec<ModuleId>,
}

/// Immutable after construction: all fields are private and no mutation API is exposed.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "CurriculumWire")]
pub struct Curriculum {
    id: CurriculumId,
    courses: Vec<Course>,
    modules: Vec<Module>,
    lessons: Vec<Lesson>,
    objective_mappings: Vec<ObjectiveMapping>,
}

#[derive(Clone, Debug, Deserialize)]
struct CurriculumWire {
    id: CurriculumId,
    courses: Vec<Course>,
    modules: Vec<Module>,
    lessons: Vec<Lesson>,
    objective_mappings: Vec<ObjectiveMapping>,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum CurriculumError {
    #[error("{collection} must not be empty")]
    Empty { collection: &'static str },
    #[error("duplicate {kind} identifier {id}")]
    Duplicate { kind: &'static str, id: String },
    #[error("dangling {kind} reference {id}")]
    Dangling { kind: &'static str, id: String },
    #[error("lesson {lesson_id} has a self prerequisite")]
    SelfPrerequisite { lesson_id: LessonId },
    #[error("lesson prerequisite graph contains a cycle")]
    PrerequisiteCycle,
    #[error("lesson {lesson_id} has invalid entry or completion step {step_id}")]
    InvalidBoundary {
        lesson_id: LessonId,
        step_id: LessonStepId,
    },
    #[error("lesson {lesson_id} step ordering is invalid at {step_id}")]
    InvalidStepOrdering {
        lesson_id: LessonId,
        step_id: LessonStepId,
    },
    #[error("objective {objective_id} has no competency mapping")]
    MissingCompetencyMapping { objective_id: LearningObjectiveId },
    #[error("step {step_id} has no objective mapping")]
    MissingObjectiveMapping { step_id: LessonStepId },
}

impl TryFrom<CurriculumWire> for Curriculum {
    type Error = CurriculumError;
    fn try_from(w: CurriculumWire) -> Result<Self, Self::Error> {
        if w.courses.is_empty() {
            return Err(CurriculumError::Empty {
                collection: "courses",
            });
        }
        if w.modules.is_empty() {
            return Err(CurriculumError::Empty {
                collection: "modules",
            });
        }
        if w.lessons.is_empty() {
            return Err(CurriculumError::Empty {
                collection: "lessons",
            });
        }
        unique(w.courses.iter().map(|x| x.id.to_string()), "course")?;
        unique(w.modules.iter().map(|x| x.id.to_string()), "module")?;
        unique(w.lessons.iter().map(|x| x.id.to_string()), "lesson")?;
        let course_modules: BTreeSet<_> = w.modules.iter().map(|x| x.id).collect();
        let all_lessons: BTreeSet<_> = w.lessons.iter().map(|x| x.id).collect();
        for c in &w.courses {
            if c.module_ids.is_empty() {
                return Err(CurriculumError::Empty {
                    collection: "course modules",
                });
            }
            unique(
                c.module_ids.iter().map(ToString::to_string),
                "course module",
            )?;
            for id in &c.module_ids {
                if !course_modules.contains(id) {
                    return Err(dangling("module", id));
                }
            }
        }
        for m in &w.modules {
            if m.lesson_ids.is_empty() {
                return Err(CurriculumError::Empty {
                    collection: "module lessons",
                });
            }
            unique(
                m.lesson_ids.iter().map(ToString::to_string),
                "module lesson",
            )?;
            for id in &m.lesson_ids {
                if !all_lessons.contains(id) {
                    return Err(dangling("lesson", id));
                }
            }
        }
        let mut mappings = BTreeMap::new();
        for mapping in &w.objective_mappings {
            if mapping.competency_ids.is_empty() {
                return Err(CurriculumError::MissingCompetencyMapping {
                    objective_id: mapping.objective_id,
                });
            }
            if mappings
                .insert(mapping.objective_id, &mapping.competency_ids)
                .is_some()
            {
                return Err(CurriculumError::Duplicate {
                    kind: "objective mapping",
                    id: mapping.objective_id.to_string(),
                });
            }
        }
        let mut global_steps = BTreeSet::new();
        for lesson in &w.lessons {
            if lesson.steps.is_empty() {
                return Err(CurriculumError::Empty {
                    collection: "lesson steps",
                });
            }
            if lesson.prerequisites.contains(&lesson.id) {
                return Err(CurriculumError::SelfPrerequisite {
                    lesson_id: lesson.id,
                });
            }
            for p in &lesson.prerequisites {
                if !all_lessons.contains(p) {
                    return Err(dangling("prerequisite lesson", p));
                }
            }
            let step_ids: BTreeSet<_> = lesson.steps.iter().map(|s| s.id).collect();
            if step_ids.len() != lesson.steps.len() {
                return Err(CurriculumError::Duplicate {
                    kind: "lesson step",
                    id: lesson.id.to_string(),
                });
            }
            for step in &lesson.steps {
                if !global_steps.insert(step.id) {
                    return Err(CurriculumError::Duplicate {
                        kind: "lesson step",
                        id: step.id.to_string(),
                    });
                }
                if step.objective_ids.is_empty() {
                    return Err(CurriculumError::MissingObjectiveMapping { step_id: step.id });
                }
                for o in &step.objective_ids {
                    if !lesson.objective_ids.contains(o) || !mappings.contains_key(o) {
                        return Err(CurriculumError::MissingObjectiveMapping { step_id: step.id });
                    }
                }
                for target in step.routes.values() {
                    if !step_ids.contains(target) {
                        return Err(dangling("route step", target));
                    }
                }
            }
            for o in &lesson.objective_ids {
                if !mappings.contains_key(o) {
                    return Err(CurriculumError::MissingCompetencyMapping { objective_id: *o });
                }
            }
            if !step_ids.contains(&lesson.entry_step_id) {
                return Err(CurriculumError::InvalidBoundary {
                    lesson_id: lesson.id,
                    step_id: lesson.entry_step_id,
                });
            }
            if !step_ids.contains(&lesson.completion_step_id) {
                return Err(CurriculumError::InvalidBoundary {
                    lesson_id: lesson.id,
                    step_id: lesson.completion_step_id,
                });
            }
            if lesson.steps.first().map(|s| s.id) != Some(lesson.entry_step_id)
                || lesson.steps.last().map(|s| s.id) != Some(lesson.completion_step_id)
            {
                return Err(CurriculumError::InvalidStepOrdering {
                    lesson_id: lesson.id,
                    step_id: lesson.entry_step_id,
                });
            }
        }
        validate_acyclic(&w.lessons)?;
        Ok(Self {
            id: w.id,
            courses: w.courses,
            modules: w.modules,
            lessons: w.lessons,
            objective_mappings: w.objective_mappings,
        })
    }
}

fn unique(values: impl Iterator<Item = String>, kind: &'static str) -> Result<(), CurriculumError> {
    let mut seen = BTreeSet::new();
    for id in values {
        if !seen.insert(id.clone()) {
            return Err(CurriculumError::Duplicate { kind, id });
        }
    }
    Ok(())
}
fn dangling(kind: &'static str, id: &impl ToString) -> CurriculumError {
    CurriculumError::Dangling {
        kind,
        id: id.to_string(),
    }
}
fn validate_acyclic(lessons: &[Lesson]) -> Result<(), CurriculumError> {
    let mut indegree: BTreeMap<_, usize> = lessons
        .iter()
        .map(|l| (l.id, l.prerequisites.len()))
        .collect();
    let mut ready: BTreeSet<_> = indegree
        .iter()
        .filter(|(_, d)| **d == 0)
        .map(|(id, _)| *id)
        .collect();
    let mut visited = 0;
    while let Some(id) = ready.pop_first() {
        visited += 1;
        for l in lessons.iter().filter(|l| l.prerequisites.contains(&id)) {
            let d = indegree.get_mut(&l.id).expect("validated lesson index");
            *d -= 1;
            if *d == 0 {
                ready.insert(l.id);
            }
        }
    }
    if visited == lessons.len() {
        Ok(())
    } else {
        Err(CurriculumError::PrerequisiteCycle)
    }
}

impl Curriculum {
    pub fn new(
        id: CurriculumId,
        courses: Vec<Course>,
        modules: Vec<Module>,
        lessons: Vec<Lesson>,
        objective_mappings: Vec<ObjectiveMapping>,
    ) -> Result<Self, CurriculumError> {
        Self::try_from(CurriculumWire {
            id,
            courses,
            modules,
            lessons,
            objective_mappings,
        })
    }
    pub fn id(&self) -> CurriculumId {
        self.id
    }
    pub fn lesson(&self, id: LessonId) -> Option<&Lesson> {
        self.lessons.iter().find(|l| l.id == id)
    }
    pub fn prerequisite_order(&self) -> Vec<LessonId> {
        let mut remaining: BTreeSet<_> = self.lessons.iter().map(|l| l.id).collect();
        let mut done = BTreeSet::new();
        let mut out = Vec::new();
        while !remaining.is_empty() {
            let id = *remaining
                .iter()
                .find(|id| {
                    self.lesson(**id)
                        .is_some_and(|l| l.prerequisites.is_subset(&done))
                })
                .expect("validated acyclic graph");
            remaining.remove(&id);
            done.insert(id);
            out.push(id);
        }
        out
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LessonLifecycle {
    NotStarted,
    Active,
    Waiting,
    Completed,
    Blocked,
    Abandoned,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "ProgressWire")]
pub struct LessonProgress {
    policy_version: ProtocolVersion,
    student_id: StudentId,
    lesson_id: LessonId,
    lifecycle: LessonLifecycle,
    current_step_id: Option<LessonStepId>,
    completed_steps: BTreeSet<LessonStepId>,
    started_at: Option<Timestamp>,
    updated_at: Option<Timestamp>,
    completed_at: Option<Timestamp>,
    rationale: Vec<String>,
}
#[derive(Clone, Debug, Deserialize)]
struct ProgressWire {
    policy_version: ProtocolVersion,
    student_id: StudentId,
    lesson_id: LessonId,
    lifecycle: LessonLifecycle,
    current_step_id: Option<LessonStepId>,
    completed_steps: BTreeSet<LessonStepId>,
    started_at: Option<Timestamp>,
    updated_at: Option<Timestamp>,
    completed_at: Option<Timestamp>,
    rationale: Vec<String>,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum TransitionError {
    #[error("lesson policy version mismatch: expected {expected}, got {actual}")]
    PolicyVersionMismatch {
        expected: ProtocolVersion,
        actual: ProtocolVersion,
    },
    #[error("invalid external lesson state: {message}")]
    InvalidState { message: &'static str },
    #[error("lesson route {option:?} is unavailable from step {step_id}")]
    RouteUnavailable {
        option: InstructionalOption,
        step_id: LessonStepId,
    },
    #[error("lesson route {option:?} is incompatible with step {step_id}")]
    IncompatibleRoute {
        option: InstructionalOption,
        step_id: LessonStepId,
    },
    #[error("lesson prerequisites are unmet: {lesson_ids:?}")]
    UnmetPrerequisites { lesson_ids: Vec<LessonId> },
    #[error("completed, blocked, or abandoned lesson is terminal")]
    TerminalState,
    #[error("pedagogy decision targets an unrelated competency")]
    PedagogyCompetencyMismatch,
    #[error("pedagogy decision targets a different student")]
    PedagogyStudentMismatch,
    #[error("pedagogy decision policy version is incompatible: expected {expected}, got {actual}")]
    PedagogyPolicyVersionMismatch {
        expected: ProtocolVersion,
        actual: ProtocolVersion,
    },
    #[error("lesson transition timestamp regressed")]
    TimestampRegression,
}
impl TryFrom<ProgressWire> for LessonProgress {
    type Error = TransitionError;
    fn try_from(w: ProgressWire) -> Result<Self, Self::Error> {
        if w.rationale.is_empty() || w.rationale.iter().any(|r| r.trim().is_empty()) {
            return Err(TransitionError::InvalidState {
                message: "transition rationale must be nonempty",
            });
        }
        let has_cursor = w.current_step_id.is_some();
        let has_completed_steps = !w.completed_steps.is_empty();
        let timestamps_are_monotonic = match (w.started_at, w.updated_at) {
            (Some(started), Some(updated)) => started <= updated,
            (None, None) => true,
            _ => false,
        } && w
            .completed_at
            .is_none_or(|completed| w.updated_at == Some(completed));
        let valid = match w.lifecycle {
            LessonLifecycle::NotStarted => {
                !has_cursor
                    && !has_completed_steps
                    && w.started_at.is_none()
                    && w.updated_at.is_none()
                    && w.completed_at.is_none()
            }
            LessonLifecycle::Active | LessonLifecycle::Waiting => {
                has_cursor
                    && !w
                        .completed_steps
                        .contains(&w.current_step_id.expect("cursor is present"))
                    && w.started_at.is_some()
                    && w.updated_at.is_some()
                    && w.completed_at.is_none()
            }
            LessonLifecycle::Completed => {
                !has_cursor
                    && has_completed_steps
                    && w.started_at.is_some()
                    && w.updated_at.is_some()
                    && w.completed_at.is_some()
            }
            LessonLifecycle::Blocked | LessonLifecycle::Abandoned => {
                !has_cursor
                    && w.started_at.is_some()
                    && w.updated_at.is_some()
                    && w.completed_at.is_none()
            }
        };
        if !valid || !timestamps_are_monotonic {
            return Err(TransitionError::InvalidState {
                message: "accumulated state or timestamps contradict lifecycle",
            });
        }
        Ok(Self {
            policy_version: w.policy_version,
            student_id: w.student_id,
            lesson_id: w.lesson_id,
            lifecycle: w.lifecycle,
            current_step_id: w.current_step_id,
            completed_steps: w.completed_steps,
            started_at: w.started_at,
            updated_at: w.updated_at,
            completed_at: w.completed_at,
            rationale: w.rationale,
        })
    }
}
impl LessonProgress {
    pub fn not_started(student_id: StudentId, lesson_id: LessonId) -> Self {
        Self {
            policy_version: LESSON_POLICY_V1,
            student_id,
            lesson_id,
            lifecycle: LessonLifecycle::NotStarted,
            current_step_id: None,
            completed_steps: BTreeSet::new(),
            started_at: None,
            updated_at: None,
            completed_at: None,
            rationale: vec!["lesson.created".into()],
        }
    }
    pub fn lifecycle(&self) -> LessonLifecycle {
        self.lifecycle
    }
    pub fn student_id(&self) -> StudentId {
        self.student_id
    }
    pub fn lesson_id(&self) -> LessonId {
        self.lesson_id
    }
    pub fn policy_version(&self) -> ProtocolVersion {
        self.policy_version
    }
    pub fn updated_at(&self) -> Option<Timestamp> {
        self.updated_at
    }
    pub fn current_step_id(&self) -> Option<LessonStepId> {
        self.current_step_id
    }
    pub fn rationale(&self) -> &[String] {
        &self.rationale
    }
}

pub struct LessonPolicyV1;
impl LessonPolicyV1 {
    fn validate<'a>(
        curriculum: &'a Curriculum,
        p: &LessonProgress,
    ) -> Result<&'a Lesson, TransitionError> {
        if p.policy_version != LESSON_POLICY_V1 {
            return Err(TransitionError::PolicyVersionMismatch {
                expected: LESSON_POLICY_V1,
                actual: p.policy_version,
            });
        }
        let l = curriculum
            .lesson(p.lesson_id)
            .ok_or(TransitionError::InvalidState {
                message: "lesson_id is dangling",
            })?;
        let ids: BTreeSet<_> = l.steps.iter().map(|s| s.id).collect();
        if p.current_step_id.is_some_and(|id| !ids.contains(&id))
            || !p.completed_steps.is_subset(&ids)
        {
            return Err(TransitionError::InvalidState {
                message: "cursor or completed step is dangling",
            });
        }
        Ok(l)
    }
    fn validate_timestamp(p: &LessonProgress, at: Timestamp) -> Result<(), TransitionError> {
        if p.updated_at
            .or(p.started_at)
            .is_some_and(|prior| at < prior)
        {
            return Err(TransitionError::TimestampRegression);
        }
        Ok(())
    }
    pub fn start(
        curriculum: &Curriculum,
        p: &LessonProgress,
        completed_lessons: &BTreeSet<LessonId>,
        at: Timestamp,
    ) -> Result<LessonProgress, TransitionError> {
        let l = Self::validate(curriculum, p)?;
        Self::validate_timestamp(p, at)?;
        if p.lifecycle != LessonLifecycle::NotStarted {
            return Err(TransitionError::TerminalState);
        }
        let unmet: Vec<_> = l
            .prerequisites
            .difference(completed_lessons)
            .copied()
            .collect();
        if !unmet.is_empty() {
            return Err(TransitionError::UnmetPrerequisites { lesson_ids: unmet });
        }
        let mut n = p.clone();
        n.lifecycle = LessonLifecycle::Active;
        n.current_step_id = Some(l.entry_step_id);
        n.started_at = Some(at);
        n.updated_at = Some(at);
        n.rationale = vec!["lesson.started".into(), "prerequisites.satisfied".into()];
        Ok(n)
    }
    pub fn route(
        curriculum: &Curriculum,
        p: &LessonProgress,
        decision: &PedagogyDecision,
        completed_lessons: &BTreeSet<LessonId>,
        at: Timestamp,
    ) -> Result<LessonProgress, TransitionError> {
        let l = Self::validate(curriculum, p)?;
        Self::validate_timestamp(p, at)?;
        if matches!(
            p.lifecycle,
            LessonLifecycle::Completed | LessonLifecycle::Blocked | LessonLifecycle::Abandoned
        ) {
            return Err(TransitionError::TerminalState);
        }
        if p.lifecycle != LessonLifecycle::Active {
            return Err(TransitionError::InvalidState {
                message: "routing requires active lifecycle",
            });
        }
        if decision.student_id() != p.student_id {
            return Err(TransitionError::PedagogyStudentMismatch);
        }
        if decision.policy_version() != LESSON_POLICY_V1 {
            return Err(TransitionError::PedagogyPolicyVersionMismatch {
                expected: LESSON_POLICY_V1,
                actual: decision.policy_version(),
            });
        }
        let current = p.current_step_id.ok_or(TransitionError::InvalidState {
            message: "active state requires cursor",
        })?;
        let step =
            l.steps
                .iter()
                .find(|s| s.id == current)
                .ok_or(TransitionError::InvalidState {
                    message: "cursor is dangling",
                })?;
        let mapped = step.objective_ids.iter().any(|o| {
            curriculum.objective_mappings.iter().any(|m| {
                m.objective_id == *o && m.competency_ids.contains(&decision.competency_id())
            })
        });
        if !mapped {
            return Err(TransitionError::PedagogyCompetencyMismatch);
        }
        let option = decision.selected_option();
        if matches!(option, InstructionalOption::Assess) {
            return Err(TransitionError::IncompatibleRoute {
                option,
                step_id: current,
            });
        }
        let target = *step
            .routes
            .get(&option)
            .ok_or(TransitionError::RouteUnavailable {
                option,
                step_id: current,
            })?;
        let unmet: Vec<_> = l
            .prerequisites
            .difference(completed_lessons)
            .copied()
            .collect();
        if !unmet.is_empty() {
            return Err(TransitionError::UnmetPrerequisites { lesson_ids: unmet });
        }
        let mut n = p.clone();
        n.completed_steps.insert(current);
        n.current_step_id = Some(target);
        n.updated_at = Some(at);
        n.rationale = vec![
            format!("pedagogy.option.{option:?}").to_ascii_lowercase(),
            "route.authored".into(),
            "prerequisites.satisfied".into(),
        ];
        Ok(n)
    }
    pub fn advance(
        curriculum: &Curriculum,
        p: &LessonProgress,
        at: Timestamp,
    ) -> Result<LessonProgress, TransitionError> {
        let l = Self::validate(curriculum, p)?;
        Self::validate_timestamp(p, at)?;
        if p.lifecycle != LessonLifecycle::Active {
            return Err(TransitionError::TerminalState);
        }
        let current = p.current_step_id.ok_or(TransitionError::InvalidState {
            message: "active state requires cursor",
        })?;
        let mut n = p.clone();
        n.completed_steps.insert(current);
        n.updated_at = Some(at);
        if current == l.completion_step_id {
            n.lifecycle = LessonLifecycle::Completed;
            n.current_step_id = None;
            n.completed_at = Some(at);
            n.rationale = vec![
                "lesson.completed".into(),
                "completion_step.completed".into(),
            ];
        } else {
            let index = l.steps.iter().position(|s| s.id == current).ok_or(
                TransitionError::InvalidState {
                    message: "cursor is dangling",
                },
            )?;
            n.current_step_id = Some(l.steps[index + 1].id);
            n.rationale = vec!["lesson.advanced".into(), "authored.order".into()];
        }
        Ok(n)
    }
    pub fn wait(
        curriculum: &Curriculum,
        p: &LessonProgress,
        at: Timestamp,
    ) -> Result<LessonProgress, TransitionError> {
        Self::set_open(
            curriculum,
            p,
            LessonLifecycle::Waiting,
            at,
            "lesson.waiting",
        )
    }
    pub fn resume(
        curriculum: &Curriculum,
        p: &LessonProgress,
        at: Timestamp,
    ) -> Result<LessonProgress, TransitionError> {
        Self::validate(curriculum, p)?;
        Self::validate_timestamp(p, at)?;
        if p.lifecycle != LessonLifecycle::Waiting {
            return Err(TransitionError::InvalidState {
                message: "resume requires waiting lifecycle",
            });
        }
        let mut n = p.clone();
        n.lifecycle = LessonLifecycle::Active;
        n.updated_at = Some(at);
        n.rationale = vec!["lesson.resumed".into()];
        Ok(n)
    }
    fn set_open(
        curriculum: &Curriculum,
        p: &LessonProgress,
        state: LessonLifecycle,
        at: Timestamp,
        reason: &str,
    ) -> Result<LessonProgress, TransitionError> {
        Self::validate(curriculum, p)?;
        Self::validate_timestamp(p, at)?;
        if p.lifecycle != LessonLifecycle::Active {
            return Err(TransitionError::TerminalState);
        }
        let mut n = p.clone();
        n.lifecycle = state;
        n.updated_at = Some(at);
        n.rationale = vec![reason.into()];
        Ok(n)
    }
    pub fn block(
        curriculum: &Curriculum,
        p: &LessonProgress,
        at: Timestamp,
    ) -> Result<LessonProgress, TransitionError> {
        let mut n = Self::set_open(
            curriculum,
            p,
            LessonLifecycle::Blocked,
            at,
            "lesson.blocked",
        )?;
        n.current_step_id = None;
        Ok(n)
    }
    pub fn abandon(
        curriculum: &Curriculum,
        p: &LessonProgress,
        at: Timestamp,
    ) -> Result<LessonProgress, TransitionError> {
        let mut n = Self::set_open(
            curriculum,
            p,
            LessonLifecycle::Abandoned,
            at,
            "lesson.abandoned",
        )?;
        n.current_step_id = None;
        Ok(n)
    }
}

pub trait LessonProgressRepository {
    type Error;
    fn load(
        &self,
        student: StudentId,
        lesson: LessonId,
    ) -> Result<Option<LessonProgress>, Self::Error>;
    fn save(&mut self, progress: LessonProgress) -> Result<(), Self::Error>;
}
#[derive(Default)]
pub struct InMemoryLessonProgressRepository {
    items: BTreeMap<(StudentId, LessonId), LessonProgress>,
}
impl LessonProgressRepository for InMemoryLessonProgressRepository {
    type Error = std::convert::Infallible;
    fn load(&self, s: StudentId, l: LessonId) -> Result<Option<LessonProgress>, Self::Error> {
        Ok(self.items.get(&(s, l)).cloned())
    }
    fn save(&mut self, p: LessonProgress) -> Result<(), Self::Error> {
        self.items.insert((p.student_id, p.lesson_id), p);
        Ok(())
    }
}
