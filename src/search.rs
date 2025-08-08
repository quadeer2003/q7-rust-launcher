use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::PathBuf;
use std::process::Command;

pub fn fuzzy_score(query: &str, candidate: &str) -> Option<i64> {
    let m = SkimMatcherV2::default();
    m.fuzzy_match(candidate, query)
}

pub fn fd_search(query: &str, limit: usize) -> Vec<PathBuf> {
    // Prefer `fd` (fd-find). Fallback to `rg --files | rg query` could be added later.
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let mut cmd = Command::new("fd");
    cmd.arg("--hidden").arg("--follow").arg("--max-results").arg(limit.to_string()).arg(query).arg(&home);
    if let Ok(out) = cmd.output() {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            return s.lines().map(|l| PathBuf::from(l)).collect();
        }
    }
    vec![]
}
