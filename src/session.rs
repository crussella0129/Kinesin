//! Bounded reference data carried between runs of one live local session.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

pub const MAX_SESSION_CONTEXT_BYTES: usize = 8_192;
pub const MAX_SESSION_TURNS: usize = 16;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SessionTurn {
    pub run_id: String,
    pub prompt: String,
    pub answer: String,
}

/// The caller retains successful turns in memory. This is reference data, never
/// inherited authority, tool observations, or independently checked evidence.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SessionContext {
    pub turns: Vec<SessionTurn>,
}

impl SessionContext {
    pub fn validate(&self) -> Result<(), String> {
        self.encode().map(|_| ())
    }

    /// A single encoding is used for byte admission, provenance and the model's
    /// reference message. Bound individual fields before allocating the JSON.
    pub fn encode(&self) -> Result<String, String> {
        if self.turns.is_empty() || self.turns.len() > MAX_SESSION_TURNS {
            return Err("session context must contain from 1 to 16 turns".into());
        }
        let mut origins = BTreeSet::new();
        for turn in &self.turns {
            crate::config::validate_id(&turn.run_id)?;
            if !origins.insert(&turn.run_id) {
                return Err("session context contains a duplicate run origin".into());
            }
            for text in [&turn.prompt, &turn.answer] {
                if text.trim().is_empty() || text.len() > MAX_SESSION_CONTEXT_BYTES {
                    return Err("session text must be nonempty and within its byte limit".into());
                }
            }
        }
        let encoded = serde_json::to_string(self).map_err(|_| "cannot encode session context")?;
        if encoded.len() > MAX_SESSION_CONTEXT_BYTES {
            return Err("session context exceeds its 8 KiB encoded byte limit".into());
        }
        Ok(encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn(index: usize) -> SessionTurn {
        SessionTurn {
            run_id: format!("run-{index}"),
            prompt: "Remember the codename: 雪豹.".into(),
            answer: "The codename is 雪豹.".into(),
        }
    }

    #[test]
    fn session_context_bounds_use_encoded_bytes_and_preserve_order() {
        let context = SessionContext {
            turns: (0..MAX_SESSION_TURNS).map(turn).collect(),
        };
        let encoded = context.encode().unwrap();
        assert_eq!(
            serde_json::from_str::<SessionContext>(&encoded).unwrap(),
            context
        );
        let mut too_many = context;
        too_many.turns.push(turn(MAX_SESSION_TURNS));
        assert!(too_many.validate().is_err());

        let mut exact = SessionContext {
            turns: vec![turn(0)],
        };
        let spare = MAX_SESSION_CONTEXT_BYTES - exact.encode().unwrap().len();
        exact.turns[0].answer.push_str(&"x".repeat(spare));
        assert_eq!(exact.encode().unwrap().len(), MAX_SESSION_CONTEXT_BYTES);
        exact.turns[0].answer.push('x');
        assert!(exact.validate().is_err());

        // Each decoded field fits; escaping still overflows the encoded cap.
        let escaped = SessionContext {
            turns: vec![SessionTurn {
                answer: "\"".repeat(MAX_SESSION_CONTEXT_BYTES / 2),
                ..turn(0)
            }],
        };
        assert!(escaped.validate().is_err());
    }

    #[test]
    fn session_context_rejects_empty_text_and_ambiguous_origins() {
        for context in [
            SessionContext { turns: vec![] },
            SessionContext {
                turns: vec![turn(0), turn(0)],
            },
            SessionContext {
                turns: vec![SessionTurn {
                    run_id: "../other-owner".into(),
                    ..turn(0)
                }],
            },
            SessionContext {
                turns: vec![SessionTurn {
                    prompt: " \n\t".into(),
                    ..turn(0)
                }],
            },
            SessionContext {
                turns: vec![SessionTurn {
                    answer: String::new(),
                    ..turn(0)
                }],
            },
            SessionContext {
                turns: vec![SessionTurn {
                    answer: "x".repeat(MAX_SESSION_CONTEXT_BYTES + 1),
                    ..turn(0)
                }],
            },
        ] {
            assert!(context.validate().is_err());
        }
    }
}
