//! Prior-art core types (T-018, MASTER_SPEC section 10).
//!
//! Atomic claims, the closed five-conclusion enum, passage references
//! verified against captured bytes (R-107), and confidentiality-aware
//! redacted queries (section 10:205). Pure logic: no I/O, no retrieval.

pub mod record;

pub use record::{
    AtomicClaim, ChartRow, ClaimChart, Conclusion, DisclosureGate, PassageRef, PriorArtReport,
    RediscoveryLabel, ReportMeta, SearchPlan,
};

/// Verify a passage reference against corpus bytes (R-107): the captured
/// text must exactly match the bytes at `[start, end)` of the named
/// source. Mutable-locator-only references (empty captured text) are
/// rejected outright.
pub fn verify_span(reference: &PassageRef, corpora: &[(String, Vec<u8>)]) -> bool {
    if reference.captured_text.is_empty() {
        return false;
    }
    corpora
        .iter()
        .filter(|(name, _)| *name == reference.source)
        .any(|(_, bytes)| {
            reference.end <= bytes.len()
                && reference.start < reference.end
                && &bytes[reference.start..reference.end] == reference.captured_text.as_bytes()
        })
}

/// Derive redacted queries from claims: mechanism-specific tokens are
/// stripped (section 10:205). Tokens longer than 3 chars that appear in
/// the claim's mechanism field are removed from the query text.
pub fn redact_query(query: &str, mechanism_tokens: &[&str]) -> String {
    let lowered = query.to_lowercase();
    let kept: Vec<&str> = lowered
        .split_whitespace()
        .filter(|t| !mechanism_tokens.iter().any(|m| t.contains(m)))
        .collect();
    kept.join(" ")
}

/// Stage 1: fast keyword pass — does any corpus byte text contain the
/// claim's component AND mechanism phrases? A hit flags a rediscovery.
fn stage1_known(claim: &AtomicClaim, corpora: &[(String, Vec<u8>)]) -> bool {
    let component = claim.component.to_lowercase();
    let mechanism = claim.mechanism.to_lowercase();
    if component.is_empty() || mechanism.is_empty() {
        return false;
    }
    corpora.iter().any(|(_, bytes)| {
        let text = String::from_utf8_lossy(bytes).to_lowercase();
        text.contains(&component) && text.contains(&mechanism)
    })
}

/// Stage 2: claim-level comparison against the best near-match corpus.
/// Returns (conclusion, prior-art corpus name, passage) for the claim.
fn stage2_compare(
    claim: &AtomicClaim,
    corpora: &[(String, Vec<u8>)],
) -> (Conclusion, Option<(String, String, usize, usize)>) {
    let component = claim.component.to_lowercase();
    // Find a corpus mentioning the component (near-match candidate).
    let hit = corpora.iter().find_map(|(name, bytes)| {
        let text = String::from_utf8_lossy(bytes);
        let lower = text.to_lowercase();
        let idx = lower.find(&component)?;
        let end = (idx + component.len()).min(text.len());
        Some((name.clone(), text.to_string(), idx, end))
    });
    match hit {
        None => (Conclusion::NoMatchWithinSearchScope, None),
        Some((name, text, start, end)) => {
            // Field-level overlap: how many of the claim's fields appear
            // near the hit? Deterministic accounting, no scores.
            let lower = text.to_lowercase();
            let fields = [
                &claim.component,
                &claim.mechanism,
                &claim.use_context,
                &claim.claimed_result,
            ];
            let overlapping = fields
                .iter()
                .filter(|f| !f.is_empty() && lower.contains(&f.to_lowercase()))
                .count();
            let conclusion = if overlapping >= 3 {
                Conclusion::NearMatch
            } else {
                Conclusion::Unresolved
            };
            (
                conclusion,
                Some((name, text[start..end].to_string(), start, end)),
            )
        }
    }
}

/// Two-stage prior-art investigation (MASTER_SPEC §10:202): stage 1
/// catches clear rediscoveries; stage 2 runs claim-level comparison for
/// survivors. Conclusions are the closed five; no novelty language.
/// Deterministic: sorted claims, fixed iteration, twin-run identical.
pub fn investigate(
    claims: &[AtomicClaim],
    corpora: &[(String, Vec<u8>)],
    plan: &SearchPlan,
    search_date: &str,
) -> PriorArtReport {
    let mut conclusions = Vec::new();
    let mut charts = Vec::new();
    let mut near_matches = Vec::new();

    for claim in claims {
        if stage1_known(claim, corpora) {
            conclusions.push((claim.id.clone(), Conclusion::Known));
            continue;
        }
        let (conclusion, passage) = stage2_compare(claim, corpora);
        if let Some((source, captured, start, end)) = passage {
            near_matches.push(source.clone());
            let reference = PassageRef {
                source,
                start,
                end,
                captured_text: captured,
            };
            // Chart only when the span verifies (R-107).
            if verify_span(&reference, corpora)
                && let Some(chart) = build_chart(claim, reference)
            {
                charts.push(chart);
            }
        }
        conclusions.push((claim.id.clone(), conclusion));
    }

    conclusions.sort_by(|a, b| a.0.cmp(&b.0));
    charts.sort_by(|a, b| a.claim_id.cmp(&b.claim_id));
    near_matches.sort();
    near_matches.dedup();

    PriorArtReport {
        conclusions,
        charts,
        meta: ReportMeta {
            search_date: search_date.to_string(),
            scopes_covered: plan.scopes.clone(),
            query_families: plan.query_families.clone(),
            near_matches,
            // In-repo fixtures only: no external access attempted.
            missing_access: vec!["external-databases-not-authorized".to_string()],
        },
    }
}

/// Build a claim chart from a verified passage: per-field rows with
/// similarity accounts (§10:206). Differences assessed when any field
/// differs.
fn build_chart(claim: &AtomicClaim, passage: PassageRef) -> Option<ClaimChart> {
    let text = passage.captured_text.to_lowercase();
    let rows = [
        ("component", &claim.component),
        ("mechanism", &claim.mechanism),
        ("use_context", &claim.use_context),
        ("claimed_result", &claim.claimed_result),
    ]
    .into_iter()
    .map(|(field, value)| {
        let similar = !value.is_empty() && text.contains(&value.to_lowercase());
        let account = if similar {
            format!("prior-art passage contains the claimed {field}")
        } else {
            format!("prior-art passage does not state the claimed {field}: difference assessed")
        };
        ChartRow {
            field: field.to_string(),
            candidate_value: value.clone(),
            prior_art_value: passage.captured_text.clone(),
            passage: passage.clone(),
            similar,
            account,
        }
    })
    .collect::<Vec<_>>();
    let differences_assessed = rows.iter().any(|r| !r.similar);
    if rows.is_empty() {
        None
    } else {
        Some(ClaimChart {
            claim_id: claim.id.clone(),
            rows,
            differences_assessed,
        })
    }
}

/// Public claim-chart constructor with explicit span verification
/// (R-107): refuses to produce a chart unless the caller's captured text
/// matches the corpus bytes at the referenced range.
pub fn claim_chart(
    claim: &AtomicClaim,
    passage: PassageRef,
    corpora: &[(String, Vec<u8>)],
) -> Option<ClaimChart> {
    if !verify_span(&passage, corpora) {
        return None;
    }
    build_chart(claim, passage)
}

/// Rediscovery labeling (R-112): `Known` -> validated solution;
/// `NearMatch` -> validated candidate only with a verified chart whose
/// differences are assessed; anything else -> unlabeled.
pub fn label_for(
    conclusion: Conclusion,
    charts: &[ClaimChart],
    claim_id: &str,
) -> Option<RediscoveryLabel> {
    match conclusion {
        Conclusion::Known => Some(RediscoveryLabel::ValidatedSolution),
        Conclusion::NearMatch => {
            let chart = charts.iter().find(|c| c.claim_id == claim_id)?;
            if chart.differences_assessed {
                Some(RediscoveryLabel::ValidatedCandidate)
            } else {
                None
            }
        }
        _ => None,
    }
}
