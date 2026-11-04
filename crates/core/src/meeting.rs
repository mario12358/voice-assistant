//! Transkrypcja spotkania (VA-CONF-4): tury mówców wyznaczane z sygnału obu ścieżek (mikrofon,
//! dźwięk systemowy), każda tura transkrybowana osobno, wynik w kolejności tur z etykietą przy
//! każdej zmianie mówcy.
//!
//! Kolejność bierze się z czasu w sygnale, nie ze znaczników modelu: Whisper skleja kilka
//! wypowiedzi jednej osoby rozdzielonych ciszą w jeden segment i podaje nieprecyzyjne początki,
//! więc scalanie po segmentach przestawiało wypowiedzi (test ręczny 2026-11-04).

use std::ops::Range;

use va_audio::{SilenceParams, TARGET_SAMPLE_RATE, compress_pauses, speech_regions};
use va_stt::SpeechToText;

/// Etykieta wypowiedzi użytkownika (ścieżka mikrofonu).
pub const ME_LABEL: &str = "Ja:";
/// Etykieta wypowiedzi rozmówców (ścieżka dźwięku systemowego).
pub const OTHERS_LABEL: &str = "Rozmówcy:";
/// Krótsza przerwa w mowie nie dzieli odcinka (oddech, pauza w zdaniu).
const MIN_GAP_MS: u32 = 400;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speaker {
    Me,
    Others,
}

impl Speaker {
    fn label(self) -> &'static str {
        match self {
            Self::Me => ME_LABEL,
            Self::Others => OTHERS_LABEL,
        }
    }
}

/// Tura: ciągły fragment ścieżki jednego mówcy, bez wypowiedzi drugiego w środku.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Turn {
    pub speaker: Speaker,
    /// Zakres próbek (16 kHz) na osi nagrania tej ścieżki.
    pub samples: Range<usize>,
}

/// Tury w kolejności czasu: odcinki mowy obu ścieżek posortowane po początku, a kolejne
/// odcinki tego samego mówcy (bez drugiego pomiędzy) łączone w jedną turę.
pub fn plan_turns(microphone: &[f32], system: &[f32], silence: &SilenceParams) -> Vec<Turn> {
    let regions = |samples, speaker| {
        speech_regions(samples, TARGET_SAMPLE_RATE, silence, MIN_GAP_MS)
            .into_iter()
            .map(move |range| Turn {
                speaker,
                samples: range,
            })
    };
    let mut timeline: Vec<Turn> = regions(microphone, Speaker::Me)
        .chain(regions(system, Speaker::Others))
        .collect();
    timeline.sort_by_key(|turn| (turn.samples.start, turn.speaker == Speaker::Others));

    let mut turns: Vec<Turn> = Vec::new();
    for region in timeline {
        match turns.last_mut() {
            Some(turn) if turn.speaker == region.speaker => {
                turn.samples.end = turn.samples.end.max(region.samples.end);
            }
            _ => turns.push(region),
        }
    }
    turns
}

/// Każda tura osobno przez model (z krótszymi pauzami, VA-REC-5, i filtrami VA-STT-3);
/// tury bez tekstu odpadają. Zwraca tekst w kolejności tur.
pub fn transcribe_turns(
    stt: &mut dyn SpeechToText,
    microphone: &[f32],
    system: &[f32],
    silence: &SilenceParams,
) -> va_stt::Result<Vec<(Speaker, String)>> {
    let mut spoken = Vec::new();
    for turn in plan_turns(microphone, system, silence) {
        let track = match turn.speaker {
            Speaker::Me => microphone,
            Speaker::Others => system,
        };
        let audio = compress_pauses(&track[turn.samples], TARGET_SAMPLE_RATE, silence);
        let text = stt.transcribe(&audio)?.text;
        if !text.trim().is_empty() {
            spoken.push((turn.speaker, text.trim().to_owned()));
        }
    }
    Ok(spoken)
}

/// Zapis rozmowy: każda zmiana mówcy w nowej linii, z `labels` poprzedzona „Ja:” albo
/// „Rozmówcy:”; kolejne teksty tego samego mówcy w jednej linii.
pub fn format_transcript(spoken: &[(Speaker, String)], labels: bool) -> String {
    let mut lines: Vec<(Speaker, Vec<&str>)> = Vec::new();
    for (speaker, text) in spoken {
        match lines.last_mut() {
            Some((current, parts)) if current == speaker => parts.push(text),
            _ => lines.push((*speaker, vec![text])),
        }
    }
    lines
        .iter()
        .map(|(speaker, parts)| {
            let text = parts.join(" ");
            if labels {
                format!("{} {text}", speaker.label())
            } else {
                text
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use va_stt::testing::ScriptedStt;

    use super::*;

    const SILENCE: SilenceParams = SilenceParams {
        threshold_rms: 0.01,
        padding_ms: 0,
    };
    const RATE: usize = 16_000;

    /// Ścieżka długości `total_s` z mową (sygnał powyżej progu) w podanych przedziałach sekund.
    fn track(total_s: f32, speech: &[(f32, f32)]) -> Vec<f32> {
        let mut samples = vec![0.0; (total_s * RATE as f32) as usize];
        for (from, to) in speech {
            let range = (*from * RATE as f32) as usize..(*to * RATE as f32) as usize;
            for (index, sample) in samples[range].iter_mut().enumerate() {
                *sample = if index % 2 == 0 { 0.3 } else { -0.3 };
            }
        }
        samples
    }

    fn spoken(items: &[(Speaker, &str)]) -> Vec<(Speaker, String)> {
        items
            .iter()
            .map(|(speaker, text)| (*speaker, (*text).to_owned()))
            .collect()
    }

    #[test]
    // specky: crit 01M6JR8S2RWX49RC3BZ7YSK3HW
    fn alternating_speech_gives_turns_in_signal_order() {
        let microphone = track(16.0, &[(0.0, 2.0), (5.0, 7.0), (10.0, 12.0)]);
        let system = track(16.0, &[(3.0, 4.5), (8.0, 9.5), (13.0, 15.0)]);

        let turns = plan_turns(&microphone, &system, &SILENCE);

        let speakers: Vec<Speaker> = turns.iter().map(|turn| turn.speaker).collect();
        assert_eq!(
            speakers,
            [
                Speaker::Me,
                Speaker::Others,
                Speaker::Me,
                Speaker::Others,
                Speaker::Me,
                Speaker::Others
            ]
        );
        assert_eq!(turns[2].samples, 5 * RATE..7 * RATE);
    }

    #[test]
    fn consecutive_regions_of_one_speaker_form_one_turn() {
        let microphone = track(10.0, &[(0.0, 2.0), (4.0, 6.0)]);
        let system = track(10.0, &[(7.0, 9.0)]);

        let turns = plan_turns(&microphone, &system, &SILENCE);

        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].samples, 0..6 * RATE);
        assert_eq!(turns[1].speaker, Speaker::Others);
    }

    #[test]
    // specky: crit 01M6JR8S2RJ391FHM6BF9C8PS1
    fn each_turn_is_transcribed_separately_in_time_order() {
        let microphone = track(8.0, &[(0.0, 1.0), (4.0, 5.0)]);
        let system = track(8.0, &[(2.0, 3.0)]);
        let mut stt = ScriptedStt::answering([
            Ok(" Pierwsza.".into()),
            Ok(" Druga.".into()),
            Ok(" Trzecia.".into()),
        ]);
        let observer = stt.clone();

        let result = transcribe_turns(&mut stt, &microphone, &system, &SILENCE).unwrap();

        assert_eq!(
            result,
            spoken(&[
                (Speaker::Me, "Pierwsza."),
                (Speaker::Others, "Druga."),
                (Speaker::Me, "Trzecia.")
            ])
        );
        let lengths: Vec<usize> = observer.received().iter().map(Vec::len).collect();
        assert_eq!(lengths, [RATE, RATE, RATE]);
    }

    #[test]
    // specky: crit 01M6JR8S2RHN883WPQB8YQBSPK
    fn silent_track_gives_no_label_and_is_not_sent_to_the_model() {
        let microphone = track(4.0, &[(0.0, 2.0)]);
        let system = track(4.0, &[]);
        let mut stt = ScriptedStt::answering([Ok(" Notatka tylko ode mnie.".into())]);
        let observer = stt.clone();

        let result = transcribe_turns(&mut stt, &microphone, &system, &SILENCE).unwrap();

        assert_eq!(
            format_transcript(&result, true),
            "Ja: Notatka tylko ode mnie."
        );
        assert_eq!(observer.received().len(), 1);
    }

    #[test]
    // specky: crit 01M6JR8S2RRK4A1VQD6ECZY8E9
    fn label_appears_only_when_the_speaker_changes() {
        let result = spoken(&[
            (Speaker::Me, "Dzień dobry."),
            (Speaker::Me, "Zaczynamy?"),
            (Speaker::Others, "Tak."),
            (Speaker::Me, "Dziękuję."),
        ]);

        assert_eq!(
            format_transcript(&result, true),
            "Ja: Dzień dobry. Zaczynamy?\nRozmówcy: Tak.\nJa: Dziękuję."
        );
    }

    #[test]
    // specky: crit 01M6JR8S2RF2G1JMEW0CM3FJTY
    fn without_labels_text_keeps_turn_order() {
        let result = spoken(&[(Speaker::Others, "Pierwsza."), (Speaker::Me, "Druga.")]);

        assert_eq!(format_transcript(&result, false), "Pierwsza.\nDruga.");
        assert_eq!(format_transcript(&[], true), "");
    }

    #[test]
    fn simultaneous_start_puts_user_first() {
        let microphone = track(3.0, &[(1.0, 2.0)]);
        let system = track(3.0, &[(1.0, 2.0)]);

        let turns = plan_turns(&microphone, &system, &SILENCE);

        assert_eq!(turns[0].speaker, Speaker::Me);
        assert_eq!(turns[1].speaker, Speaker::Others);
    }
}
