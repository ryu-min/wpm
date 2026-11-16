//! Session state is independent of terminal widgets and audio processes.
use super::language as language_input;
use crate::domain::{Language, VocabularyEntry};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase {
    Preparing,
    Input,
    Checked,
    Finished,
    Unavailable,
}
#[derive(Debug)]
pub(crate) struct DictationMaterial {
    pub(crate) entries: Vec<VocabularyEntry>,
    pub(crate) case_insensitive: bool,
}

#[derive(Debug)]
pub(crate) struct Dictation {
    entries: Vec<VocabularyEntry>,
    case_insensitive: bool,
    index: usize,
    phase: Phase,
    input: String,
    checked: usize,
    correct: usize,
    heard: bool,
    operation: u64,
    status: String,
    accepted: bool,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct InputView {
    pub(crate) input: String,
    pub(crate) index: usize,
    pub(crate) total: usize,
    pub(crate) checked: usize,
    pub(crate) correct: usize,
    pub(crate) status: String,
    pub(crate) hint: &'static str,
}
impl Dictation {
    pub(crate) fn phase(&self) -> Phase {
        self.phase
    }
    pub(crate) fn input(&self) -> &str {
        &self.input
    }
    pub(crate) fn checked(&self) -> usize {
        self.checked
    }
    pub(crate) fn correct(&self) -> usize {
        self.correct
    }
    pub(crate) fn status(&self) -> &str {
        &self.status
    }
    pub(crate) fn accepted(&self) -> bool {
        self.accepted
    }
    pub(crate) fn operation(&self) -> u64 {
        self.operation
    }
    pub(crate) fn checking_audio(&mut self, operation: u64) {
        self.operation = operation;
    }
    pub(crate) fn playing_audio(&mut self, operation: u64) {
        self.operation = operation;
        self.status = "Playing…".into();
    }

    pub(crate) fn new(
        mut entries: Vec<VocabularyEntry>,
        length: usize,
    ) -> Result<Self, &'static str> {
        if entries.is_empty() || length == 0 {
            return Err("Vocabulary set is empty");
        }
        entries.truncate(length);
        Ok(Self {
            entries,
            case_insensitive: false,
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
    pub(crate) fn with_case_insensitive(mut self, enabled: bool) -> Self {
        self.case_insensitive = enabled;
        self
    }
    pub(crate) fn current(&self) -> &VocabularyEntry {
        &self.entries[self.index]
    }
    pub(crate) fn input_view(&self) -> InputView {
        InputView {
            input: self.input.clone(),
            index: self.index + 1,
            total: self.entries.len(),
            checked: self.checked,
            correct: self.correct,
            status: self.status.clone(),
            hint: if self.case_insensitive {
                "Type a word or phrase; case and accents are optional"
            } else {
                "Type lowercase; accents are optional"
            },
        }
    }
    pub(crate) fn audio_event(&mut self, operation: u64, result: Result<(), String>) -> bool {
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

    pub(crate) fn push(&mut self, ch: char) {
        if self.phase == Phase::Input {
            self.input.push(ch);
        }
    }
    pub(crate) fn backspace(&mut self) {
        if self.phase == Phase::Input {
            language_input::pop_grapheme(&mut self.input);
        }
    }
    pub(crate) fn submit(&mut self) -> bool {
        if self.phase != Phase::Input || self.input.trim().is_empty() {
            return false;
        }
        if !self.heard {
            self.status = "Wait for successful audio; F2: repeat".into();
            return false;
        }
        self.accepted = if self.case_insensitive {
            language_input::matches(
                &self.input.trim().to_lowercase(),
                &self.current().es.to_lowercase(),
                Language::Spanish,
            )
        } else {
            language_input::matches(self.input.trim(), &self.current().es, Language::Spanish)
        };
        self.checked += 1;
        self.correct += usize::from(self.accepted);
        self.phase = Phase::Checked;
        true
    }
    pub(crate) fn advance(&mut self) -> bool {
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
    pub(crate) fn errors(&self) -> usize {
        self.checked - self.correct
    }
    pub(crate) fn percent(&self) -> usize {
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
