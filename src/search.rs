use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::PathBuf;
use std::process::Command;

pub fn fuzzy_score(query: &str, candidate: &str) -> Option<i64> {
    let m = SkimMatcherV2::default();
    m.fuzzy_match(candidate, query)
}

pub fn fd_search(query: &str, limit: usize) -> Vec<PathBuf> {
    // Use `fd` and skip common heavy directories; limit results for responsiveness.
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let mut cmd = Command::new("fd");
    cmd.arg("--hidden")
        .arg("--follow")
        .arg("--exclude").arg(".git")
        .arg("--exclude").arg("node_modules")
        .arg("--exclude").arg("target")
        .arg("--max-results").arg(limit.to_string())
        .arg(query)
        .arg(&home);
    if let Ok(out) = cmd.output() {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            return s.lines().map(|l| PathBuf::from(l)).collect();
        }
    }
    vec![]
}
