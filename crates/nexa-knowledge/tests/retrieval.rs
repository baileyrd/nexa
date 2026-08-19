use nexa_knowledge::*;
use std::{collections::BTreeMap, str::FromStr};

fn id<T: FromStr>(value: &str) -> T
where
    T::Err: std::fmt::Debug,
{
    value.parse().unwrap()
}

fn record(
    suffixes: [u64; 3],
    body: &str,
    visibility: KnowledgeVisibility,
    status: SourceStatus,
    course: Option<&str>,
    lesson: Option<&str>,
) -> (KnowledgeSource, KnowledgeArtifact, KnowledgeChunk) {
    let [source_suffix, artifact_suffix, chunk_suffix] = suffixes;
    let source = KnowledgeSource {
        contract_version: V1,
        source_id: id(&format!("018f0000-0000-7000-8000-{source_suffix:012x}")),
        source_version: 1,
        source_type: SourceType::PlainText,
        authority: SourceAuthority::Approved,
        trust: SourceTrust::Reviewed,
        origin: SourceOrigin::Authored,
        status,
        visibility,
        scope: KnowledgeScope {
            course_id: course.map(id),
            lesson_id: lesson.map(id),
        },
        source_metadata: BTreeMap::new(),
        inferred_metadata: BTreeMap::new(),
        registered_at: id("2026-08-19T00:00:00Z"),
    };
    let artifact = KnowledgeArtifact::new(
        id(&format!("018f0000-0000-7000-9000-{artifact_suffix:012x}")),
        &source,
        "text/plain",
        body.as_bytes().to_vec(),
        id("2026-08-19T00:00:00Z"),
    )
    .unwrap();
    let (heading_path, byte_range, line_range) =
        structural_ranges(&source, &artifact).unwrap().remove(0);
    let content = &artifact.bytes()[byte_range.start as usize..byte_range.end as usize];
    let chunk = KnowledgeChunk {
        contract_version: V1,
        chunk_id: id(&format!("018f0000-0000-7000-a000-{chunk_suffix:012x}")),
        source_id: source.source_id,
        source_version: source.source_version,
        artifact_id: artifact.artifact_id,
        ordinal: 0,
        heading_path,
        byte_range,
        line_range,
        original_content_hash: artifact.content_hash.clone(),
        chunk_content_hash: ContentHash::sha256(content),
        chunking_policy_version: V1,
    };
    (source, artifact, chunk)
}

fn corpus(
    records: Vec<(KnowledgeSource, KnowledgeArtifact, KnowledgeChunk)>,
) -> RetrievalCorpusRecords {
    let mut sources = vec![];
    let mut artifacts = vec![];
    let mut chunks = vec![];
    for (source, artifact, chunk) in records {
        sources.push(source);
        artifacts.push(artifact);
        chunks.push(chunk);
    }
    RetrievalCorpusRecords {
        sources,
        artifacts,
        chunks,
    }
}

fn query(text: &str, limit: usize) -> RetrievalQuery {
    RetrievalQuery {
        contract_version: V1,
        retrieval_policy_version: LEXICAL_RETRIEVAL_V1,
        query_id: id("018f0000-0000-7000-b000-000000000001"),
        result_id: id("018f0000-0000-7000-b000-000000000002"),
        text: text.into(),
        filters: RetrievalFilters {
            audience: Audience::StudentLearning,
            course_id: None,
            lesson_id: None,
        },
        maximum_results: limit,
    }
}

#[test]
fn tokenization_is_exact_bounded_and_line_ending_independent() {
    assert_eq!(
        tokenize("Rust,RUST Straße\r\nCAFÉ λ_2").unwrap(),
        ["rust", "rust", "straße", "café", "λ", "2"]
    );
    assert_eq!(tokenize("a\r\nb").unwrap(), tokenize("a\nb").unwrap());
    assert!(tokenize(&"x".repeat(MAX_RETRIEVAL_TERM_BYTES + 1)).is_err());
    assert!(tokenize("...\r\n").unwrap().is_empty());
}

#[test]
fn query_and_result_have_golden_round_trips_and_reject_malformed_wire() {
    let q = query("Rust ownership", 2);
    let wire = serde_json::to_string(&q).unwrap();
    assert_eq!(serde_json::from_str::<RetrievalQuery>(&wire).unwrap(), q);
    let mut value = serde_json::to_value(&q).unwrap();
    value["unexpected"] = true.into();
    assert!(serde_json::from_value::<RetrievalQuery>(value).is_err());
    for bad in ["", "---"] {
        let mut value = serde_json::to_value(&q).unwrap();
        value["text"] = bad.into();
        assert!(serde_json::from_value::<RetrievalQuery>(value).is_err());
    }
    let mut value = serde_json::to_value(&q).unwrap();
    value["retrieval_policy_version"] = "2.0".into();
    assert!(serde_json::from_value::<RetrievalQuery>(value).is_err());
    let mut value = serde_json::to_value(&q).unwrap();
    value["maximum_results"] = (MAX_RETRIEVAL_RESULTS + 1).into();
    assert!(serde_json::from_value::<RetrievalQuery>(value).is_err());
    let mut value = serde_json::to_value(&q).unwrap();
    value["text"] = ("x ".repeat(MAX_RETRIEVAL_QUERY_BYTES)).into();
    assert!(serde_json::from_value::<RetrievalQuery>(value).is_err());

    let snapshot = InMemoryRetrievalSnapshot::from_records(corpus(vec![record(
        [1, 1, 1],
        "Rust ownership",
        KnowledgeVisibility::Student,
        SourceStatus::Active,
        None,
        None,
    )]))
    .unwrap();
    let result = snapshot.retrieve(&q).unwrap();
    let wire = serde_json::to_string(&result).unwrap();
    assert_eq!(
        serde_json::from_str::<RetrievalResult>(&wire).unwrap(),
        result
    );
    let mut value = serde_json::to_value(&result).unwrap();
    value["candidates"][0]["score"] = serde_json::json!(0.5);
    assert!(serde_json::from_value::<RetrievalResult>(value).is_err());
}

#[test]
fn scoring_repeated_terms_document_frequency_and_ties_are_explicit() {
    let snapshot = InMemoryRetrievalSnapshot::from_records(corpus(vec![
        record(
            [1, 1, 2],
            "rust rust memory",
            KnowledgeVisibility::Student,
            SourceStatus::Active,
            None,
            None,
        ),
        record(
            [2, 2, 1],
            "rust safety",
            KnowledgeVisibility::Student,
            SourceStatus::Active,
            None,
            None,
        ),
        record(
            [3, 3, 3],
            "memory only",
            KnowledgeVisibility::Student,
            SourceStatus::Active,
            None,
            None,
        ),
    ]))
    .unwrap();
    let result = snapshot.retrieve(&query("RUST rust", 3)).unwrap();
    assert_eq!(
        result
            .candidates
            .iter()
            .map(|c| c.chunk_id)
            .collect::<Vec<_>>(),
        vec![
            id("018f0000-0000-7000-a000-000000000002"),
            id("018f0000-0000-7000-a000-000000000001"),
        ]
    );
    let evidence = &result.candidates[0].score_evidence[0];
    assert_eq!(
        (
            evidence.query_frequency,
            evidence.term_frequency,
            evidence.document_frequency,
            evidence.document_count,
            evidence.contribution
        ),
        (2, 2, 2, 3, 8)
    );
    assert_eq!(result.candidates[0].score.get(), 8.0);
    assert_eq!(result.candidates[1].score.get(), 4.0);
}

#[test]
fn governance_and_scope_are_applied_before_scoring() {
    let course = "018f0000-0000-7000-c000-000000000001";
    let lesson = "018f0000-0000-7000-c000-000000000002";
    let snapshot = InMemoryRetrievalSnapshot::from_records(corpus(vec![
        record(
            [1, 1, 1],
            "rust",
            KnowledgeVisibility::Student,
            SourceStatus::Active,
            Some(course),
            Some(lesson),
        ),
        record(
            [2, 2, 2],
            "rust",
            KnowledgeVisibility::AssessmentProtected,
            SourceStatus::Active,
            Some(course),
            Some(lesson),
        ),
        record(
            [3, 3, 3],
            "rust",
            KnowledgeVisibility::Instructor,
            SourceStatus::Active,
            Some(course),
            Some(lesson),
        ),
        record(
            [4, 4, 4],
            "rust",
            KnowledgeVisibility::Student,
            SourceStatus::Active,
            None,
            None,
        ),
        record(
            [5, 5, 5],
            "rust",
            KnowledgeVisibility::Student,
            SourceStatus::Active,
            Some(course),
            None,
        ),
    ]))
    .unwrap();
    let mut q = query("rust", 10);
    q.filters.course_id = Some(id(course));
    q.filters.lesson_id = Some(id(lesson));
    let result = snapshot.retrieve(&q).unwrap();
    assert_eq!(result.candidates.len(), 1);
    assert_eq!(result.candidates[0].score_evidence[0].document_count, 1);
    assert!(result
        .exclusions
        .iter()
        .any(|e| e.reason == RetrievalExclusionReason::AssessmentProtected));
    assert!(result
        .exclusions
        .iter()
        .any(|e| e.reason == RetrievalExclusionReason::AudienceRestricted));
    assert!(result
        .exclusions
        .iter()
        .any(|e| e.reason == RetrievalExclusionReason::CourseScopeMismatch));
    assert!(result
        .exclusions
        .iter()
        .any(|e| e.reason == RetrievalExclusionReason::LessonScopeMismatch));
}

#[test]
fn inactive_versions_are_excluded_and_equivalent_orders_are_identical() {
    let active = record(
        [1, 1, 1],
        "active rust",
        KnowledgeVisibility::Student,
        SourceStatus::Active,
        None,
        None,
    );
    let mut old = record(
        [1, 2, 2],
        "old rust",
        KnowledgeVisibility::Student,
        SourceStatus::Superseded,
        None,
        None,
    );
    old.0.source_version = 2;
    old.1 = KnowledgeArtifact::new(
        old.1.artifact_id,
        &old.0,
        "text/plain",
        b"old rust".to_vec(),
        id("2026-08-19T00:00:00Z"),
    )
    .unwrap();
    let ranges = structural_ranges(&old.0, &old.1).unwrap();
    old.2.source_version = 2;
    old.2.artifact_id = old.1.artifact_id;
    old.2.byte_range = ranges[0].1.clone();
    old.2.line_range = ranges[0].2.clone();
    old.2.original_content_hash = old.1.content_hash.clone();
    old.2.chunk_content_hash = ContentHash::sha256(b"old rust");
    let first =
        InMemoryRetrievalSnapshot::from_records(corpus(vec![active.clone(), old.clone()])).unwrap();
    let second = InMemoryRetrievalSnapshot::from_records(corpus(vec![old, active])).unwrap();
    let q = query("rust", 10);
    let a = first.retrieve(&q).unwrap();
    let b = second.retrieve(&q).unwrap();
    assert_eq!(a, b);
    assert_eq!(a.candidates.len(), 1);
    assert!(a
        .exclusions
        .iter()
        .any(|e| e.reason == RetrievalExclusionReason::NotActive));
}

#[test]
fn limits_no_match_and_redaction_are_enforced() {
    let snapshot = InMemoryRetrievalSnapshot::from_records(corpus(vec![
        record(
            [1, 1, 1],
            "rust",
            KnowledgeVisibility::Student,
            SourceStatus::Active,
            None,
            None,
        ),
        record(
            [2, 2, 2],
            "rust",
            KnowledgeVisibility::Student,
            SourceStatus::Active,
            None,
            None,
        ),
    ]))
    .unwrap();
    assert_eq!(
        snapshot
            .retrieve(&query("rust", 1))
            .unwrap()
            .candidates
            .len(),
        1
    );
    assert!(snapshot
        .retrieve(&query("python", 2))
        .unwrap()
        .candidates
        .is_empty());
    assert!(query("rust", 0).validate().is_err());
    assert!(!format!("{:?}", query("private-query-secret", 1)).contains("private-query-secret"));
    assert!(!format!("{}", RetrievalError::InvalidQuery).contains("private"));
}

#[test]
fn conflicting_incomplete_and_corrupted_corpora_fail_closed() {
    let valid = record(
        [1, 1, 1],
        "secret body",
        KnowledgeVisibility::Student,
        SourceStatus::Active,
        None,
        None,
    );
    let mut duplicate = corpus(vec![valid.clone()]);
    duplicate.sources.push(valid.0.clone());
    assert!(matches!(
        InMemoryRetrievalSnapshot::from_records(duplicate),
        Err(RetrievalError::InvalidCorpus)
    ));

    let mut missing = corpus(vec![valid.clone()]);
    missing.artifacts.clear();
    assert!(InMemoryRetrievalSnapshot::from_records(missing).is_err());

    let mut corrupt = valid;
    corrupt.2.chunk_content_hash = ContentHash::sha256(b"forged");
    assert!(matches!(
        InMemoryRetrievalSnapshot::from_records(corpus(vec![corrupt])),
        Err(RetrievalError::IntegrityFailure)
    ));

    let active = record(
        [2, 2, 2],
        "new body",
        KnowledgeVisibility::Student,
        SourceStatus::Active,
        None,
        None,
    );
    let mut conflict = record(
        [2, 3, 3],
        "new body",
        KnowledgeVisibility::Student,
        SourceStatus::Active,
        None,
        None,
    );
    conflict.0.source_version = 2;
    conflict.1 = KnowledgeArtifact::new(
        conflict.1.artifact_id,
        &conflict.0,
        "text/plain",
        b"new body".to_vec(),
        id("2026-08-19T00:00:00Z"),
    )
    .unwrap();
    conflict.2.source_version = 2;
    conflict.2.original_content_hash = conflict.1.content_hash.clone();
    assert!(matches!(
        InMemoryRetrievalSnapshot::from_records(corpus(vec![active, conflict])),
        Err(RetrievalError::InvalidCorpus)
    ));
}

#[test]
fn repository_port_is_synchronous_and_results_only_carry_references() {
    let (source, artifact, chunk) = record(
        [1, 1, 1],
        "never copied",
        KnowledgeVisibility::Student,
        SourceStatus::Active,
        None,
        None,
    );
    let mut repository = InMemoryKnowledgeRepository::default();
    repository
        .sources
        .insert((source.source_id, source.source_version), source);
    repository.artifacts.insert(artifact.artifact_id, artifact);
    repository.chunks.insert(chunk.chunk_id, chunk);
    let snapshot = InMemoryRetrievalSnapshot::load(&repository).unwrap();
    let wire = serde_json::to_string(&snapshot.retrieve(&query("copied", 1)).unwrap()).unwrap();
    assert!(!wire.contains("never copied"));
}
