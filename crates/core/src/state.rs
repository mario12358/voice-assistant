//! Automat stanów nagrywania: Idle → Recording → Transcribing → Idle, oraz Error → Idle.
//!
//! Polecenia Start/Stop są takie same dla skrótu klawiszowego i kliknięcia ikony —
//! automat nie wie, skąd przyszły. Wejście niepasujące do stanu jest ignorowane (z logiem),
//! dzięki czemu drugi Start w trakcie nagrania nie tworzy drugiego nagrania, a Stop bez
//! nagrania nic nie robi.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Idle,
    Recording,
    Transcribing,
    Error,
}

/// Wejście automatu: polecenie użytkownika albo wynik pracy kontrolera.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Start,
    Stop,
    TranscriptionFinished,
    Failed,
    ErrorAcknowledged,
}

/// Co kontroler ma zrobić po przejściu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    StartRecording,
    StopRecordingAndTranscribe,
    ReportError,
    None,
}

/// Wynik obsługi wejścia; `ignored` = wejście nie pasowało do stanu i nic się nie zmieniło.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transition {
    pub from: State,
    pub to: State,
    pub action: Action,
    pub ignored: bool,
}

#[derive(Debug)]
pub struct StateMachine {
    state: State,
}

impl Default for StateMachine {
    fn default() -> Self {
        Self { state: State::Idle }
    }
}

impl StateMachine {
    pub fn state(&self) -> State {
        self.state
    }

    pub fn handle(&mut self, input: Input) -> Transition {
        let from = self.state;
        let Some((to, action)) = next(from, input) else {
            tracing::info!(?input, state = ?from, "polecenie zignorowane w tym stanie");
            return Transition {
                from,
                to: from,
                action: Action::None,
                ignored: true,
            };
        };
        self.state = to;
        tracing::debug!(?input, ?from, ?to, "zmiana stanu");
        Transition {
            from,
            to,
            action,
            ignored: false,
        }
    }
}

fn next(state: State, input: Input) -> Option<(State, Action)> {
    use {Action as A, Input as I, State as S};
    match (state, input) {
        (S::Idle, I::Start) => Some((S::Recording, A::StartRecording)),
        (S::Recording, I::Stop) => Some((S::Transcribing, A::StopRecordingAndTranscribe)),
        (S::Transcribing, I::TranscriptionFinished) => Some((S::Idle, A::None)),
        (S::Recording | S::Transcribing, I::Failed) => Some((S::Error, A::ReportError)),
        (S::Error, I::ErrorAcknowledged) => Some((S::Idle, A::None)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_STATES: [State; 4] = [
        State::Idle,
        State::Recording,
        State::Transcribing,
        State::Error,
    ];

    fn machine_in(state: State) -> StateMachine {
        StateMachine { state }
    }

    #[test]
    fn transition_table() {
        use {Action as A, Input as I, State as S};
        let accepted = [
            (S::Idle, I::Start, S::Recording, A::StartRecording),
            (
                S::Recording,
                I::Stop,
                S::Transcribing,
                A::StopRecordingAndTranscribe,
            ),
            (S::Transcribing, I::TranscriptionFinished, S::Idle, A::None),
            (S::Recording, I::Failed, S::Error, A::ReportError),
            (S::Transcribing, I::Failed, S::Error, A::ReportError),
            (S::Error, I::ErrorAcknowledged, S::Idle, A::None),
        ];
        for (from, input, to, action) in accepted {
            let transition = machine_in(from).handle(input);

            assert_eq!(
                (transition.to, transition.action, transition.ignored),
                (to, action, false),
                "{from:?} + {input:?}"
            );
        }
    }

    #[test]
    fn every_other_combination_is_ignored_without_side_effects() {
        let inputs = [
            Input::Start,
            Input::Stop,
            Input::TranscriptionFinished,
            Input::Failed,
            Input::ErrorAcknowledged,
        ];
        for state in ALL_STATES {
            for input in inputs {
                if next(state, input).is_some() {
                    continue;
                }
                let mut machine = machine_in(state);

                let transition = machine.handle(input);

                assert!(transition.ignored, "{state:?} + {input:?}");
                assert_eq!(transition.action, Action::None);
                assert_eq!(machine.state(), state);
            }
        }
    }

    #[test]
    // specky: crit 01M4EKHCREMA5QQMEWG7S8YJ98
    fn second_start_while_recording_does_not_start_second_recording() {
        let mut machine = StateMachine::default();
        machine.handle(Input::Start);

        let transition = machine.handle(Input::Start);

        assert!(transition.ignored);
        assert_eq!(transition.action, Action::None);
        assert_eq!(machine.state(), State::Recording);
    }

    #[test]
    // specky: crit 01M4EKHCW5YNRZQXRNVTE4Z3DN
    fn stop_without_recording_does_nothing() {
        let mut machine = StateMachine::default();

        let transition = machine.handle(Input::Stop);

        assert!(transition.ignored);
        assert_eq!(transition.action, Action::None);
        assert_eq!(machine.state(), State::Idle);
    }

    #[test]
    fn start_during_transcription_is_ignored() {
        let mut machine = machine_in(State::Transcribing);

        assert!(machine.handle(Input::Start).ignored);
        assert_eq!(machine.state(), State::Transcribing);
    }

    #[test]
    fn full_cycle_returns_to_idle_and_allows_next_recording() {
        let mut machine = StateMachine::default();

        machine.handle(Input::Start);
        machine.handle(Input::Stop);
        machine.handle(Input::TranscriptionFinished);
        let next_recording = machine.handle(Input::Start);

        assert_eq!(next_recording.action, Action::StartRecording);
        assert_eq!(machine.state(), State::Recording);
    }

    #[test]
    fn failure_goes_through_error_back_to_idle() {
        let mut machine = machine_in(State::Transcribing);

        machine.handle(Input::Failed);
        assert_eq!(machine.state(), State::Error);
        machine.handle(Input::ErrorAcknowledged);

        assert_eq!(machine.state(), State::Idle);
    }
}
