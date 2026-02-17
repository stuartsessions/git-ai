use crate::authorship::attribution_tracker::Attribution;
use crate::authorship::working_log::Checkpoint;

mod tmp_repo;
pub use tmp_repo::{TmpFile, TmpRepo};

// @todo move this acunniffe
/// Sanitized checkpoint representation for deterministic snapshots
#[allow(dead_code)]
#[derive(Debug)]
pub struct SnapshotCheckpoint {
    author: String,
    has_agent: bool,
    agent_tool: Option<String>,
    entries: Vec<SnapshotEntry>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct SnapshotEntry {
    file: String,
    attributions: Vec<Attribution>,
}

#[allow(dead_code)]
pub fn snapshot_checkpoints(checkpoints: &[Checkpoint]) -> Vec<SnapshotCheckpoint> {
    let mut snapshots: Vec<SnapshotCheckpoint> = checkpoints
        .iter()
        .map(|cp| {
            let mut entries: Vec<SnapshotEntry> = cp
                .entries
                .iter()
                .map(|e| {
                    let mut attributions = e.attributions.clone();
                    // Sort attributions by start position, then end position, then author_id for determinism
                    attributions.sort_by(|a, b| {
                        a.start
                            .cmp(&b.start)
                            .then_with(|| a.end.cmp(&b.end))
                            .then_with(|| a.author_id.cmp(&b.author_id))
                    });

                    SnapshotEntry {
                        file: e.file.clone(),
                        attributions,
                    }
                })
                .collect();

            // Sort entries by file name for deterministic ordering
            entries.sort_by(|a, b| a.file.cmp(&b.file));

            SnapshotCheckpoint {
                author: cp.author.clone(),
                has_agent: cp.agent_id.is_some(),
                agent_tool: cp.agent_id.as_ref().map(|a| a.tool.clone()),
                entries,
            }
        })
        .collect();

    // Sort checkpoints by author name, then by first file name, then by first attribution start position
    // for deterministic ordering
    snapshots.sort_by(|a, b| {
        // First sort by author
        match a.author.cmp(&b.author) {
            std::cmp::Ordering::Equal => {
                // If authors are equal, sort by first file name
                let a_file = a.entries.first().map(|e| e.file.as_str()).unwrap_or("");
                let b_file = b.entries.first().map(|e| e.file.as_str()).unwrap_or("");
                match a_file.cmp(b_file) {
                    std::cmp::Ordering::Equal => {
                        // If files are equal, sort by first attribution start position
                        let a_start = a
                            .entries
                            .first()
                            .and_then(|e| e.attributions.first())
                            .map(|attr| attr.start)
                            .unwrap_or(0);
                        let b_start = b
                            .entries
                            .first()
                            .and_then(|e| e.attributions.first())
                            .map(|attr| attr.start)
                            .unwrap_or(0);
                        a_start.cmp(&b_start)
                    }
                    other => other,
                }
            }
            other => other,
        }
    });

    snapshots
}

/// Reset mode for git reset command
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum ResetMode {
    Hard,
    Soft,
    Mixed,
    Merge,
    Keep,
}

#[allow(dead_code)]
const ALPHABET: &str = "A
B
C
D
E
F
G
H
I
J
K
L
M
N
O
P
Q
R
S
T
U
V
W
X
Y
Z";

#[allow(dead_code)]
const LINES: &str = "1
2
3
4
5
6
7
8
9
10
11
12
13
14
15
16
17
18
19
20
21
22
23
24
25
26
27
28
29
30
31
32
33";
