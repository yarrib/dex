//! `dex agent-env` — declarative agentic dev environments.
//!
//! Reads `dex.agent-env.toml` and generates a devcontainer, an auth bootstrap
//! script, a verify script, and a CI workflow stub, so a project gives coding
//! agents a reproducible, non-interactive environment to build and verify
//! changes in.

pub mod manifest;
pub mod plan;
pub mod render;

pub use manifest::{
    AgentEnvManifest, AuthSpec, BaseSpec, RefreshSpec, ScopeSpec, ServicePrincipalEnvSpec,
    VerifySpec, VerifyStep,
};
pub use plan::{
    AgentEnvFilePlan, AgentEnvPlan, FileState, apply_agent_env_plan, plan_agent_env_init,
};
