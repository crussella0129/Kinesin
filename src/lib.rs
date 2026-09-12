//! Kinesin's owned execution boundaries.

pub mod auth;
pub mod cli;
pub mod config;
pub mod core;
pub mod dispatch;
pub mod ingress;
pub mod mcp;
pub mod model;
pub mod onboarding;
pub mod operator;
pub mod policy;
pub mod private_state;
pub mod process;
pub mod replay;
pub mod runner;
pub mod scheduler;
pub mod service;
pub mod signal;
pub mod storage;
pub mod tools;
pub mod verification;

pub fn product_name() -> &'static str {
    "Kinesin"
}

#[cfg(test)]
mod tests {
    #[test]
    fn binary_entry_uses_the_library() {
        assert_eq!(super::product_name(), "Kinesin");
    }
}
