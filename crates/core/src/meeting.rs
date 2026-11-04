//! Transkrypcja spotkania (VA-CONF-4): ścieżka mikrofonu i ścieżka dźwięku systemowego idą
//! przez model osobno, a wynik jest scalany po czasie z etykietą przy każdej zmianie mówcy.

use va_audio::{SilenceParams, TARGET_SAMPLE_RATE, compress_pauses_mapped, speech_bounds};
use va_stt::{Segment, SpeechToText};

/// Etykieta wypowiedzi użytkownika (ścieżka mikrofonu).
pub const ME_LABEL: &str = "Ja:";
/// Etykieta wypowiedzi rozmówców (ścieżka dźwięku systemowego).
pub const OTHERS_LABEL: &str = "Rozmówcy:";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Speaker {
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

/// Jedna ścieżka: przycięcie ciszy na brzegach, skrócenie pauz (VA-REC-5), transkrypcja,
/// a znaczniki segmentów wracają na oś czasu całej ścieżki. Sama cisza nie trafia do modelu.
pub fn transcribe_track(
    stt: &mut dyn SpeechToText,
    samples: &[f32],
    silence: &SilenceParams,
) -> va_stt::Result<Vec<Segment>> {
    let Some(bounds) = speech_bounds(samples, TARGET_SAMPLE_RATE, silence) else {
        return Ok(Vec::new());
    };
    let compressed = compress_pauses_mapped(&samples[bounds.clone()], TARGET_SAMPLE_RATE, silence);
    let map = compressed.map.shifted(bounds.start);
    let transcript = stt.transcribe(&compressed.samples)?;
    Ok(transcript
        .segments
        .into_iter()
        .map(|segment| Segment {
            start_ms: map.original_ms(segment.start_ms),
            end_ms: map.original_ms(segment.end_ms),
            text: segment.text,
        })
        .collect())
}

/// Scala segmenty obu ścieżek w kolejności czasu. Każda zmiana mówcy zaczyna nową linię;
/// z `labels` linia zaczyna się od „Ja:” albo „Rozmówcy:”. Segmenty bez tekstu są pomijane.
pub fn merge_tracks(mic: &[Segment], system: &[Segment], labels: bool) -> String {
    let mut timeline: Vec<(Speaker, &Segment)> = mic
        .iter()
        .map(|segment| (Speaker::Me, segment))
        .chain(system.iter().map(|segment| (Speaker::Others, segment)))
        .filter(|(_, segment)| !segment.text.trim().is_empty())
        .collect();
    timeline.sort_by_key(|(speaker, segment)| (segment.start_ms, *speaker == Speaker::Others));

    let mut turns: Vec<(Speaker, Vec<&str>)> = Vec::new();
    for (speaker, segment) in timeline {
        match turns.last_mut() {
            Some((current, words)) if *current == speaker => words.push(segment.text.trim()),
            _ => turns.push((speaker, vec![segment.text.trim()])),
        }
    }
    turns
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

    fn segment(start_ms: u64, end_ms: u64, text: &str) -> Segment {
        Segment {
            start_ms,
            end_ms,
            text: text.into(),
        }
    }

    #[test]
    // specky: crit 01M6JR8S2RJ391FHM6BF9C8PS1
    fn earlier_speaker_comes_first_and_order_follows_time_offsets() {
        let mic = [segment(0, 3_000, " Idę na spacer do parku.")];
        let system = [segment(10_000, 13_000, " I am going to the park.")];

        let mic_first = merge_tracks(&mic, &system, true);
        let mic_later = [segment(10_000, 13_000, " Idę na spacer do parku.")];
        let system_earlier = [segment(0, 3_000, " I am going to the park.")];
        let system_first = merge_tracks(&mic_later, &system_earlier, true);

        assert_eq!(
            mic_first,
            "Ja: Idę na spacer do parku.\nRozmówcy: I am going to the park."
        );
        assert_eq!(
            system_first,
            "Rozmówcy: I am going to the park.\nJa: Idę na spacer do parku."
        );
    }

    #[test]
    // specky: crit 01M6JR8S2RRK4A1VQD6ECZY8E9
    fn label_appears_only_when_the_speaker_changes() {
        let mic = [
            segment(0, 1_000, " Dzień dobry."),
            segment(1_000, 2_000, " Zaczynamy?"),
            segment(9_000, 10_000, " Dziękuję."),
        ];
        let system = [
            segment(3_000, 4_000, " Tak."),
            segment(5_000, 6_000, " Proszę bardzo."),
        ];

        let merged = merge_tracks(&mic, &system, true);

        assert_eq!(
            merged,
            "Ja: Dzień dobry. Zaczynamy?\nRozmówcy: Tak. Proszę bardzo.\nJa: Dziękuję."
        );
    }

    #[test]
    // specky: crit 01M6JR8S2RHN883WPQB8YQBSPK
    fn silent_track_gives_no_label() {
        let mic = [segment(0, 2_000, " Notatka tylko ode mnie.")];

        let merged = merge_tracks(&mic, &[], true);
        let with_blank_segment = merge_tracks(&mic, &[segment(500, 900, "  ")], true);

        assert_eq!(merged, "Ja: Notatka tylko ode mnie.");
        assert_eq!(with_blank_segment, merged);
        assert_eq!(merge_tracks(&[], &[], true), "");
    }

    #[test]
    // specky: crit 01M6JR8S2RF2G1JMEW0CM3FJTY
    fn without_labels_text_keeps_time_order() {
        let mic = [segment(4_000, 5_000, " Druga.")];
        let system = [segment(0, 1_000, " Pierwsza.")];

        assert_eq!(merge_tracks(&mic, &system, false), "Pierwsza.\nDruga.");
    }

    #[test]
    fn simultaneous_start_puts_user_first() {
        let mic = [segment(1_000, 2_000, " Ja też.")];
        let system = [segment(1_000, 2_000, " Hej.")];

        assert_eq!(
            merge_tracks(&mic, &system, true),
            "Ja: Ja też.\nRozmówcy: Hej."
        );
    }

    #[test]
    // specky: crit 01M6JR8S2RWX49RC3BZ7YSK3HW
    fn track_segments_are_placed_on_the_original_time_axis() {
        let silence = SilenceParams {
            threshold_rms: 0.01,
            padding_ms: 0,
        };
        let mut samples = vec![0.0; 16_000 * 3];
        samples.extend((0..16_000).map(|index| if index % 2 == 0 { 0.3 } else { -0.3 }));
        let mut stt = ScriptedStt::answering([Ok(" mowa".into())]);

        let segments = transcribe_track(&mut stt, &samples, &silence).unwrap();

        assert_eq!(segments, vec![segment(3_000, 4_000, " mowa")]);
    }

    #[test]
    fn silent_track_is_not_sent_to_the_model() {
        let silence = SilenceParams {
            threshold_rms: 0.01,
            padding_ms: 0,
        };
        let mut stt = ScriptedStt::answering([]);
        let observer = stt.clone();

        let segments = transcribe_track(&mut stt, &vec![0.0; 16_000], &silence).unwrap();

        assert!(segments.is_empty());
        assert!(observer.received().is_empty());
    }
}
