use nexa_knowledge::*;
use std::{collections::BTreeMap, str::FromStr};

fn id<T: FromStr>(s: &str) -> T
where
    T::Err: std::fmt::Debug,
{
    s.parse().unwrap()
}
fn record(
    n: u64,
    body: &str,
    status: SourceStatus,
    visibility: KnowledgeVisibility,
    course: Option<&str>,
    lesson: Option<&str>,
) -> (KnowledgeSource, KnowledgeArtifact, KnowledgeChunk) {
    let s = KnowledgeSource {
        contract_version: V1,
        source_id: id(&format!("018f0000-0000-7000-8000-{n:012x}")),
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
    let a = KnowledgeArtifact::new(
        id(&format!("018f0000-0000-7000-9000-{n:012x}")),
        &s,
        "text/plain",
        body.as_bytes().to_vec(),
        id("2026-08-19T00:00:00Z"),
    )
    .unwrap();
    let (h, br, lr) = structural_ranges(&s, &a).unwrap().remove(0);
    let c = KnowledgeChunk {
        contract_version: V1,
        chunk_id: id(&format!("018f0000-0000-7000-a000-{n:012x}")),
        source_id: s.source_id,
        source_version: 1,
        artifact_id: a.artifact_id,
        ordinal: 0,
        heading_path: h,
        byte_range: br.clone(),
        line_range: lr,
        original_content_hash: a.content_hash.clone(),
        chunk_content_hash: ContentHash::sha256(&a.bytes()[br.start as usize..br.end as usize]),
        chunking_policy_version: V1,
    };
    (s, a, c)
}
fn profile() -> EmbeddingProfile {
    EmbeddingProfile::new(
        id("018f0000-0000-7000-b000-000000000001"),
        "neutral.family-v1",
        3,
        BTreeMap::from([("revision".into(), "stable-1".into())]),
    )
    .unwrap()
}
fn embedding(n: u64, c: &KnowledgeChunk, p: &EmbeddingProfile, v: [i16; 3]) -> ChunkEmbedding {
    ChunkEmbedding::new(
        id(&format!("018f0000-0000-7000-c000-{n:012x}")),
        c,
        p,
        EmbeddingVector::new(v.to_vec(), 3).unwrap(),
        id("2026-08-19T00:00:01Z"),
    )
    .unwrap()
}
fn records(
    rows: Vec<(
        KnowledgeSource,
        KnowledgeArtifact,
        KnowledgeChunk,
        Option<[i16; 3]>,
    )>,
    p: &EmbeddingProfile,
) -> VectorCorpusRecords {
    let mut r = VectorCorpusRecords {
        sources: vec![],
        artifacts: vec![],
        chunks: vec![],
        profiles: vec![p.clone()],
        embeddings: vec![],
    };
    for (n, (s, a, c, v)) in rows.into_iter().enumerate() {
        if let Some(v) = v {
            r.embeddings.push(embedding(n as u64 + 1, &c, p, v))
        }
        r.sources.push(s);
        r.artifacts.push(a);
        r.chunks.push(c)
    }
    r
}
fn query(p: &EmbeddingProfile, v: [i16; 3], limit: usize) -> VectorRetrievalQuery {
    VectorRetrievalQuery::new(
        id("018f0000-0000-7000-d000-000000000001"),
        id("018f0000-0000-7000-d000-000000000002"),
        p,
        EmbeddingVector::new(v.to_vec(), 3).unwrap(),
        VectorRetrievalFilters {
            audience: Audience::StudentLearning,
            course_id: None,
            lesson_id: None,
        },
        limit,
    )
    .unwrap()
}

#[test]
fn profile_fingerprint_and_wire_validation_are_exact() {
    let p = profile();
    let json = serde_json::to_string(&p).unwrap();
    assert_eq!(serde_json::from_str::<EmbeddingProfile>(&json).unwrap(), p);
    let same = profile();
    assert_eq!(p.fingerprint, same.fingerprint);
    let mut changed = serde_json::to_value(&p).unwrap();
    changed["model_family"] = "different".into();
    assert!(serde_json::from_value::<EmbeddingProfile>(changed).is_err());
    let mut unknown = serde_json::to_value(&p).unwrap();
    unknown["contract_version"] = "2.0".into();
    assert!(serde_json::from_value::<EmbeddingProfile>(unknown).is_err());
    let mut extra = serde_json::to_value(&p).unwrap();
    extra["endpoint"] = "secret".into();
    assert!(serde_json::from_value::<EmbeddingProfile>(extra).is_err());
}

#[test]
fn vectors_and_queries_reject_malformed_standalone_wire() {
    for bad in [
        serde_json::json!([]),
        serde_json::json!([32768]),
        serde_json::Value::Array(vec![serde_json::json!(0); MAX_VECTOR_DIMENSION + 1]),
    ] {
        assert!(serde_json::from_value::<EmbeddingVector>(bad).is_err())
    }
    assert!(EmbeddingVector::new(vec![], 0).is_err());
    assert!(EmbeddingVector::new(vec![1], MAX_VECTOR_DIMENSION + 1).is_err());
    let p = profile();
    let q = query(&p, [1, -2, 3], 2);
    let wire = serde_json::to_string(&q).unwrap();
    let decoded: VectorRetrievalQuery = serde_json::from_str(&wire).unwrap();
    assert_eq!(
        serde_json::to_value(decoded).unwrap(),
        serde_json::to_value(&q).unwrap()
    );
    for (field, bad) in [
        ("contract_version", serde_json::json!("2.0")),
        ("vector_policy_version", serde_json::json!("2.0")),
        ("maximum_results", serde_json::json!(0)),
        ("dimension", serde_json::json!(2)),
    ] {
        let mut value = serde_json::to_value(&q).unwrap();
        value[field] = bad;
        assert!(serde_json::from_value::<VectorRetrievalQuery>(value).is_err())
    }
}

#[test]
fn exact_metric_ties_limits_missing_and_order_are_deterministic() {
    let p = profile();
    let a = record(
        2,
        "secret text",
        SourceStatus::Active,
        KnowledgeVisibility::Student,
        None,
        None,
    );
    let b = record(
        1,
        "other source",
        SourceStatus::Active,
        KnowledgeVisibility::Student,
        None,
        None,
    );
    let missing = record(
        3,
        "missing vector",
        SourceStatus::Active,
        KnowledgeVisibility::Student,
        None,
        None,
    );
    let r = records(
        vec![
            (a.0, a.1, a.2, Some([2, -3, 4])),
            (b.0, b.1, b.2, Some([2, -3, 4])),
            (missing.0, missing.1, missing.2, None),
        ],
        &p,
    );
    let mut reversed = r.clone();
    reversed.sources.reverse();
    reversed.artifacts.reverse();
    reversed.chunks.reverse();
    reversed.embeddings.reverse();
    let one = InMemoryVectorSnapshot::from_records(r, p.profile_id)
        .unwrap()
        .retrieve(&query(&p, [5, 6, -2], 1))
        .unwrap();
    let two = InMemoryVectorSnapshot::from_records(reversed, p.profile_id)
        .unwrap()
        .retrieve(&query(&p, [5, 6, -2], 1))
        .unwrap();
    assert_eq!(one, two);
    assert_eq!(one.candidates[0].evidence.exact_dot_product, -16);
    assert!(one
        .exclusions
        .iter()
        .any(|e| e.reason == VectorExclusionReason::MissingEmbedding));
    assert!(one
        .exclusions
        .iter()
        .any(|e| e.reason == VectorExclusionReason::ResultLimit));
    assert!(format!("{one:?}").contains("exact_dot_product"));
    let wire = serde_json::to_string(&one).unwrap();
    assert!(!wire.contains("secret text"));
    assert!(!wire.contains("[2,-3,4]"));
}

#[test]
fn governance_scope_and_inactive_filters_precede_ranking() {
    let p = profile();
    let rows = vec![
        record(
            1,
            "a",
            SourceStatus::Active,
            KnowledgeVisibility::AssessmentProtected,
            None,
            None,
        ),
        record(
            2,
            "b",
            SourceStatus::Active,
            KnowledgeVisibility::Instructor,
            None,
            None,
        ),
        record(
            3,
            "c",
            SourceStatus::Active,
            KnowledgeVisibility::Student,
            Some("018f0000-0000-7000-e000-000000000001"),
            None,
        ),
        record(
            4,
            "d",
            SourceStatus::Active,
            KnowledgeVisibility::Student,
            Some("018f0000-0000-7000-e000-000000000099"),
            Some("018f0000-0000-7000-e000-000000000002"),
        ),
    ];
    let rows = rows
        .into_iter()
        .map(|(s, a, c)| (s, a, c, Some([1, 1, 1])))
        .collect();
    let snap = InMemoryVectorSnapshot::from_records(records(rows, &p), p.profile_id).unwrap();
    let mut q = query(&p, [1, 1, 1], 4);
    q.filters.course_id = Some(id("018f0000-0000-7000-e000-000000000099"));
    q.filters.lesson_id = Some(id("018f0000-0000-7000-e000-000000000099"));
    let result = snap.retrieve(&q).unwrap();
    assert!(result.candidates.is_empty());
    assert!(result
        .exclusions
        .iter()
        .any(|e| e.reason == VectorExclusionReason::AssessmentProtected));
    assert!(result
        .exclusions
        .iter()
        .any(|e| e.reason == VectorExclusionReason::AudienceRestricted));
    assert!(result
        .exclusions
        .iter()
        .any(|e| e.reason == VectorExclusionReason::CourseScopeMismatch));
    assert!(result
        .exclusions
        .iter()
        .any(|e| e.reason == VectorExclusionReason::LessonScopeMismatch));
}

#[test]
fn snapshot_rejects_duplicates_orphans_conflicts_and_profile_mismatch() {
    let p = profile();
    let (s, a, c) = record(
        1,
        "inert",
        SourceStatus::Active,
        KnowledgeVisibility::Student,
        None,
        None,
    );
    let base = records(vec![(s, a, c, Some([1, 2, 3]))], &p);
    let mut duplicate = base.clone();
    let mut forged = duplicate.embeddings[0].clone();
    let value = serde_json::to_value(&forged).unwrap();
    forged = serde_json::from_value(value).unwrap();
    duplicate.embeddings.push(forged);
    assert!(InMemoryVectorSnapshot::from_records(duplicate, p.profile_id).is_err());
    let mut orphan = base.clone();
    orphan.chunks.clear();
    assert!(InMemoryVectorSnapshot::from_records(orphan, p.profile_id).is_err());
    let mut mismatch = serde_json::to_value(&base.embeddings[0]).unwrap();
    mismatch["chunk_content_hash"]["digest"] =
        "0000000000000000000000000000000000000000000000000000000000000000".into();
    let bad: ChunkEmbedding = serde_json::from_value(mismatch).unwrap();
    let mut corrupt = base.clone();
    corrupt.embeddings[0] = bad;
    assert!(InMemoryVectorSnapshot::from_records(corrupt, p.profile_id).is_err());
    let other = EmbeddingProfile::new(
        id("018f0000-0000-7000-b000-000000000002"),
        "other",
        3,
        BTreeMap::new(),
    )
    .unwrap();
    assert_eq!(
        InMemoryVectorSnapshot::from_records(base, p.profile_id)
            .unwrap()
            .retrieve(&query(&other, [1, 2, 3], 1)),
        Err(VectorError::ProfileMismatch)
    );
}

#[test]
fn debug_output_redacts_payloads() {
    let p = profile();
    assert!(!format!("{p:?}").contains("stable-1"));
    let (s, a, c) = record(
        1,
        "query/source secret",
        SourceStatus::Active,
        KnowledgeVisibility::Student,
        None,
        None,
    );
    let e = embedding(1, &c, &p, [123, 456, 789]);
    let q = query(&p, [321, 654, 987], 1);
    assert!(!format!("{e:?}{q:?}").contains("123"));
    assert!(!serde_json::to_string(
        &InMemoryVectorSnapshot::from_records(
            records(vec![(s, a, c, Some([123, 456, 789]))], &p),
            p.profile_id
        )
        .unwrap()
        .retrieve(&q)
        .unwrap()
    )
    .unwrap()
    .contains("987"));
}
