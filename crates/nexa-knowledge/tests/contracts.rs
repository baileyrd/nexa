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
