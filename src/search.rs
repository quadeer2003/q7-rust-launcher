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
        // Use PowerShell to search top-level home and Desktop/Documents plus PATH executables.
        let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOMEPATH")).unwrap_or_default();
        let ps_query = format!(
            "$l=@();$p='{}';if(Test-Path $p){{$l+=Get-ChildItem -ErrorAction SilentlyContinue -Depth 2 -Path $p -Filter '*{}*'}};$env:PATH.Split(';')|ForEach-Object{{if(Test-Path $_){{$l+=Get-ChildItem -ErrorAction SilentlyContinue -Path $_ -Filter '*{}*'}}}};$l|Select -First {} -ExpandProperty FullName",
            home.replace('\\', "/"), query, query, limit
        );
        if let Ok(out) = Command::new("powershell").arg("-NoProfile").arg("-Command").arg(ps_query).output() {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout);
                let mut paths: Vec<PathBuf> = s.lines().filter(|l| !l.trim().is_empty()).map(|l| PathBuf::from(l.trim())).collect();
                if paths.is_empty() {
                    // fallback to where.exe
                    if let Ok(out2) = Command::new("where").arg(query).output() {
                        if out2.status.success() {
                            let s2 = String::from_utf8_lossy(&out2.stdout);
                            paths.extend(s2.lines().filter(|l| !l.trim().is_empty()).map(|l| PathBuf::from(l.trim())));
                        }
                    }
                }
                paths.truncate(limit);
                return paths;
            }
        }
        vec![]
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
