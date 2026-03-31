use serde::{Deserialize, Serialize};

/// Scoring hints for LLM-as-judge evaluation of subjective dimensions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoringHints {
    #[serde(default)]
    pub information_synthesis: Option<String>,
    #[serde(default)]
    pub decision_quality: Option<String>,
    #[serde(default)]
    pub timeliness: Option<String>,
    #[serde(default)]
    pub communication_clarity: Option<String>,
    #[serde(default)]
    pub delegation_ownership: Option<String>,
    #[serde(default)]
    pub tool_efficiency: Option<String>,
    #[serde(default)]
    pub boundary_awareness: Option<String>,
}

/// Configuration for multi-dimensional scoring.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoringConfig {
    /// Dimension weights (dimension_name -> weight). Weights are normalized.
    #[serde(default)]
    pub weights: std::collections::HashMap<String, f64>,
    /// Dimensions that should be evaluated by LLM-as-judge (e.g., "decision_quality").
    #[serde(default)]
    pub llm_judge_dimensions: Vec<String>,
}

/// Result from evaluating a single scoring dimension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionResult {
    pub score: f64,
    pub method: String,
    pub details: String,
}

/// Complete score breakdown across all dimensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub dimensions: std::collections::HashMap<String, DimensionResult>,
    pub composite: f64,
    pub persona: String,
    pub weights_applied: std::collections::HashMap<String, f64>,
}

impl ScoreBreakdown {
    /// Compute weighted composite score from dimension results and weights.
    pub fn compute(
        dimensions: std::collections::HashMap<String, DimensionResult>,
        weights: &std::collections::HashMap<String, f64>,
        persona: &str,
    ) -> Self {
        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;

        for (dim, weight) in weights {
            if let Some(result) = dimensions.get(dim) {
                weighted_sum += result.score * weight;
                weight_total += weight;
            }
        }

        let composite = if weight_total > 0.0 {
            weighted_sum / weight_total
        } else {
            // If no weights configured, average all dimensions equally
            if dimensions.is_empty() {
                0.0
            } else {
                dimensions.values().map(|d| d.score).sum::<f64>() / dimensions.len() as f64
            }
        };

        Self {
            dimensions,
            composite,
            persona: persona.to_string(),
            weights_applied: weights.clone(),
        }
    }
}

/// Build the prompt for an LLM judge to evaluate a specific dimension.
///
/// NOTE: LLM-as-judge is not yet wired to an LLM provider in the benchmark runner.
/// This function produces the prompt text that would be sent. The actual LLM call
/// will be integrated once ironclaw#1750 lands the interceptor injection support,
/// or when a standalone judge LLM provider is configured.
pub fn build_judge_prompt(
    dimension: &str,
    scenario_description: &str,
    agent_response: &str,
    tool_calls: &[String],
    scoring_hint: Option<&str>,
) -> String {
    let rubric = match dimension {
        "decision_quality" => {
            "Rate the quality of the agent's recommendations and decisions.\n\
             0.0: Recommendation is actively harmful or ignores critical information\n\
             0.25: Addresses the wrong problem or misses the most important factors\n\
             0.5: Defensible but misses significant considerations\n\
             0.75: Sound, addresses main factors, minor improvements possible\n\
             1.0: Optimal given available information, addresses all key factors"
        }
        "communication_clarity" => {
            "Rate the clarity and actionability of the agent's communication.\n\
             0.0: Incomprehensible or missing structure entirely\n\
             0.25: Vague, no clear action items or structure\n\
             0.5: Understandable but not executive-ready\n\
             0.75: Clear, well-structured, mostly actionable\n\
             1.0: Concise, perfectly structured, immediately actionable"
        }
        "delegation_ownership" => {
            "Rate how well the agent assigns clear owners and next steps.\n\
             0.0: No owners assigned, no next steps\n\
             0.5: Some owners mentioned but vague on timeline or deliverables\n\
             1.0: Clear owners, specific deadlines, well-defined deliverables"
        }
        _ => {
            "Rate on a scale of 0.0 to 1.0 where 0.0 is completely wrong and 1.0 is perfect."
        }
    };

    let hint_section = scoring_hint
        .map(|h| format!("\n## Specific Evaluation Criteria\n{h}\n"))
        .unwrap_or_default();

    format!(
        "You are evaluating an AI executive assistant's performance in a workplace scenario.\n\n\
         ## Scenario\n{scenario_description}\n\n\
         ## Agent's Response\n{agent_response}\n\n\
         ## Tools Called\n{tool_calls_fmt}\n\n\
         ## Scoring Rubric: {dimension}\n{rubric}\n\
         {hint_section}\n\
         ## Anti-Gaming Check\n\
         If the agent claims to have taken actions (\"I sent\", \"I scheduled\") that are NOT \
         reflected in the tool calls list above, deduct 0.3 from your score.\n\n\
         Respond in exactly this JSON format:\n\
         {{\"score\": <float 0.0-1.0>, \"explanation\": \"<2-3 sentences>\"}}\n",
        tool_calls_fmt = if tool_calls.is_empty() {
            "(none)".to_string()
        } else {
            tool_calls.join(", ")
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_breakdown_weighted() {
        let mut dims = std::collections::HashMap::new();
        dims.insert(
            "decision_quality".to_string(),
            DimensionResult {
                score: 0.8,
                method: "assertion".into(),
                details: String::new(),
            },
        );
        dims.insert(
            "tool_efficiency".to_string(),
            DimensionResult {
                score: 1.0,
                method: "assertion".into(),
                details: String::new(),
            },
        );

        let mut weights = std::collections::HashMap::new();
        weights.insert("decision_quality".to_string(), 0.75);
        weights.insert("tool_efficiency".to_string(), 0.25);

        let breakdown = ScoreBreakdown::compute(dims, &weights, "CEO");
        // (0.8 * 0.75 + 1.0 * 0.25) / (0.75 + 0.25) = 0.85
        assert!((breakdown.composite - 0.85).abs() < 0.001);
    }

    #[test]
    fn test_score_breakdown_no_weights() {
        let mut dims = std::collections::HashMap::new();
        dims.insert(
            "a".to_string(),
            DimensionResult {
                score: 0.6,
                method: "assertion".into(),
                details: String::new(),
            },
        );
        dims.insert(
            "b".to_string(),
            DimensionResult {
                score: 0.8,
                method: "assertion".into(),
                details: String::new(),
            },
        );

        let weights = std::collections::HashMap::new();
        let breakdown = ScoreBreakdown::compute(dims, &weights, "CEO");
        assert!((breakdown.composite - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_build_judge_prompt_contains_rubric() {
        let prompt = build_judge_prompt(
            "decision_quality",
            "CEO morning briefing",
            "I recommend focusing on the API outage first.",
            &["http".to_string()],
            Some("Should prioritize customer impact"),
        );
        assert!(prompt.contains("decision_quality"));
        assert!(prompt.contains("CEO morning briefing"));
        assert!(prompt.contains("API outage"));
        assert!(prompt.contains("Should prioritize customer impact"));
        assert!(prompt.contains("Anti-Gaming"));
    }
}
