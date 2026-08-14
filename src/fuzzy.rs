#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FuzzyMatch {
    pub score: i32,
    pub matched_indices: Vec<usize>,
}

pub fn fuzzy_match(pattern: &str, target: &str) -> Option<FuzzyMatch> {
    if pattern.is_empty() {
        return Some(FuzzyMatch {
            score: 0,
            matched_indices: Vec::new(),
        });
    }

    let pattern_chars: Vec<char> = pattern.to_lowercase().chars().collect();
    let target_chars: Vec<char> = target.chars().collect();
    let target_lower: Vec<char> = target.to_lowercase().chars().collect();

    let mut matched_indices = Vec::with_capacity(pattern_chars.len());
    let mut pattern_idx = 0;
    let mut score: i32 = 0;
    let mut last_match_idx: Option<usize> = None;

    for (t_idx, &t_char) in target_lower.iter().enumerate() {
        if pattern_idx < pattern_chars.len() && t_char == pattern_chars[pattern_idx] {
            matched_indices.push(t_idx);

            // Base match score
            score += 10;

            // Bonus for consecutive matches
            if let Some(prev_idx) = last_match_idx {
                if t_idx == prev_idx + 1 {
                    score += 20;
                }
            }

            // Bonus for word boundaries (e.g. after '_', '-', '.', or CamelCase)
            if t_idx == 0 {
                score += 30; // Start of string
            } else {
                let prev_char = target_chars[t_idx - 1];
                if prev_char == '_'
                    || prev_char == '-'
                    || prev_char == '.'
                    || prev_char == '/'
                    || prev_char == ' '
                {
                    score += 25;
                } else if prev_char.is_lowercase() && target_chars[t_idx].is_uppercase() {
                    score += 20; // CamelCase boundary
                }
            }

            last_match_idx = Some(t_idx);
            pattern_idx += 1;
        }
    }

    if pattern_idx == pattern_chars.len() {
        // Penalty for length difference
        let len_diff = (target_chars.len() as i32) - (pattern_chars.len() as i32);
        score -= len_diff.min(50);
        Some(FuzzyMatch {
            score,
            matched_indices,
        })
    } else {
        None
    }
}
