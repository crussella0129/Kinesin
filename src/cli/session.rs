//! Ephemeral recent-turn memory. Durable runs retain their own frozen inputs.

use crate::session::{MAX_SESSION_CONTEXT_BYTES, MAX_SESSION_TURNS, SessionContext, SessionTurn};

const ENTRY_BYTES: usize = 2_048;
const CLIPPED: &str = "\n[truncated for session memory]\n";

pub(super) struct Memory {
    context: SessionContext,
    byte_limit: usize,
    omitted: usize,
    clipped: usize,
}

impl Memory {
    pub(super) fn new(context_size: u32, output_tokens: u32) -> Self {
        // Reserve a modest byte window for history on small local models. This
        // is deliberately a memory policy, not a claim to count model tokens.
        let byte_limit =
            (context_size.saturating_sub(output_tokens) as usize).min(MAX_SESSION_CONTEXT_BYTES);
        Self {
            context: SessionContext { turns: Vec::new() },
            byte_limit,
            omitted: 0,
            clipped: 0,
        }
    }

    pub(super) fn context(&self) -> Option<SessionContext> {
        (!self.context.turns.is_empty()).then(|| self.context.clone())
    }

    pub(super) fn clear(&mut self) {
        self.context.turns.clear();
        self.omitted = 0;
        self.clipped = 0;
    }

    pub(super) fn status(&self) -> String {
        format!(
            "Session memory: {} recent completed turns, {} / {} bytes.\n{} older turns omitted; {} turns shortened.\nLive context clears on /new or exit; saved runs follow capture and retention settings.\nFile contents are re-read as needed.\n\n",
            self.context.turns.len(),
            self.bytes(),
            self.byte_limit,
            self.omitted,
            self.clipped,
        )
    }

    fn bytes(&self) -> usize {
        if self.context.turns.is_empty() {
            0
        } else {
            serde_json::to_vec(&self.context).map_or(usize::MAX, |bytes| bytes.len())
        }
    }

    pub(super) fn omit_oldest(&mut self) -> bool {
        if self.context.turns.is_empty() {
            return false;
        }
        self.context.turns.remove(0);
        self.omitted = self.omitted.saturating_add(1);
        true
    }

    pub(super) fn retain_recent(&mut self, count: usize) {
        while self.context.turns.len() > count {
            self.omit_oldest();
        }
    }

    pub(super) fn remember(
        &mut self,
        run_id: String,
        prompt: &str,
        answer: &str,
    ) -> Option<String> {
        let omitted_before = self.omitted;
        let mut shortened = prompt.len() > ENTRY_BYTES || answer.len() > ENTRY_BYTES;
        let turn = SessionTurn {
            run_id,
            prompt: clip(prompt, ENTRY_BYTES),
            answer: clip(answer, ENTRY_BYTES),
        };
        self.context.turns.push(turn);
        while self.context.turns.len() > 1
            && (self.context.turns.len() > MAX_SESSION_TURNS || self.bytes() > self.byte_limit)
        {
            self.omit_oldest();
        }
        // JSON escaping can expand an entry. Even one enormous/control-heavy
        // answer must fit without blocking the next request or growing memory.
        while self.bytes() > self.byte_limit && !self.context.turns.is_empty() {
            let last = self.context.turns.last_mut().expect("nonempty memory");
            let text = if last.prompt.len() > last.answer.len() {
                &mut last.prompt
            } else {
                &mut last.answer
            };
            if text.len() <= CLIPPED.len() * 2 {
                self.omit_oldest();
            } else {
                *text = clip(text, text.len() / 2);
                shortened = true;
            }
        }
        if shortened {
            self.clipped = self.clipped.saturating_add(1);
        }
        (shortened || self.omitted > omitted_before).then(|| {
            format!(
                "[Session memory adjusted: {} older turns omitted{}; /context shows the retained size.]\n\n",
                self.omitted - omitted_before,
                if shortened { ", this turn shortened" } else { "" },
            )
        })
    }
}

fn clip(text: &str, max_bytes: usize) -> String {
    if text.len() <= max_bytes {
        return text.to_owned();
    }
    let available = max_bytes.saturating_sub(CLIPPED.len());
    let mut head = available / 2;
    while !text.is_char_boundary(head) {
        head -= 1;
    }
    let mut tail = text.len() - (available - head);
    while !text.is_char_boundary(tail) {
        tail += 1;
    }
    format!("{}{CLIPPED}{}", &text[..head], &text[tail..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_preserves_unicode_ends_when_shortening_a_turn() {
        let mut memory = Memory::new(16_384, 512);
        let prompt = format!("start 🦀{}終 end", "é中🦀".repeat(500));
        let answer = format!("answer 🦀{}終 done", "é中🦀".repeat(500));

        assert!(
            memory
                .remember("unicode-run".into(), &prompt, &answer)
                .is_some()
        );
        let context = memory.context().unwrap();
        let turn = &context.turns[0];
        assert!(turn.prompt.starts_with("start 🦀"));
        assert!(turn.prompt.ends_with("終 end"));
        assert!(turn.answer.starts_with("answer 🦀"));
        assert!(turn.answer.ends_with("終 done"));
        assert!(turn.prompt.contains(CLIPPED));
        assert!(turn.answer.contains(CLIPPED));
        assert!(turn.prompt.len() <= ENTRY_BYTES);
        assert!(turn.answer.len() <= ENTRY_BYTES);
        assert!(context.encode().unwrap().len() <= MAX_SESSION_CONTEXT_BYTES);
        assert_eq!(memory.clipped, 1);
    }

    #[test]
    fn escaped_text_fits_the_encoded_budget_even_for_tiny_windows() {
        let prompt = format!("request{}end", "\u{0001}\"\\🦀".repeat(600));
        let answer = format!("answer{}end", "\u{0002}\"\\中".repeat(600));
        for available in [0, 1, 64, 512, 4_096, 100_000] {
            let mut memory = Memory::new(available + 512, 512);
            memory.remember("escaped-run".into(), &prompt, &answer);
            let limit = (available as usize).min(MAX_SESSION_CONTEXT_BYTES);
            match memory.context() {
                Some(context) => {
                    let encoded = context.encode().unwrap();
                    assert!(encoded.len() <= limit);
                    let decoded: SessionContext = serde_json::from_str(&encoded).unwrap();
                    assert_eq!(decoded, context);
                }
                None => assert!(limit < 512),
            }
        }
    }

    #[test]
    fn memory_keeps_the_newest_turns_and_clear_resets_context_and_counts() {
        let mut memory = Memory::new(16_384, 512);
        for index in 0..MAX_SESSION_TURNS + 4 {
            memory.remember(format!("run-{index}"), "request", "answer");
        }
        let context = memory.context().unwrap();
        assert_eq!(context.turns.len(), MAX_SESSION_TURNS);
        assert_eq!(context.turns[0].run_id, "run-4");
        assert_eq!(context.turns.last().unwrap().run_id, "run-19");
        assert_eq!(memory.omitted, 4);
        memory.retain_recent(2);
        assert_eq!(memory.context().unwrap().turns[0].run_id, "run-18");
        assert_eq!(memory.omitted, 18);

        memory.clear();
        assert!(memory.context().is_none());
        assert_eq!(memory.bytes(), 0);
        assert_eq!(memory.omitted, 0);
        assert_eq!(memory.clipped, 0);
        assert!(
            memory
                .remember("fresh-run".into(), "new request", "new answer")
                .is_none()
        );
        assert_eq!(memory.context().unwrap().turns.len(), 1);
    }
}
