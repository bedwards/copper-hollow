// MIDI file writing via midly.
// Implementation will follow in a future issue.

#[cfg(test)]
mod tests {
    use midly::num::{u15, u28, u4, u7};
    use midly::{Format, Header, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};

    #[test]
    fn midi_write_read_roundtrip() {
        let mut smf = Smf::new(Header::new(
            Format::Parallel,
            Timing::Metrical(u15::new(480)),
        ));

        let track = vec![
            TrackEvent {
                delta: u28::new(0),
                kind: TrackEventKind::Midi {
                    channel: u4::new(0),
                    message: MidiMessage::NoteOn {
                        key: u7::new(60),
                        vel: u7::new(100),
                    },
                },
            },
            TrackEvent {
                delta: u28::new(480),
                kind: TrackEventKind::Midi {
                    channel: u4::new(0),
                    message: MidiMessage::NoteOff {
                        key: u7::new(60),
                        vel: u7::new(0),
                    },
                },
            },
            TrackEvent {
                delta: u28::new(0),
                kind: TrackEventKind::Meta(midly::MetaMessage::EndOfTrack),
            },
        ];
        smf.tracks.push(track);

        let mut buf = Vec::new();
        smf.write_std(&mut buf).unwrap();

        let parsed = Smf::parse(&buf).unwrap();
        assert_eq!(parsed.tracks.len(), 1);
        assert_eq!(parsed.tracks[0].len(), 3);
    }
}
