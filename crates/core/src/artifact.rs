use std::fmt::Write as _;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactHeader {
    pub version: u32,
    pub family: String,
    pub description: String,
}

impl ArtifactHeader {
    pub fn new(family: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            version: 1,
            family: family.into(),
            description: description.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchArtifact {
    pub header: ArtifactHeader,
    pub body: Vec<String>,
}

impl SearchArtifact {
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "version={}", self.header.version);
        let _ = writeln!(out, "family={}", self.header.family);
        let _ = writeln!(out, "description={}", self.header.description);
        for line in &self.body {
            let _ = writeln!(out, "{line}");
        }
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckpointState {
    pub version: u32,
    pub mode: String,
    pub length: usize,
    pub compression: usize,
    pub shard_index: usize,
    pub shard_count: usize,
    pub next_attempt: u64,
    pub matches_found: u64,
}

impl CheckpointState {
    pub fn new(
        mode: impl Into<String>,
        length: usize,
        compression: usize,
        shard_index: usize,
        shard_count: usize,
    ) -> Self {
        Self {
            version: 1,
            mode: mode.into(),
            length,
            compression,
            shard_index,
            shard_count,
            next_attempt: 0,
            matches_found: 0,
        }
    }

    pub fn to_text(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "version={}", self.version);
        let _ = writeln!(out, "mode={}", self.mode);
        let _ = writeln!(out, "length={}", self.length);
        let _ = writeln!(out, "compression={}", self.compression);
        let _ = writeln!(out, "shard_index={}", self.shard_index);
        let _ = writeln!(out, "shard_count={}", self.shard_count);
        let _ = writeln!(out, "next_attempt={}", self.next_attempt);
        let _ = writeln!(out, "matches_found={}", self.matches_found);
        out
    }

    pub fn from_text(text: &str) -> Result<Self, String> {
        let mut version = None;
        let mut mode = None;
        let mut length = None;
        let mut compression = None;
        let mut shard_index = None;
        let mut shard_count = None;
        let mut next_attempt = None;
        let mut matches_found = None;

        for line in text.lines().filter(|line| !line.trim().is_empty()) {
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("invalid checkpoint line: {line}"))?;
            match key.trim() {
                "version" => version = Some(value.trim().parse::<u32>().map_err(|e| e.to_string())?),
                "mode" => mode = Some(value.trim().to_string()),
                "length" => length = Some(value.trim().parse::<usize>().map_err(|e| e.to_string())?),
                "compression" => {
                    compression = Some(value.trim().parse::<usize>().map_err(|e| e.to_string())?)
                }
                "shard_index" => {
                    shard_index = Some(value.trim().parse::<usize>().map_err(|e| e.to_string())?)
                }
                "shard_count" => {
                    shard_count = Some(value.trim().parse::<usize>().map_err(|e| e.to_string())?)
                }
                "next_attempt" => {
                    next_attempt = Some(value.trim().parse::<u64>().map_err(|e| e.to_string())?)
                }
                "matches_found" => {
                    matches_found = Some(value.trim().parse::<u64>().map_err(|e| e.to_string())?)
                }
                other => return Err(format!("unknown checkpoint key: {other}")),
            }
        }

        Ok(Self {
            version: version.ok_or_else(|| "missing version".to_string())?,
            mode: mode.ok_or_else(|| "missing mode".to_string())?,
            length: length.ok_or_else(|| "missing length".to_string())?,
            compression: compression.ok_or_else(|| "missing compression".to_string())?,
            shard_index: shard_index.ok_or_else(|| "missing shard_index".to_string())?,
            shard_count: shard_count.ok_or_else(|| "missing shard_count".to_string())?,
            next_attempt: next_attempt.ok_or_else(|| "missing next_attempt".to_string())?,
            matches_found: matches_found.ok_or_else(|| "missing matches_found".to_string())?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::CheckpointState;

    #[test]
    fn checkpoint_roundtrip() {
        let original = CheckpointState {
            version: 1,
            mode: "lp".into(),
            length: 333,
            compression: 9,
            shard_index: 2,
            shard_count: 64,
            next_attempt: 10_000,
            matches_found: 7,
        };
        let reparsed = CheckpointState::from_text(&original.to_text()).expect("parse");
        assert_eq!(original, reparsed);
    }
}
