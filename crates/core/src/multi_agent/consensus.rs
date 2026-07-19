use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Vote {
    Approve,
    Reject,
    Abstain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentVote {
    pub agent_id: String,
    pub vote: Vote,
    pub reasoning: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub proposal_id: String,
    pub votes: Vec<AgentVote>,
    pub passed: bool,
    pub approval_rate: f64,
    pub consensus_reached: bool,
    pub final_decision: String,
}

pub struct ConsensusEngine {
    threshold: f64,
}

impl ConsensusEngine {
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }

    pub fn evaluate(&self, proposal_id: &str, votes: &[AgentVote]) -> ConsensusResult {
        if votes.is_empty() {
            return ConsensusResult {
                proposal_id: proposal_id.to_string(),
                votes: vec![],
                passed: false,
                approval_rate: 0.0,
                consensus_reached: false,
                final_decision: "No votes cast".to_string(),
            };
        }

        let approve_count = votes.iter().filter(|v| v.vote == Vote::Approve).count() as f64;
        let total_votes = votes.len() as f64;
        let approval_rate = approve_count / total_votes;

        let consensus_reached = approval_rate >= self.threshold;
        let passed = consensus_reached;

        let final_decision = if passed {
            "Approved by consensus".to_string()
        } else if approval_rate > 0.5 {
            "Majority but no consensus".to_string()
        } else {
            "Rejected".to_string()
        };

        ConsensusResult {
            proposal_id: proposal_id.to_string(),
            votes: votes.to_vec(),
            passed,
            approval_rate,
            consensus_reached,
            final_decision,
        }
    }

    pub fn calculate_weighted_score(&self, votes: &[AgentVote]) -> f64 {
        if votes.is_empty() {
            return 0.0;
        }

        let total_weight: f64 = votes.iter().map(|v| v.confidence).sum();
        if total_weight == 0.0 {
            return 0.0;
        }

        let weighted_sum: f64 = votes.iter().map(|v| {
            let vote_value = match v.vote {
                Vote::Approve => 1.0,
                Vote::Reject => -1.0,
                Vote::Abstain => 0.0,
            };
            vote_value * v.confidence
        }).sum();

        weighted_sum / total_weight
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vote(agent_id: &str, vote: Vote, confidence: f64) -> AgentVote {
        AgentVote {
            agent_id: agent_id.to_string(),
            vote,
            reasoning: "test".to_string(),
            confidence,
        }
    }

    #[test]
    fn test_consensus_approve() {
        let engine = ConsensusEngine::new(0.6);
        let votes = vec![
            make_vote("a1", Vote::Approve, 0.9),
            make_vote("a2", Vote::Approve, 0.8),
            make_vote("a3", Vote::Approve, 0.7),
        ];
        let result = engine.evaluate("p1", &votes);
        assert!(result.passed);
        assert!(result.consensus_reached);
    }

    #[test]
    fn test_consensus_reject() {
        let engine = ConsensusEngine::new(0.6);
        let votes = vec![
            make_vote("a1", Vote::Approve, 0.9),
            make_vote("a2", Vote::Reject, 0.8),
            make_vote("a3", Vote::Reject, 0.7),
        ];
        let result = engine.evaluate("p1", &votes);
        assert!(!result.passed);
        assert!(!result.consensus_reached);
    }

    #[test]
    fn test_consensus_no_votes() {
        let engine = ConsensusEngine::new(0.6);
        let result = engine.evaluate("p1", &[]);
        assert!(!result.passed);
        assert_eq!(result.votes.len(), 0);
    }

    #[test]
    fn test_weighted_score() {
        let engine = ConsensusEngine::new(0.6);
        let votes = vec![
            make_vote("a1", Vote::Approve, 1.0),
            make_vote("a2", Vote::Approve, 0.5),
        ];
        let score = engine.calculate_weighted_score(&votes);
        assert!(score > 0.0);
    }

    #[test]
    fn test_weighted_score_mixed() {
        let engine = ConsensusEngine::new(0.6);
        let votes = vec![
            make_vote("a1", Vote::Approve, 1.0),
            make_vote("a2", Vote::Reject, 1.0),
        ];
        let score = engine.calculate_weighted_score(&votes);
        assert!((score - 0.0).abs() < 0.01);
    }
}