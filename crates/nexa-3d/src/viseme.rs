//! Renderer-neutral viseme cue scheduling and envelope sampling.

#[derive(Debug, Clone, PartialEq)]
pub struct VisemeCue {
    pub canonical_name: String,
    pub start_seconds: f32,
    pub duration_seconds: f32,
    pub peak_weight: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisemeWeight {
    pub canonical_name: String,
    pub weight: f32,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct VisemePlayer {
    cues: Vec<VisemeCue>,
}

impl VisemePlayer {
    pub fn schedule(&mut self, cue: VisemeCue) {
        if cue.duration_seconds > 0.0 && cue.peak_weight > 0.0 {
            self.cues.push(VisemeCue {
                start_seconds: cue.start_seconds.max(0.0),
                duration_seconds: cue.duration_seconds,
                peak_weight: cue.peak_weight.clamp(0.0, 1.0),
                canonical_name: cue.canonical_name,
            });
        }
    }

    /// Returns active weights with 20% attack/release ramps. Identical named
    /// cues are combined conservatively so blendshape weights never exceed 1.
    pub fn sample(&self, time_seconds: f32) -> Vec<VisemeWeight> {
        let mut weights = std::collections::BTreeMap::<String, f32>::new();
        for cue in &self.cues {
            let local_time = time_seconds - cue.start_seconds;
            let Some(weight) = envelope(local_time, cue.duration_seconds, cue.peak_weight) else {
                continue;
            };
            let combined = weights.entry(cue.canonical_name.clone()).or_default();
            *combined = (*combined + weight).min(1.0);
        }
        weights
            .into_iter()
            .map(|(canonical_name, weight)| VisemeWeight {
                canonical_name,
                weight,
            })
            .collect()
    }

    pub fn discard_finished(&mut self, time_seconds: f32) {
        self.cues
            .retain(|cue| time_seconds < cue.start_seconds + cue.duration_seconds);
    }
}

fn envelope(local_time: f32, duration: f32, peak: f32) -> Option<f32> {
    if !(0.0..duration).contains(&local_time) {
        return None;
    }
    let ramp = (duration * 0.2).min(duration * 0.5);
    let multiplier = if local_time < ramp {
        local_time / ramp
    } else if local_time > duration - ramp {
        (duration - local_time) / ramp
    } else {
        1.0
    };
    Some(peak * multiplier)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cue(name: &str, start: f32, duration: f32, peak: f32) -> VisemeCue {
        VisemeCue {
            canonical_name: name.to_owned(),
            start_seconds: start,
            duration_seconds: duration,
            peak_weight: peak,
        }
    }

    #[test]
    fn cue_has_a_smooth_attack_hold_and_release() {
        let mut player = VisemePlayer::default();
        player.schedule(cue("A", 1.0, 1.0, 0.8));
        assert!((player.sample(1.1)[0].weight - 0.4).abs() < 0.0001);
        assert_eq!(player.sample(1.5)[0].weight, 0.8);
        assert!(player.sample(1.9)[0].weight < 0.8);
        assert!(player.sample(2.0).is_empty());
    }

    #[test]
    fn matching_cues_are_capped_at_one() {
        let mut player = VisemePlayer::default();
        player.schedule(cue("MBP", 0.0, 1.0, 0.8));
        player.schedule(cue("MBP", 0.0, 1.0, 0.8));
        assert_eq!(player.sample(0.5)[0].weight, 1.0);
    }

    #[test]
    fn finished_cues_can_be_discarded() {
        let mut player = VisemePlayer::default();
        player.schedule(cue("O", 0.0, 0.2, 1.0));
        player.discard_finished(0.2);
        assert!(player.cues.is_empty());
    }
}
