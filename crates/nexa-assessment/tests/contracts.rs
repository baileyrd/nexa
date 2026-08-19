use nexa_assessment::*;
use nexa_domain::*;
use std::collections::BTreeMap;
use std::str::FromStr;

fn id<T: FromStr>(n: u128) -> T
where
    T::Err: std::fmt::Debug,
{
    T::from_str(&uuid(n)).unwrap()
}
fn uuid(n: u128) -> String {
    format!("00000000-0000-0000-0000-{n:012x}")
}
fn ts(s: &str) -> Timestamp {
    s.parse().unwrap()
}
fn key(s: &str) -> SemanticKey {
    s.parse().unwrap()
}
fn score(v: f64) -> Score {
    Score::new(v).unwrap()
}
fn q(n: u128, evaluation: Evaluation, competencies: Vec<CompetencyId>) -> Question {
    Question {
        id: id(n),
        version: ProtocolVersion::new(1, 0),
        prompt: format!("Question {n}"),
        purpose: QuestionPurpose::Recall,
        competency_ids: competencies,
        evaluation,
    }
}
fn assessment(questions: Vec<Question>, rubrics: Vec<Rubric>) -> Assessment {
    Assessment::new(
        id(1),
        ProtocolVersion::new(1, 0),
        "Checkpoint",
        AssessmentMode::Formative,
        SCORING_POLICY_V1,
        score(0.6),
        questions,
        rubrics,
    )
    .unwrap()
}
fn started(a: &Assessment) -> AssessmentAttempt {
    let ids = a
        .questions()
        .iter()
        .map(|q| (q.id, id(100 + q.id.as_uuid().as_u128())))
        .collect();
    let x = ScoringPolicyV1
        .start(a, id(2), id(3), ids, ts("2026-08-19T10:00:00Z"))
        .unwrap();
    ScoringPolicyV1
        .transition(a, &x, AttemptState::Active, ts("2026-08-19T10:00:01Z"))
        .unwrap()
}
fn submit(
    a: &Assessment,
    attempt: &AssessmentAttempt,
    n: u128,
    value: ResponseValue,
    at: &str,
) -> Submission {
    let q = &a.questions()[(n - 1) as usize];
    let evidence = q
        .competency_ids
        .iter()
        .enumerate()
        .map(|(i, c)| (*c, id(500 + n * 10 + i as u128)))
        .collect();
    ScoringPolicyV1
        .submit(
            a,
            attempt,
            AssessmentResponse {
                id: id(200 + n),
                student_id: attempt.student_id,
                assessment_id: attempt.assessment_id,
                question_id: q.id,
                value,
                submitted_at: ts(at),
            },
            evidence,
        )
        .unwrap()
}

#[test]
fn golden_assessment_round_trip_is_canonical() {
    let a = assessment(
        vec![
            q(2, Evaluation::Boolean { correct: true }, vec![id(12)]),
            q(
                1,
                Evaluation::Choice {
                    correct: key("server"),
                },
                vec![id(11)],
            ),
        ],
        vec![],
    );
    let json = serde_json::to_string_pretty(&a).unwrap() + "\n";
    assert_eq!(json, include_str!("fixtures/assessment.json"));
    assert_eq!(serde_json::from_str::<Assessment>(&json).unwrap(), a);
}
#[test]
fn malformed_wire_is_rejected() {
    let fixture = include_str!("fixtures/assessment.json");
    for bad in [
        fixture.replace("0.6", "1.1"),
        fixture.replace("Question 1", ""),
        fixture.replace("1.0\",\n  \"passing", "9.0\",\n  \"passing"),
    ] {
        assert!(serde_json::from_str::<Assessment>(&bad).is_err());
    }
}
#[test]
fn duplicate_dangling_and_mapping_contracts_are_rejected() {
    let c = id(9);
    let one = q(1, Evaluation::Boolean { correct: true }, vec![c]);
    assert!(Assessment::new(
        id(1),
        ProtocolVersion::new(1, 0),
        "x",
        AssessmentMode::Practice,
        SCORING_POLICY_V1,
        score(0.5),
        vec![one.clone(), one],
        vec![]
    )
    .is_err());
    assert!(Assessment::new(
        id(1),
        ProtocolVersion::new(1, 0),
        "x",
        AssessmentMode::Practice,
        SCORING_POLICY_V1,
        score(0.5),
        vec![q(1, Evaluation::Rubric { rubric_id: id(8) }, vec![c])],
        vec![]
    )
    .is_err());
    assert!(Rubric::new(
        id(8),
        vec![RubricCriterion {
            id: id(7),
            description: "criterion".into(),
            weight: score(0.9),
            competency_id: c
        }]
    )
    .is_err());

    let uncovered = Rubric::new(
        id(8),
        vec![RubricCriterion {
            id: id(7),
            description: "criterion".into(),
            weight: score(1.0),
            competency_id: c,
        }],
    )
    .unwrap();
    assert!(Assessment::new(
        id(1),
        ProtocolVersion::new(1, 0),
        "x",
        AssessmentMode::Practice,
        SCORING_POLICY_V1,
        score(0.5),
        vec![q(
            1,
            Evaluation::Rubric { rubric_id: id(8) },
            vec![c, id(10)],
        )],
        vec![uncovered],
    )
    .is_err());
}
#[test]
fn score_boundaries_and_exact_evaluation() {
    for (value, ok) in [
        (0.0, true),
        (1.0, true),
        (-0.01, false),
        (1.01, false),
        (f64::NAN, false),
    ] {
        assert_eq!(Score::new(value).is_ok(), ok)
    }
    let a = assessment(
        vec![q(
            1,
            Evaluation::Exact {
                expected: " SYN-ACK ".into(),
                case_sensitive: false,
            },
            vec![id(8)],
        )],
        vec![],
    );
    let x = started(&a);
    assert_eq!(
        submit(
            &a,
            &x,
            1,
            ResponseValue::Text {
                value: "syn-ack".into()
            },
            "2026-08-19T10:00:02Z"
        )
        .result
        .outcome,
        EvaluationOutcome::Correct
    );
    let x = started(&a);
    assert_eq!(
        submit(
            &a,
            &x,
            1,
            ResponseValue::Text {
                value: "ACK".into()
            },
            "2026-08-19T10:00:02Z"
        )
        .result
        .outcome,
        EvaluationOutcome::Incorrect
    );
}
#[test]
fn rubric_weights_and_multi_question_scores_aggregate() {
    let c1 = id(11);
    let c2 = id(12);
    let rubric = Rubric::new(
        id(20),
        vec![
            RubricCriterion {
                id: id(21),
                description: "accuracy".into(),
                weight: score(0.25),
                competency_id: c1,
            },
            RubricCriterion {
                id: id(22),
                description: "reasoning".into(),
                weight: score(0.75),
                competency_id: c2,
            },
        ],
    )
    .unwrap();
    let a = assessment(
        vec![
            q(
                1,
                Evaluation::Rubric {
                    rubric_id: rubric.id,
                },
                vec![c2, c1],
            ),
            q(2, Evaluation::Boolean { correct: true }, vec![c1]),
        ],
        vec![rubric],
    );
    let x = started(&a);
    let s = submit(
        &a,
        &x,
        1,
        ResponseValue::Rubric {
            levels: BTreeMap::from([(id(21), score(1.0)), (id(22), score(0.5))]),
        },
        "2026-08-19T10:00:02Z",
    );
    assert_eq!(s.result.score.get(), 0.625);
    assert_eq!(s.evidence.len(), 2);
    assert_eq!(s.evidence[0].competency_id, c1);
    assert_eq!(
        s.evidence[0].outcome,
        nexa_student::EvidenceOutcome::Success
    );
    assert_eq!(s.evidence[1].competency_id, c2);
    assert_eq!(
        s.evidence[1].outcome,
        nexa_student::EvidenceOutcome::PartialSuccess
    );
    let s2 = submit(
        &a,
        &s.attempt,
        2,
        ResponseValue::Boolean { value: true },
        "2026-08-19T10:00:03Z",
    );
    let sub = ScoringPolicyV1
        .transition(
            &a,
            &s2.attempt,
            AttemptState::Submitted,
            ts("2026-08-19T10:00:04Z"),
        )
        .unwrap();
    let done = ScoringPolicyV1
        .transition(
            &a,
            &sub,
            AttemptState::Completed,
            ts("2026-08-19T10:00:05Z"),
        )
        .unwrap();
    let out = ScoringPolicyV1.result(&a, &done).unwrap();
    assert_eq!(out.score.get(), 0.8125);
    assert!(out.passed);
}
#[test]
fn deterministic_ordering_and_replay() {
    let a = assessment(
        vec![
            q(2, Evaluation::Boolean { correct: true }, vec![id(12)]),
            q(1, Evaluation::Boolean { correct: true }, vec![id(11)]),
        ],
        vec![],
    );
    assert_eq!(
        a.questions().iter().map(|q| q.id).collect::<Vec<_>>(),
        vec![id(1), id(2)]
    );
    let x = started(&a);
    let response = AssessmentResponse {
        id: id(201),
        student_id: x.student_id,
        assessment_id: x.assessment_id,
        question_id: id(1),
        value: ResponseValue::Boolean { value: true },
        submitted_at: ts("2026-08-19T10:00:02Z"),
    };
    let evidence = BTreeMap::from([(id(11), id(511))]);
    let first = ScoringPolicyV1
        .submit(&a, &x, response.clone(), evidence.clone())
        .unwrap();
    let replay = ScoringPolicyV1
        .submit(&a, &first.attempt, response, evidence)
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.attempt, first.attempt);
    assert!(replay.evidence.is_empty());
}

#[test]
fn out_of_order_answers_remain_canonical_and_delayed_replays_are_noops() {
    let a = assessment(
        vec![
            q(1, Evaluation::Boolean { correct: true }, vec![id(11)]),
            q(2, Evaluation::Boolean { correct: true }, vec![id(12)]),
        ],
        vec![],
    );
    let x = started(&a);
    let second = submit(
        &a,
        &x,
        2,
        ResponseValue::Boolean { value: true },
        "2026-08-19T10:00:03Z",
    );
    let first = submit(
        &a,
        &second.attempt,
        1,
        ResponseValue::Boolean { value: true },
        "2026-08-19T10:00:04Z",
    );
    assert_eq!(
        first
            .attempt
            .responses()
            .iter()
            .map(|r| r.question_id)
            .collect::<Vec<_>>(),
        vec![id(1), id(2)]
    );
    let restored: AssessmentAttempt =
        serde_json::from_value(serde_json::to_value(&first.attempt).unwrap()).unwrap();
    let old_response = second.attempt.responses()[0].clone();
    let replay = ScoringPolicyV1
        .submit(
            &a,
            &restored,
            old_response,
            BTreeMap::from([(id(12), id(522))]),
        )
        .unwrap();
    assert!(replay.replayed);
}

#[test]
fn duplicate_instances_and_malformed_imported_coverage_are_rejected() {
    let a = assessment(
        vec![
            q(1, Evaluation::Boolean { correct: true }, vec![id(11)]),
            q(2, Evaluation::Boolean { correct: true }, vec![id(12)]),
        ],
        vec![],
    );
    assert!(ScoringPolicyV1
        .start(
            &a,
            id(2),
            id(3),
            BTreeMap::from([(id(1), id(101)), (id(2), id(101))]),
            ts("2026-08-19T10:00:00Z")
        )
        .is_err());
    let x = started(&a);
    let mut wire = serde_json::to_value(&x).unwrap();
    wire["items"][0]["question_id"] = serde_json::to_value(id::<QuestionId>(99)).unwrap();
    assert!(serde_json::from_value::<AssessmentAttempt>(wire).is_err());

    let mut wrong_version = x.clone();
    let mut wire = serde_json::to_value(&wrong_version).unwrap();
    wire["items"][0]["question_version"] = "2.0".into();
    wrong_version = serde_json::from_value(wire).unwrap();
    assert!(ScoringPolicyV1
        .submit(
            &a,
            &wrong_version,
            AssessmentResponse {
                id: id(201),
                student_id: x.student_id,
                assessment_id: x.assessment_id,
                question_id: id(1),
                value: ResponseValue::Boolean { value: true },
                submitted_at: ts("2026-08-19T10:00:02Z")
            },
            BTreeMap::from([(id(11), id(511))])
        )
        .is_err());
}
#[test]
fn lifecycle_terminal_and_timestamp_rules_are_atomic() {
    let a = assessment(
        vec![q(1, Evaluation::Boolean { correct: true }, vec![id(11)])],
        vec![],
    );
    let x = started(&a);
    let before = x.clone();
    assert_eq!(
        ScoringPolicyV1.transition(&a, &x, AttemptState::Completed, ts("2026-08-19T10:00:02Z")),
        Err(AssessmentError::InvalidState)
    );
    assert_eq!(x, before);
    assert_eq!(
        ScoringPolicyV1.transition(&a, &x, AttemptState::Paused, ts("2026-08-19T09:00:00Z")),
        Err(AssessmentError::TimestampRegression)
    );
    assert_eq!(x, before);
    let cancelled = ScoringPolicyV1
        .transition(&a, &x, AttemptState::Cancelled, ts("2026-08-19T10:00:02Z"))
        .unwrap();
    assert_eq!(
        ScoringPolicyV1.transition(
            &a,
            &cancelled,
            AttemptState::Active,
            ts("2026-08-19T10:00:03Z")
        ),
        Err(AssessmentError::InvalidState)
    );
}
#[test]
fn cross_scope_policy_and_conflict_are_rejected_without_mutation() {
    let a = assessment(
        vec![q(1, Evaluation::Boolean { correct: true }, vec![id(11)])],
        vec![],
    );
    let x = started(&a);
    let before = x.clone();
    let other = Assessment::new(
        id(99),
        a.version,
        "other",
        a.mode,
        SCORING_POLICY_V1,
        score(0.5),
        a.questions().to_vec(),
        vec![],
    )
    .unwrap();
    let response = AssessmentResponse {
        id: id(201),
        student_id: x.student_id,
        assessment_id: x.assessment_id,
        question_id: id(1),
        value: ResponseValue::Boolean { value: true },
        submitted_at: ts("2026-08-19T10:00:02Z"),
    };
    assert_eq!(
        ScoringPolicyV1.submit(
            &other,
            &x,
            response.clone(),
            BTreeMap::from([(id(11), id(511))])
        ),
        Err(AssessmentError::ScopeMismatch)
    );
    assert_eq!(x, before);
    let cross_student = AssessmentResponse {
        student_id: id(98),
        ..response.clone()
    };
    assert_eq!(
        ScoringPolicyV1.submit(&a, &x, cross_student, BTreeMap::from([(id(11), id(511))])),
        Err(AssessmentError::ScopeMismatch)
    );
    let mut wire = serde_json::to_value(&x).unwrap();
    wire["policy_version"] = "2.0".into();
    let mismatch: AssessmentAttempt = serde_json::from_value(wire).unwrap();
    assert!(matches!(
        ScoringPolicyV1.submit(
            &a,
            &mismatch,
            response.clone(),
            BTreeMap::from([(id(11), id(511))])
        ),
        Err(AssessmentError::PolicyMismatch { .. })
    ));
    let first = ScoringPolicyV1
        .submit(
            &a,
            &x,
            response.clone(),
            BTreeMap::from([(id(11), id(511))]),
        )
        .unwrap();
    let conflict = AssessmentResponse {
        value: ResponseValue::Boolean { value: false },
        ..response
    };
    assert_eq!(
        ScoringPolicyV1.submit(
            &a,
            &first.attempt,
            conflict,
            BTreeMap::from([(id(11), id(511))])
        ),
        Err(AssessmentError::ConflictingReplay)
    );
    assert_eq!(first.attempt.responses().len(), 1);
}
#[test]
fn evidence_is_privacy_minimal_and_ledger_compatible() {
    use nexa_student::{AppendOutcome, EvidenceRepository, InMemoryEvidenceRepository};
    let a = assessment(
        vec![q(1, Evaluation::Boolean { correct: true }, vec![id(11)])],
        vec![],
    );
    let x = started(&a);
    let s = submit(
        &a,
        &x,
        1,
        ResponseValue::Boolean { value: true },
        "2026-08-19T10:00:02Z",
    );
    assert_eq!(s.evidence[0].student_id, x.student_id);
    assert_eq!(
        s.evidence[0].source,
        nexa_student::EvidenceSource::Assessment(x.id)
    );
    let mut ledger = InMemoryEvidenceRepository::default();
    assert_eq!(
        ledger.append(s.evidence[0].clone()).unwrap(),
        AppendOutcome::Appended
    );
    assert_eq!(
        ledger.append(s.evidence[0].clone()).unwrap(),
        AppendOutcome::Duplicate
    );
    let mut conflicting = s.evidence[0].clone();
    conflicting.outcome = nexa_student::EvidenceOutcome::Failure;
    assert!(ledger.append(conflicting).is_err());
}
