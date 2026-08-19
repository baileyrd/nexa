//! Dependency-light canonical values shared by Nexa contracts.
#![forbid(unsafe_code)]

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str::FromStr};
use thiserror::Error;
use uuid::Uuid;

/// An error raised while constructing or decoding a canonical value.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ValueError {
    #[error("{kind} must not be the nil UUID")]
    NilIdentifier { kind: &'static str },
    #[error("invalid {kind}: {message}")]
    InvalidIdentifier { kind: &'static str, message: String },
    #[error("version must use MAJOR.MINOR with unsigned 16-bit components")]
    InvalidVersion,
    #[error("confidence must be finite and within the inclusive range 0.0..=1.0")]
    InvalidConfidence,
    #[error("mastery score must be finite and within the inclusive range 0.0..=1.0")]
    InvalidMasteryScore,
    #[error("timestamp must be a valid RFC 3339 instant")]
    InvalidTimestamp,
}

macro_rules! uuid_id {
    ($($name:ident),+ $(,)?) => {$(
        #[doc = concat!("A validated, non-nil ", stringify!($name), ".")]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new(value: Uuid) -> Result<Self, ValueError> {
                if value.is_nil() { Err(ValueError::NilIdentifier { kind: stringify!($name) }) } else { Ok(Self(value)) }
            }
            pub const fn as_uuid(&self) -> &Uuid { &self.0 }
            pub const fn into_uuid(self) -> Uuid { self.0 }
        }
        impl fmt::Display for $name { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) } }
        impl FromStr for $name {
            type Err = ValueError;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                let parsed = Uuid::parse_str(value).map_err(|error| ValueError::InvalidIdentifier { kind: stringify!($name), message: error.to_string() })?;
                Self::new(parsed)
            }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = Uuid::deserialize(deserializer)?;
                Self::new(value).map_err(de::Error::custom)
            }
        }
    )+};
}

uuid_id!(
    StudentId,
    CompetencyId,
    LearningObjectiveId,
    EvidenceId,
    AttemptId,
    AssessmentId,
    QuestionId,
    ResponseId,
    AssessmentItemInstanceId,
    RubricId,
    RubricCriterionId,
    CurriculumId,
    CourseId,
    ModuleId,
    LessonId,
    LessonStepId,
    LessonTransitionId,
    SessionId,
    EventId,
    MessageId,
    BehaviorId,
    CorrelationId,
    TraceId,
    KnowledgeSourceId,
    KnowledgeArtifactId,
    KnowledgeChunkId,
    IngestionJobId,
    EmbeddingRecordId,
    EmbeddingProfileId,
    RetrievalQueryId,
    RetrievalResultId,
    HybridRetrievalResultId,
    CitationId
);

/// A finite mastery estimate in the inclusive range `[0, 1]`.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct MasteryScore(f64);
impl MasteryScore {
    pub fn new(value: f64) -> Result<Self, ValueError> {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(ValueError::InvalidMasteryScore)
        }
    }
    pub const fn get(self) -> f64 {
        self.0
    }
}
impl<'de> Deserialize<'de> for MasteryScore {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(f64::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

macro_rules! semantic_string {
    ($($name:ident),+ $(,)?) => {$(
        #[doc = concat!("A non-empty semantic ", stringify!($name), " containing safe identifier characters.")]
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);
        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, ValueError> {
                let value = value.into();
                if !value.is_empty() && value.len() <= 255 && value.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-')) {
                    Ok(Self(value))
                } else {
                    Err(ValueError::InvalidIdentifier { kind: stringify!($name), message: "expected 1..=255 ASCII letters, digits, '.', '_' or '-'".into() })
                }
            }
            pub fn as_str(&self) -> &str { &self.0 }
        }
        impl fmt::Display for $name { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.0) } }
        impl FromStr for $name { type Err = ValueError; fn from_str(value: &str) -> Result<Self, Self::Err> { Self::new(value) } }
        impl<'de> Deserialize<'de> for $name { fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { Self::new(String::deserialize(deserializer)?).map_err(de::Error::custom) } }
    )+};
}

semantic_string!(EndpointId, SubjectId, SemanticKey);

/// A protocol or schema version independent of crate releases.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProtocolVersion {
    major: u16,
    minor: u16,
}
impl ProtocolVersion {
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
    pub const fn major(self) -> u16 {
        self.major
    }
    pub const fn minor(self) -> u16 {
        self.minor
    }
}
impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}
impl FromStr for ProtocolVersion {
    type Err = ValueError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (major, minor) = value.split_once('.').ok_or(ValueError::InvalidVersion)?;
        if minor.contains('.') || major.is_empty() || minor.is_empty() {
            return Err(ValueError::InvalidVersion);
        }
        Ok(Self::new(
            major.parse().map_err(|_| ValueError::InvalidVersion)?,
            minor.parse().map_err(|_| ValueError::InvalidVersion)?,
        ))
    }
}
impl Serialize for ProtocolVersion {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
impl<'de> Deserialize<'de> for ProtocolVersion {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

/// A UTC instant, serialized in RFC 3339 form.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Timestamp(DateTime<Utc>);
impl Timestamp {
    pub const fn new(value: DateTime<Utc>) -> Self {
        Self(value)
    }
    pub const fn get(self) -> DateTime<Utc> {
        self.0
    }
}
impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.to_rfc3339_opts(SecondsFormat::AutoSi, true))
    }
}
impl FromStr for Timestamp {
    type Err = ValueError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        DateTime::parse_from_rfc3339(value)
            .map(|v| Self(v.with_timezone(&Utc)))
            .map_err(|_| ValueError::InvalidTimestamp)
    }
}
impl Serialize for Timestamp {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

/// A finite normalized confidence value in the inclusive range `[0, 1]`.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Confidence(f64);
impl Confidence {
    pub fn new(value: f64) -> Result<Self, ValueError> {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(ValueError::InvalidConfidence)
        }
    }
    pub const fn get(self) -> f64 {
        self.0
    }
}
impl<'de> Deserialize<'de> for Confidence {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(f64::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

/// An elapsed duration represented as non-negative whole milliseconds.
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct DurationMs(u64);
impl DurationMs {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A position in an envelope owner's explicitly scoped ordered stream.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Sequence(u64);
impl Sequence {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ids_reject_nil_in_construction_and_json() {
        assert!(SessionId::new(Uuid::nil()).is_err());
        assert!(
            serde_json::from_str::<SessionId>(r#""00000000-0000-0000-0000-000000000000""#).is_err()
        );
    }
    #[test]
    fn versions_are_strict_and_round_trip() {
        let v: ProtocolVersion = "1.2".parse().unwrap();
        assert_eq!(serde_json::to_string(&v).unwrap(), r#""1.2""#);
        assert!("1.2.3".parse::<ProtocolVersion>().is_err());
    }
    #[test]
    fn timestamps_normalize_offsets() {
        let t: Timestamp = "2026-08-17T19:30:00-04:00".parse().unwrap();
        assert_eq!(t.to_string(), "2026-08-17T23:30:00Z");
    }
    #[test]
    fn confidence_checks_all_boundaries() {
        assert!(Confidence::new(0.0).is_ok());
        assert!(Confidence::new(1.0).is_ok());
        assert!(Confidence::new(-0.1).is_err());
        assert!(Confidence::new(f64::NAN).is_err());
        assert!(serde_json::from_str::<Confidence>("1.1").is_err());
    }
}
