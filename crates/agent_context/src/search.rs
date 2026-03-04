use std::collections::BTreeSet;

use crate::SearchIndexEntry;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchStatus {
    Ok,
    LowConfidence,
    NoMatch,
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchMatch {
    pub node_id: String,
    pub score: f32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub match_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchResult {
    pub status: SearchStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub matches: Vec<SearchMatch>,
}

pub fn normalize_tokens(input: &str) -> Vec<String> {
    input
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_lowercase())
        .collect()
}

pub fn classify_status(score: f32) -> SearchStatus {
    if score >= 0.72 {
        SearchStatus::Ok
    } else if score >= 0.45 {
        SearchStatus::LowConfidence
    } else {
        SearchStatus::NoMatch
    }
}

pub fn rank_candidates(query: &str, entries: &[SearchIndexEntry], top_k: usize) -> SearchResult {
    if top_k == 0 {
        return SearchResult {
            status: SearchStatus::NoMatch,
            matches: Vec::new(),
        };
    }

    let query_tokens = normalize_tokens(query);
    if query_tokens.is_empty() {
        return SearchResult {
            status: SearchStatus::NoMatch,
            matches: Vec::new(),
        };
    }

    let mut matches = entries
        .iter()
        .filter_map(|entry| score_entry(query_tokens.as_slice(), entry))
        .collect::<Vec<_>>();

    matches.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.node_id.cmp(&right.node_id))
    });
    matches.truncate(top_k);

    if matches.is_empty() {
        return SearchResult {
            status: SearchStatus::NoMatch,
            matches,
        };
    }

    let status = if is_ambiguous(matches.as_slice()) {
        SearchStatus::Ambiguous
    } else {
        classify_status(matches[0].score)
    };

    SearchResult { status, matches }
}

fn score_entry(query_tokens: &[String], entry: &SearchIndexEntry) -> Option<SearchMatch> {
    let mut reasons = Vec::new();
    let mut score = 0.0f32;

    let searchable_text = build_searchable_text_token_set(entry);
    let token_overlap_ratio = overlap_ratio(query_tokens, searchable_text.as_slice());
    if token_overlap_ratio > 0.0 {
        score += token_overlap_ratio * 0.45;
        reasons.push("text_token".to_string());
    }

    let alias_score = alias_match_score(query_tokens, entry.aliases.as_slice());
    if alias_score > 0.0 {
        score += alias_score * 0.20;
        reasons.push("name_alias".to_string());
    }

    let path_tokens = normalize_tokens(entry.path.as_str());
    let path_ratio = overlap_ratio(query_tokens, path_tokens.as_slice());
    if path_ratio > 0.0 {
        score += path_ratio * 0.20;
        reasons.push("path_match".to_string());
    }

    let geometry_tokens = entry
        .geometry_tags
        .iter()
        .flat_map(|tag| normalize_tokens(tag))
        .collect::<Vec<_>>();
    let geometry_ratio = overlap_ratio(query_tokens, geometry_tokens.as_slice());
    if geometry_ratio > 0.0 {
        score += geometry_ratio * 0.15;
        reasons.push("geometry_hint".to_string());
    }

    if score <= 0.0 {
        return None;
    }

    Some(SearchMatch {
        node_id: entry.node_id.clone(),
        score,
        match_reasons: reasons,
    })
}

fn build_searchable_text_token_set(entry: &SearchIndexEntry) -> Vec<String> {
    let mut token_set = BTreeSet::new();
    for token in &entry.normalized_tokens {
        if !token.is_empty() {
            token_set.insert(token.to_lowercase());
        }
    }
    for token in &entry.raw_tokens {
        for normalized in normalize_tokens(token) {
            token_set.insert(normalized);
        }
    }
    for normalized in normalize_tokens(entry.name.as_str()) {
        token_set.insert(normalized);
    }
    token_set.into_iter().collect()
}

fn overlap_ratio(query_tokens: &[String], candidate_tokens: &[String]) -> f32 {
    if query_tokens.is_empty() || candidate_tokens.is_empty() {
        return 0.0;
    }

    let candidate_set = candidate_tokens
        .iter()
        .map(|token| token.as_str())
        .collect::<BTreeSet<_>>();
    let overlap_count = query_tokens
        .iter()
        .map(|token| token.as_str())
        .filter(|token| candidate_set.contains(token))
        .count();

    overlap_count as f32 / query_tokens.len() as f32
}

fn alias_match_score(query_tokens: &[String], aliases: &[String]) -> f32 {
    if aliases.is_empty() {
        return 0.0;
    }

    let alias_tokens = aliases
        .iter()
        .flat_map(|alias| normalize_tokens(alias))
        .collect::<Vec<_>>();
    overlap_ratio(query_tokens, alias_tokens.as_slice())
}

fn is_ambiguous(matches: &[SearchMatch]) -> bool {
    if matches.len() < 2 {
        return false;
    }

    let first = matches[0].score;
    let second = matches[1].score;
    first >= 0.45 && (first - second).abs() <= 0.03
}

#[cfg(test)]
mod tests {
    use crate::SearchIndexEntry;

    use super::*;

    #[test]
    fn normalize_tokens_lowercases_and_strips_punctuation() {
        assert_eq!(normalize_tokens("Welcome, Back!"), vec!["welcome", "back"]);
    }

    #[test]
    fn rank_candidates_is_stable_with_tie_break_on_node_id() {
        let results = rank_candidates("title", sample_entries().as_slice(), 5);
        assert_eq!(results.matches[0].node_id, "1:10");
        assert_eq!(results.matches[1].node_id, "1:11");
    }

    #[test]
    fn rank_candidates_marks_low_confidence_and_no_match_thresholds() {
        assert_eq!(classify_status(0.50), SearchStatus::LowConfidence);
        assert_eq!(classify_status(0.30), SearchStatus::NoMatch);
    }

    fn sample_entries() -> Vec<SearchIndexEntry> {
        vec![
            SearchIndexEntry {
                node_id: "1:11".to_string(),
                name: "Title Secondary".to_string(),
                node_type: "TEXT".to_string(),
                path: "Main/Header/Secondary".to_string(),
                raw_tokens: vec!["Title".to_string(), "Secondary".to_string()],
                normalized_tokens: vec!["title".to_string(), "secondary".to_string()],
                aliases: vec!["headline".to_string()],
                geometry_tags: vec!["header".to_string()],
            },
            SearchIndexEntry {
                node_id: "1:10".to_string(),
                name: "Title Primary".to_string(),
                node_type: "TEXT".to_string(),
                path: "Main/Header/Primary".to_string(),
                raw_tokens: vec!["Title".to_string(), "Primary".to_string()],
                normalized_tokens: vec!["title".to_string(), "primary".to_string()],
                aliases: vec!["title".to_string()],
                geometry_tags: vec!["header".to_string()],
            },
        ]
    }
}
