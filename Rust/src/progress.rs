use crate::levels::CAMPAIGN_COUNT;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    pub selected: usize,
    pub unlocked: usize,
}

impl Progress {
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut fields = text.split_whitespace();
        if fields.next() != Some("ironjump-v1") {
            return Err("Unrecognized progress file".into());
        }
        let selected = fields
            .next()
            .and_then(|s| s.parse().ok())
            .ok_or("Invalid selected level")?;
        let unlocked = fields
            .next()
            .and_then(|s| s.parse::<usize>().ok())
            .ok_or("Invalid unlocked level")?;
        if fields.next().is_some() {
            return Err("Unexpected progress data".into());
        }
        Ok(Self {
            selected,
            unlocked: unlocked.min(CAMPAIGN_COUNT - 1),
        })
    }

    pub fn encode(self) -> String {
        format!("ironjump-v1 {} {}\n", self.selected, self.unlocked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_format_roundtrips_without_a_filesystem() {
        let progress = Progress {
            selected: 12,
            unlocked: 15,
        };
        assert_eq!(progress.encode(), "ironjump-v1 12 15\n");
        assert_eq!(Progress::parse(&progress.encode()).unwrap(), progress);
        assert_eq!(
            Progress::parse("ironjump-v1 1000 1000").unwrap().unlocked,
            29
        );
    }

    #[test]
    fn rejects_incomplete_unknown_and_trailing_progress_data() {
        for text in [
            "broken",
            "ironjump-v1",
            "ironjump-v1 0",
            "ironjump-v1 -1 0",
            "ironjump-v1 0 nope",
            "ironjump-v1 0 0 extra",
        ] {
            assert!(Progress::parse(text).is_err(), "accepted {text:?}");
        }
    }
}
