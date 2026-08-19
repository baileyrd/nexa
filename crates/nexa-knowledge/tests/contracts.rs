use nexa_knowledge::*;
use std::{collections::BTreeMap, str::FromStr};
fn id<T: FromStr>(s: &str) -> T
where
    T::Err: std::fmt::Debug,
{
    s.parse().unwrap()
}
fn source(kind: SourceType) -> KnowledgeSource {
    KnowledgeSource {
        contract_version: V1,
        source_id: id("018f0000-0000-7000-8000-000000000001"),
        source_version: 1,
        source_type: kind,
        authority: SourceAuthority::Approved,
        trust: SourceTrust::Reviewed,
        origin: SourceOrigin::Authored,
        status: SourceStatus::Active,
        visibility: KnowledgeVisibility::Student,
        scope: KnowledgeScope {
            course_id: None,
            lesson_id: None,
        },
        source_metadata: BTreeMap::new(),
        inferred_metadata: BTreeMap::new(),
        registered_at: id("2026-08-19T00:00:00Z"),
    }
}
fn artifact(s: &KnowledgeSource, b: &[u8]) -> KnowledgeArtifact {
    KnowledgeArtifact::new(
        id("018f0000-0000-7000-8000-000000000002"),
        s,
        match s.source_type {
            SourceType::Markdown => "text/markdown",
            _ => "text/plain",
        },
        b.to_vec(),
        id("2026-08-19T00:00:00Z"),
    )
    .unwrap()
}
fn staged_version(
    version: u64,
    suffix: u64,
    body: &[u8],
) -> (
    KnowledgeSource,
    KnowledgeArtifact,
    IngestionJob,
    Vec<KnowledgeChunk>,
) {
    let mut s = source(SourceType::PlainText);
    s.source_version = version;
    s.status = SourceStatus::Staged;
    let artifact_id = id(&format!("018f0000-0000-7000-8000-{suffix:012x}"));
    let a = KnowledgeArtifact::new(
        artifact_id,
        &s,
        "text/plain",
        body.to_vec(),
        id("2026-08-19T00:00:00Z"),
    )
    .unwrap();
    let mut j = IngestionJob {
        contract_version: V1,
        job_id: id(&format!("018f0000-0000-7000-9000-{suffix:012x}")),
        source_id: s.source_id,
        source_version: version,
        artifact_id: a.artifact_id,
        artifact_hash: a.content_hash.clone(),
        state: IngestionState::Registered,
        revision: 0,
        updated_at: id("2026-08-19T00:00:00Z"),
    };
    for (state, at) in [
        (IngestionState::Parsing, "2026-08-19T00:00:01Z"),
        (IngestionState::Chunking, "2026-08-19T00:00:02Z"),
        (IngestionState::Staged, "2026-08-19T00:00:03Z"),
    ] {
        j = j.transition(state, id(at)).unwrap();
    }
    let cs = structural_ranges(&s, &a)
        .unwrap()
        .into_iter()
        .enumerate()
        .map(|(ordinal, (heading_path, byte_range, line_range))| {
            let content = &a.bytes()[byte_range.start as usize..byte_range.end as usize];
            KnowledgeChunk {
                contract_version: V1,
                chunk_id: id(&format!(
                    "018f0000-0000-7000-a000-{:012x}",
                    suffix * 100 + ordinal as u64
                )),
                source_id: s.source_id,
                source_version: version,
                artifact_id: a.artifact_id,
                ordinal: ordinal as u32,
                heading_path,
                byte_range,
                line_range,
                original_content_hash: a.content_hash.clone(),
                chunk_content_hash: ContentHash::sha256(content),
                chunking_policy_version: V1,
            }
        })
        .collect();
    (s, a, j, cs)
}
#[test]
fn sha256_known_vector_and_malformed_wire() {
    assert_eq!(
        ContentHash::sha256(b"abc").digest,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let j = serde_json::to_string(&ContentHash::sha256(b"abc")).unwrap();
    assert_eq!(
        serde_json::from_str::<ContentHash>(&j).unwrap(),
        ContentHash::sha256(b"abc")
    );
    assert!(serde_json::from_str::<ContentHash>(
        r#"{"algorithm":"sha256","contract_version":"1.0","digest":"ABC"}"#
    )
    .is_err())
}
#[test]
fn markdown_ranges_are_exact_ordered_and_fences_inert() {
    let s = source(SourceType::Markdown);
    let a = artifact(
        &s,
        b"# A\r\nhello\r\n```\r\n# inert\r\n```\r\n## B\r\nworld\r\n",
    );
    let r = structural_ranges(&s, &a).unwrap();
    assert_eq!(r.len(), 2);
    assert_eq!(r[0].0, vec!["A"]);
    assert_eq!(r[1].0, vec!["A", "B"]);
    assert_eq!(
        &a.bytes()[r[1].1.start as usize..r[1].1.end as usize],
        b"## B\r\nworld\r\n"
    )
}
#[test]
fn invalid_utf8_empty_and_visibility_fail_closed() {
    let s = source(SourceType::PlainText);
    assert!(KnowledgeArtifact::new(
        id("018f0000-0000-7000-8000-000000000002"),
        &s,
        "text/plain",
        vec![255],
        id("2026-08-19T00:00:00Z")
    )
    .is_err());
    assert!(KnowledgeArtifact::new(
        id("018f0000-0000-7000-8000-000000000002"),
        &s,
        "text/plain",
        vec![],
        id("2026-08-19T00:00:00Z")
    )
    .is_err());
    let mut p = s;
    p.visibility = KnowledgeVisibility::AssessmentProtected;
    assert_eq!(
        exposure(&p, Audience::StudentAssessment),
        Err(ExclusionReason::AssessmentProtected)
    );
    assert!(exposure(&p, Audience::Instructor).is_ok())
}
#[test]
fn transition_rejects_same_terminal_and_regression() {
    let s = source(SourceType::PlainText);
    let a = artifact(&s, b"x");
    let j = IngestionJob {
        contract_version: V1,
        job_id: id("018f0000-0000-7000-8000-000000000003"),
        source_id: s.source_id,
        source_version: 1,
        artifact_id: a.artifact_id,
        artifact_hash: a.content_hash,
        state: IngestionState::Registered,
        revision: 0,
        updated_at: id("2026-08-19T00:00:00Z"),
    };
    assert!(j
        .transition(IngestionState::Registered, id("2026-08-19T00:00:01Z"))
        .is_err());
    let f = j
        .transition(IngestionState::Failed, id("2026-08-19T00:00:01Z"))
        .unwrap();
    assert!(f
        .transition(IngestionState::Parsing, id("2026-08-19T00:00:02Z"))
        .is_err());
    assert!(j
        .transition(IngestionState::Parsing, id("2026-08-18T00:00:00Z"))
        .is_err())
}
#[test]
fn plain_text_is_one_exact_range() {
    let s = source(SourceType::PlainText);
    let a = artifact(&s, "α\nβ\n".as_bytes());
    let r = structural_ranges(&s, &a).unwrap();
    assert_eq!(r.len(), 1);
    assert_eq!((r[0].1.start, r[0].1.end), (0, a.bytes().len() as u64));
}

#[test]
fn malformed_invariant_bearing_json_is_rejected() {
    type MalformedCase = (serde_json::Value, fn(serde_json::Value) -> bool);
    let (s, a, j, cs) = staged_version(1, 10, b"hello\n");
    let cases: Vec<MalformedCase> = vec![
        (serde_json::to_value(&s).unwrap(), |mut v| {
            v["source_version"] = 0.into();
            serde_json::from_value::<KnowledgeSource>(v).is_err()
        }),
        (serde_json::to_value(&a).unwrap(), |mut v| {
            v["byte_length"] = 99.into();
            serde_json::from_value::<KnowledgeArtifact>(v).is_err()
        }),
        (serde_json::to_value(&j).unwrap(), |mut v| {
            v["state"] = "active".into();
            serde_json::from_value::<IngestionJob>(v).is_err()
        }),
        (serde_json::to_value(&cs[0]).unwrap(), |mut v| {
            v["byte_range"]["end"] = 0.into();
            serde_json::from_value::<KnowledgeChunk>(v).is_err()
        }),
    ];
    for (wire, reject) in cases {
        assert!(reject(wire));
    }
}

#[test]
fn chunk_validation_rejects_forged_provenance_and_splits_long_utf8_lines() {
    let body = "α".repeat(MAX_CHUNK_BYTES / 2 + 10);
    let (s, a, _, cs) = staged_version(1, 20, body.as_bytes());
    assert!(cs.len() > 1);
    assert!(cs
        .iter()
        .all(|c| (c.byte_range.end - c.byte_range.start) as usize <= MAX_CHUNK_BYTES));
    for chunk in &cs {
        let wire = serde_json::to_string(chunk).unwrap();
        assert_eq!(
            serde_json::from_str::<KnowledgeChunk>(&wire).unwrap(),
            *chunk
        );
    }
    assert!(validate_chunks(&cs, &s, &a).is_ok());

    let mut forged = cs.clone();
    forged[0].line_range.start = 2;
    assert_eq!(
        validate_chunks(&forged, &s, &a),
        Err(KnowledgeError::InvalidChunk)
    );
    let mut gap = cs.clone();
    gap[1].byte_range.start += 2;
    gap[1].chunk_content_hash = ContentHash::sha256(
        &a.bytes()[gap[1].byte_range.start as usize..gap[1].byte_range.end as usize],
    );
    assert_eq!(
        validate_chunks(&gap, &s, &a),
        Err(KnowledgeError::InvalidChunk)
    );
    let mut mid_codepoint = cs.clone();
    mid_codepoint[0].byte_range.end -= 1;
    assert_eq!(
        validate_chunks(&mid_codepoint, &s, &a),
        Err(KnowledgeError::InvalidChunk)
    );
}

#[test]
fn promotion_supersession_and_rollback_are_atomic() {
    let mut repo = InMemoryKnowledgeRepository::default();
    let (s1, a1, j1, c1) = staged_version(1, 30, b"one\n");
    repo.commit(s1, a1, j1, c1).unwrap();
    repo.promote(
        id("018f0000-0000-7000-8000-000000000001"),
        1,
        id("2026-08-19T00:01:00Z"),
    )
    .unwrap();
    let (s2, a2, j2, c2) = staged_version(2, 31, b"two\n");
    repo.commit(s2, a2, j2, c2).unwrap();
    repo.promote(
        id("018f0000-0000-7000-8000-000000000001"),
        2,
        id("2026-08-19T00:02:00Z"),
    )
    .unwrap();
    assert_eq!(
        repo.sources
            .values()
            .filter(|s| s.status == SourceStatus::Active)
            .count(),
        1
    );
    assert_eq!(
        repo.sources[&(id("018f0000-0000-7000-8000-000000000001"), 1)].status,
        SourceStatus::Superseded
    );
    repo.rollback(
        id("018f0000-0000-7000-8000-000000000001"),
        id("2026-08-19T00:03:00Z"),
    )
    .unwrap();
    assert_eq!(
        repo.sources[&(id("018f0000-0000-7000-8000-000000000001"), 1)].status,
        SourceStatus::Active
    );
    assert_eq!(
        repo.sources[&(id("018f0000-0000-7000-8000-000000000001"), 2)].status,
        SourceStatus::RolledBack
    );
}

#[test]
fn every_injected_failure_is_consumed_and_retry_is_safe() {
    for stage in 1..=4 {
        let mut repo = InMemoryKnowledgeRepository {
            fail_after: Some(stage),
            ..Default::default()
        };
        let (s, a, j, cs) = staged_version(1, 40 + stage as u64, b"retry\n");
        assert_eq!(
            repo.commit(s.clone(), a.clone(), j.clone(), cs.clone()),
            Err(KnowledgeError::PersistenceFailure)
        );
        assert!(
            repo.sources.is_empty()
                && repo.artifacts.is_empty()
                && repo.jobs.is_empty()
                && repo.chunks.is_empty()
        );
        assert_eq!(repo.fail_after, None);
        assert_eq!(repo.commit(s, a, j, cs), Ok(CommitOutcome::Inserted));
    }
}
