//! Session state is independent of terminal widgets and audio processes.
use crate::{
    language_input::{self, Language},
    vocabulary::VocabularyEntry,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Preparing,
    Input,
    Checked,
    Finished,
    Unavailable,
}
#[derive(Debug)]
pub struct Dictation {
    entries: Vec<VocabularyEntry>,
    index: usize,
    pub phase: Phase,
    pub input: String,
    pub checked: usize,
    pub correct: usize,
    pub heard: bool,
    pub operation: u64,
    pub status: String,
    pub accepted: bool,
}
#[derive(Debug, PartialEq, Eq)]
pub struct InputView {
    pub input: String,
    pub index: usize,
    pub total: usize,
    pub checked: usize,
    pub correct: usize,
    pub status: String,
}
impl Dictation {
    pub fn new(mut entries: Vec<VocabularyEntry>, length: usize) -> Result<Self, &'static str> {
        if entries.is_empty() || length == 0 {
            return Err("Vocabulary set is empty");
        }
        entries.truncate(length);
        Ok(Self {
            entries,
            index: 0,
            phase: Phase::Preparing,
            input: String::new(),
            checked: 0,
            correct: 0,
            heard: false,
            operation: 0,
            status: "Checking Spanish audio…".into(),
            accepted: false,
        })
    }
    pub fn current(&self) -> &VocabularyEntry {
        &self.entries[self.index]
    }
    pub fn input_view(&self) -> InputView {
        InputView {
            input: self.input.clone(),
            index: self.index + 1,
            total: self.entries.len(),
            checked: self.checked,
            correct: self.correct,
            status: self.status.clone(),
        }
    }
    pub fn audio_event(&mut self, operation: u64, result: Result<(), String>) -> bool {
        if operation != self.operation {
            return false;
        }
        match (self.phase, result) {
            (Phase::Preparing, Ok(())) => {
                self.phase = Phase::Input;
                true
            }
            (Phase::Preparing, Err(message)) => {
                self.phase = Phase::Unavailable;
                self.status = message;
                false
            }
            (_, Ok(())) => {
                self.heard = true;
                self.status = "Audio ready".into();
                false
            }
            (_, Err(message)) => {
                self.status = message;
                false
            }
        }
    }

    pub fn push(&mut self, ch: char) {
        if self.phase == Phase::Input {
            self.input.push(ch);
        }
    }
    pub fn backspace(&mut self) {
        if self.phase == Phase::Input {
            language_input::pop_grapheme(&mut self.input);
        }
    }
    pub fn submit(&mut self) -> bool {
        if self.phase != Phase::Input || self.input.trim().is_empty() {
            return false;
        }
        if !self.heard {
            self.status = "Wait for successful audio; F2: repeat".into();
            return false;
        }
        self.accepted =
            language_input::matches(self.input.trim(), &self.current().es, Language::Spanish);
        self.checked += 1;
        self.correct += usize::from(self.accepted);
        self.phase = Phase::Checked;
        true
    }
    pub fn advance(&mut self) -> bool {
        if self.phase != Phase::Checked {
            return false;
        }
        if self.index + 1 == self.entries.len() {
            self.phase = Phase::Finished;
            return false;
        }
        self.index += 1;
        self.input.clear();
        self.heard = false;
        self.phase = Phase::Input;
        true
    }
    pub fn errors(&self) -> usize {
        self.checked - self.correct
    }
    pub fn percent(&self) -> usize {
        if self.checked == 0 {
            0
        } else {
            (100.0 * self.correct as f64 / self.checked as f64).round() as usize
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn entry(es: &str) -> VocabularyEntry {
        VocabularyEntry {
            id: es.into(),
            ru: "перевод".into(),
            es: es.into(),
        }
    }
    #[test]
    fn hidden_view_does_not_receive_target() {
        let mut a = Dictation::new(vec![entry("año")], 20).unwrap();
        let mut b = Dictation::new(vec![entry("casa")], 20).unwrap();
        for s in [&mut a, &mut b] {
            s.phase = Phase::Input;
            s.push('a');
            s.push('n');
        }
        assert_eq!(a.input_view(), b.input_view());
    }
    #[test]
    fn single_evaluation_and_audio_gate() {
        let mut s = Dictation::new(vec![entry("año"), entry("sí"), entry("casa")], 20).unwrap();
        s.phase = Phase::Input;
        s.input = "ano".into();
        assert!(!s.submit());
        s.heard = true;
        assert!(s.submit());
        assert!(!s.submit());
        assert!(s.advance());
        s.heard = true;
        s.input = "wrong".into();
        assert!(s.submit());
        assert!(s.advance());
        s.heard = true;
        s.input = "casx".into();
        s.backspace();
        s.push('a');
        assert!(s.submit());
        assert!(!s.advance());
        assert_eq!(
            (s.checked, s.correct, s.errors(), s.percent()),
            (3, 2, 1, 67)
        );
    }
    #[test]
    fn obsolete_audio_cannot_unlock_current_word_and_retry_preserves_success() {
        let mut session = Dictation::new(vec![entry("sí")], 20).unwrap();
        session.phase = Phase::Input;
        session.operation = 9;
        session.input = "si".into();
        session.audio_event(8, Ok(()));
        assert!(!session.heard);
        assert!(!session.submit());
        session.audio_event(9, Err("Audio failed".into()));
        assert_eq!(session.input, "si");
        assert_eq!(session.errors(), 0);
        session.operation = 10;
        session.audio_event(9, Ok(()));
        assert!(!session.heard);
        session.audio_event(10, Ok(()));
        assert!(session.heard);
        session.operation = 11;
        session.audio_event(11, Err("Audio failed".into()));
        assert!(session.heard);
        assert!(session.submit());
        assert!(session.accepted);
    }
    #[test]
    fn whitespace_and_empty() {
        assert!(Dictation::new(vec![], 20).is_err());
        let mut s = Dictation::new(vec![entry("sí")], 20).unwrap();
        s.phase = Phase::Input;
        s.heard = true;
        s.input = "  ".into();
        assert!(!s.submit());
        s.input = " si ".into();
        assert!(s.submit());
        assert!(s.accepted);
    }
}
