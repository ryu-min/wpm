use super::{Language, language as language_input};

#[derive(Debug)]
pub(crate) struct TypingSession {
    target_text: String,
    input_text: String,
    start_time: Option<std::time::Instant>,
    wpm: f64,
    elapsed: f64,
    time_limit: Option<u64>,
    last_stats_update: Option<std::time::Instant>,
    layout_conversion: bool,
    ignore_case: bool,
    languages: Vec<Language>,
    finished_elapsed: Option<f64>,
    extend_final_grapheme: bool,
}

const STATS_UPDATE_INTERVAL_MS: u64 = 100;

impl TypingSession {
    pub(crate) fn new(target_text: String) -> Self {
        Self {
            target_text,
            input_text: String::new(),
            start_time: None,
            wpm: 0.0,
            elapsed: 0.0,
            time_limit: None,
            last_stats_update: None,
            layout_conversion: false,
            ignore_case: false,
            languages: Vec::new(),
            finished_elapsed: None,
            extend_final_grapheme: false,
        }
    }

    pub(crate) fn with_time_limit(mut self, seconds: u64) -> Self {
        self.time_limit = Some(seconds);
        self
    }

    pub(crate) fn with_layout_conversion(mut self, enabled: bool) -> Self {
        self.layout_conversion = enabled;
        self
    }

    pub(crate) fn with_ignore_case(mut self, enabled: bool) -> Self {
        self.ignore_case = enabled;
        self
    }

    pub(crate) fn accepts(&self, index: usize, input: &str, target: &str) -> bool {
        language_input::accepts(input, target, self.policy(index), self.ignore_case)
    }

    pub(crate) fn with_languages(mut self, languages: Vec<Language>) -> Self {
        self.languages = languages;
        self
    }

    pub(crate) fn policy(&self, index: usize) -> Language {
        self.languages
            .get(index)
            .copied()
            .unwrap_or(if self.layout_conversion {
                Language::Layout
            } else {
                Language::Exact
            })
    }

    fn correct_count(&self) -> usize {
        language_input::units(&self.input_text)
            .iter()
            .zip(language_input::units(&self.target_text))
            .enumerate()
            .filter(|(index, (a, b))| self.accepts(*index, a, b))
            .count()
    }

    pub(crate) fn insert_newline_if_expected(&mut self) {
        if language_input::units(&self.target_text)
            .get(language_input::units(&self.input_text).len())
            .map(String::as_str)
            == Some("\n")
        {
            self.add_char('\n');
        }
    }
    pub(crate) fn target_text(&self) -> &str {
        &self.target_text
    }
    pub(crate) fn displayed_wpm(&self) -> f64 {
        self.wpm
    }
    pub(crate) fn displayed_elapsed(&self) -> f64 {
        self.elapsed
    }

    pub(crate) fn add_char(&mut self, ch: char) {
        self.input_text.push(ch);
        self.start_timer_if_needed();
    }

    pub(crate) fn remove_char(&mut self) {
        language_input::pop_grapheme(&mut self.input_text);
    }

    pub(crate) fn reset(&mut self) {
        self.input_text.clear();
        self.start_time = None;
        self.wpm = 0.0;
        self.elapsed = 0.0;
        self.last_stats_update = None;
        self.finished_elapsed = None;
        self.extend_final_grapheme = false;
    }

    pub(crate) fn update_stats(&mut self) {
        let now = std::time::Instant::now();
        if let Some(last) = self.last_stats_update
            && now.duration_since(last).as_millis() < STATS_UPDATE_INTERVAL_MS as u128
        {
            return;
        }
        self.last_stats_update = Some(now);
        self.elapsed = self.get_elapsed_time();
        self.wpm = self.get_wpm();
    }

    pub(crate) fn get_wpm(&self) -> f64 {
        let elapsed = self.get_elapsed_time();
        if elapsed == 0.0 {
            return 0.0;
        }
        let correct = self.correct_count() as f64;
        if correct == 0.0 {
            return 0.0;
        }
        let words = correct / 5.0;
        (words / elapsed) * 60.0
    }

    pub(crate) fn get_elapsed_time(&self) -> f64 {
        if let Some(elapsed) = self.finished_elapsed {
            return elapsed;
        }
        match self.start_time {
            Some(time) => time.elapsed().as_secs_f64(),
            None => 0.0,
        }
    }

    fn start_timer_if_needed(&mut self) {
        if self.start_time.is_none() && !self.input_text.is_empty() {
            self.start_time = Some(std::time::Instant::now());
        }
    }

    pub(crate) fn get_input(&self) -> &str {
        &self.input_text
    }

    fn timed_out(&self) -> bool {
        self.time_limit
            .zip(self.start_time)
            .is_some_and(|(limit, start)| start.elapsed().as_secs() >= limit)
    }

    pub(crate) fn is_complete(&self) -> bool {
        self.timed_out()
            || language_input::units(&self.input_text).len()
                >= language_input::units(&self.target_text).len()
    }

    pub(crate) fn finalize(&mut self) {
        if self.finished_elapsed.is_none() {
            self.extend_final_grapheme = !self.timed_out() && !self.input_text.is_empty();
            self.finished_elapsed = Some(self.get_elapsed_time());
        }
    }

    /// A combining mark can arrive in another terminal event after auto-completion.
    /// Any other key seals the result and retains the normal Result behavior.
    pub(crate) fn extend_result(&mut self, character: Option<char>) -> bool {
        if self.extend_final_grapheme
            && let Some(ch) = character
            && unicode_normalization::char::is_combining_mark(ch)
        {
            let before = language_input::units(&self.input_text).len();
            let mut candidate = self.input_text.clone();
            candidate.push(ch);
            if language_input::units(&candidate).len() == before {
                self.input_text = candidate;
                return true;
            }
        }
        self.extend_final_grapheme = false;
        false
    }

    pub(crate) fn get_accuracy(&self) -> f64 {
        if self.input_text.is_empty() {
            return 0.0;
        }
        let total = language_input::units(&self.input_text).len();
        if total == 0 {
            return 0.0;
        }
        let correct = self.correct_count();
        correct as f64 / total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_positions_backspace_and_stats_share_graphemes() {
        let mut composed = TypingSession::new("caféx".into())
            .with_layout_conversion(true)
            .with_languages(vec![Language::Spanish; 5]);
        let mut decomposed = TypingSession::new("caféx".into())
            .with_layout_conversion(true)
            .with_languages(vec![Language::Spanish; 5]);
        for ch in "café".chars() {
            composed.add_char(ch);
        }
        for ch in "cafe\u{301}".chars() {
            decomposed.add_char(ch);
        }
        assert_eq!(composed.correct_count(), decomposed.correct_count());
        assert_eq!(decomposed.correct_count(), 4);
        decomposed.remove_char();
        assert_eq!(decomposed.get_input(), "caf");
        decomposed.add_char('e');
        decomposed.add_char('\u{301}');
        decomposed.add_char('x');
        assert!(decomposed.is_complete());
        assert_eq!(decomposed.get_accuracy(), 1.0);
    }
}

#[cfg(test)]
mod final_grapheme_tests {
    use super::*;
    #[test]
    fn late_marks_recompute_numerator_with_frozen_time_and_other_keys_seal() {
        let mut typing = TypingSession::new("café".into());
        for ch in "cafe".chars() {
            typing.add_char(ch);
        }
        typing.start_time = Some(std::time::Instant::now() - std::time::Duration::from_secs(1));
        typing.finalize();
        let elapsed = typing.get_elapsed_time();
        let wpm = typing.get_wpm();
        assert!(typing.extend_result(Some('\u{301}')));
        assert_eq!(typing.get_accuracy(), 1.0);
        assert_eq!(typing.get_elapsed_time(), elapsed);
        assert!(typing.get_wpm() > wpm);
        assert!(!typing.extend_result(None));
        let input = typing.get_input().to_string();
        assert!(!typing.extend_result(Some('\u{308}')));
        assert_eq!(typing.get_input(), input);
        typing.reset();
        assert_eq!(typing.finished_elapsed, None);
    }
    #[test]
    fn timed_result_and_arrow_prevent_late_marks() {
        for timed in [false, true] {
            let mut typing = TypingSession::new("café".into()).with_time_limit(1);
            for ch in "cafe".chars() {
                typing.add_char(ch);
            }
            if timed {
                typing.start_time =
                    Some(std::time::Instant::now() - std::time::Duration::from_secs(2));
            }
            typing.finalize();
            let before = (
                typing.get_input().to_string(),
                typing.get_accuracy(),
                typing.get_wpm(),
                typing.get_elapsed_time(),
            );
            if !timed {
                assert!(!typing.extend_result(None));
            }
            assert!(!typing.extend_result(Some('\u{301}')));
            assert_eq!(
                (
                    typing.get_input().to_string(),
                    typing.get_accuracy(),
                    typing.get_wpm(),
                    typing.get_elapsed_time()
                ),
                before
            );
        }
    }
}

#[cfg(test)]
mod case_tests {
    use super::*;

    #[test]
    fn case_policy_preserves_language_boundaries_and_unicode() {
        for (policy, input, target, accepted) in [
            (Language::Exact, "a", "A", true),
            (Language::Exact, "б", "Б", true),
            (Language::Exact, "a\u{301}", "Á", true),
            (Language::Exact, "ф", "A", false),
            (Language::Layout, "ф", "A", true),
            (Language::Layout, ";", "Ж", true),
            (Language::Layout, ":", "ж", true),
            (Language::Layout, "Ж", ";", true),
            (Language::Layout, ":", "Ж", true),
            (Language::Layout, ":", ";", false),
            (Language::Layout, "ñ", "N", false),
            (Language::Spanish, "a", "Á", true),
            (Language::Spanish, "т", "Ñ", true),
            (Language::Spanish, "ü", "Ü", true),
            (Language::Spanish, "e", "Á", false),
            (Language::Spanish, " ", "Á", false),
        ] {
            let session = TypingSession::new(target.into())
                .with_languages(vec![policy])
                .with_ignore_case(true);
            assert_eq!(
                session.accepts(0, input, target),
                accepted,
                "{input} -> {target}"
            );
            let strict = TypingSession::new(target.into()).with_languages(vec![policy]);
            assert_eq!(
                strict.accepts(0, input, target),
                language_input::equivalent(input, target, policy)
            );
        }
        assert!(!language_input::equivalent("a", "A", Language::Spanish));
        assert!(!language_input::matches(
            "ANO",
            "año",
            Language::Spanish,
            false
        ));
    }

    #[test]
    fn accepted_case_and_exact_input_have_equal_stats_at_fixed_time() {
        for input in ["CAFÉ", "cafe\u{301}"] {
            let mut session = TypingSession::new("CAFÉ".into()).with_ignore_case(true);
            for ch in input.chars() {
                session.add_char(ch);
            }
            session.finished_elapsed = Some(2.0);
            assert_eq!(session.correct_count(), 4);
            assert_eq!(session.get_accuracy(), 1.0);
            assert_eq!(session.get_wpm(), 24.0);
        }
        let mut session = TypingSession::new("CAFÉ".into()).with_ignore_case(true);
        for ch in "cafe".chars() {
            session.add_char(ch);
        }
        session.finalize();
        let elapsed = session.get_elapsed_time();
        assert!(session.extend_result(Some('\u{301}')));
        assert_eq!(session.get_accuracy(), 1.0);
        assert_eq!(session.get_elapsed_time(), elapsed);
        session.reset();
        for ch in "cafe\u{301}".chars() {
            session.add_char(ch);
        }
        session.remove_char();
        assert_eq!(session.get_input(), "caf");
    }
}
