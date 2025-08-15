use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::PathBuf;
use std::process::Command;

pub fn fuzzy_score(query: &str, candidate: &str) -> Option<i64> {
    let m = SkimMatcherV2::default();
    m.fuzzy_match(candidate, query)
}

pub fn fd_search(query: &str, limit: usize) -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        if query.is_empty() { return vec![]; }
        
        let mut results = Vec::new();
        
        // Fast search in common directories first
        let search_dirs = vec![
            std::env::var("USERPROFILE").unwrap_or_default(),
            format!("{}\\Desktop", std::env::var("USERPROFILE").unwrap_or_default()),
            format!("{}\\Documents", std::env::var("USERPROFILE").unwrap_or_default()),
            format!("{}\\Downloads", std::env::var("USERPROFILE").unwrap_or_default()),
        ];
        
        // Search directories with basic file system traversal (much faster than PowerShell)
        for dir in search_dirs {
            if results.len() >= limit { break; }
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    if results.len() >= limit { break; }
                    let path = entry.path();
                    if let Some(name) = path.file_name() {
                        if let Some(name_str) = name.to_str() {
                            if name_str.to_lowercase().contains(&query.to_lowercase()) {
                                results.push(path);
                            }
                        }
                    }
                }
            }
        }
        
        // Quick PATH search using where.exe if we need more results
        if results.len() < limit / 2 {
            if let Ok(out) = Command::new("where").arg(query).output() {
                if out.status.success() {
                    let s = String::from_utf8_lossy(&out.stdout);
                    for line in s.lines().take(limit - results.len()) {
                        let line = line.trim();
                        if !line.is_empty() {
                            results.push(PathBuf::from(line));
                        }
                    }
                }
            }
        }
        
        results.truncate(limit);
        results
    }
    #[cfg(not(windows))]
    {
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
}
