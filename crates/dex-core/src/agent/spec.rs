//! Agent project specification — the Q&A answers that drive scaffolding.

use serde::{Deserialize, Serialize};

/// Collected answers from the `dex agent new` Q&A flow.
///
/// These answers drive both the deterministic scaffold (file generation)
/// and the generative phase (Claude API call to flesh out agent logic).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAnswers {
    /// Agent name (e.g., "table-anomaly-monitor").
    pub name: String,
    /// One-sentence description of what the agent does.
    pub description: String,
    /// What triggers the agent.
    pub trigger: AgentTrigger,
    /// What success looks like.
    pub success_criteria: String,
    /// What the agent needs to read.
    pub reads: String,
    /// What the agent writes or changes.
    pub writes: String,
    /// Whether it hands off to a human or another agent.
    pub handoff: bool,
    /// Whether it acts autonomously or confirms before acting.
    pub autonomous: bool,
    /// Example input and correct behavior.
    pub example_input: String,
    /// Expected output for the example input.
    pub example_output: String,
    /// What a bad or dangerous output looks like.
    pub bad_output: String,
    /// Deployment target.
    pub deploy_target: AgentDeployTarget,
}

/// What triggers the agent to run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentTrigger {
    /// Triggered by a user request (chat, API call).
    UserRequest,
    /// Triggered on a schedule (cron job).
    Schedule,
    /// Triggered by an event (webhook, table update).
    Event,
    /// Triggered by an upstream system.
    UpstreamSystem,
}

/// How the agent is deployed on Databricks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentDeployTarget {
    /// Databricks Job (scheduled or triggered).
    Job,
    /// Model Serving Endpoint (real-time).
    ServingEndpoint,
    /// Interactive (local development, notebook).
    Interactive,
}
