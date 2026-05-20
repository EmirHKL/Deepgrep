use std::fs;
use std::io::{self, Read};
use rayon::prelude::*;
use regex::{Regex, RegexBuilder};
use ignore::WalkBuilder;
use std::path::Path;

use crate::index;
use crate::fuzzy::FuzzySearcher;
use crate::ast;

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub pattern: String,
    pub path: String,
    pub case_insensitive: bool,
    pub fuzzy: bool,
    pub fuzzy_threshold: i64,
    pub context_lines: usize,
    pub show_ast_context: bool,
    pub use_index: bool,
    pub max_results: Option<usize>,
    pub file_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub line_number: usize,
    pub line: String,
    pub fuzzy_score: Option<i64>,
    pub match_ranges: Vec<(usize, usize)>,
}

#[derive(Debug, Clone)]
pub struct FileResult {
    pub path: String,
    pub matches: Vec<Match>,
    pub context_before: Vec<(usize, String)>,
    pub context_after: Vec<(usize, String)>,
    pub ast_context: Option<String>,
}

pub struct SearchResults {
    pub file_results: Vec<FileResult>,
    pub total_matches: usize,
    pub files_searched: usize,
    pub used_index: bool,
    pub index_candidates: usize,
}

pub fn run_search(opts: &SearchOptions) -> io::Result<SearchResults> {
    let (files_to_search, used_index, index_candidates) = gather_files(opts)?;
    let files_searched = files_to_search.len();

    let regex = if !opts.fuzzy {
        Some(
            RegexBuilder::new(&opts.pattern)
                .case_insensitive(opts.case_insensitive)
                .build()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?,
        )
    } else {
        None
    };

    let fuzzy_searcher = if opts.fuzzy {
        Some(FuzzySearcher::new(opts.fuzzy_threshold))
    } else {
        None
    };

    let file_results: Vec<FileResult> = files_to_search
        .par_iter()
        .filter_map(|path| {
            search_file(path, opts, regex.as_ref(), fuzzy_searcher.as_ref())
        })
        .collect();

    let mut file_results: Vec<FileResult> = file_results
        .into_iter()
        .filter(|r| !r.matches.is_empty())
        .collect();

    file_results.sort_by(|a, b| a.path.cmp(&b.path));

    let total_matches: usize = file_results.iter().map(|r| r.matches.len()).sum();

    if let Some(max) = opts.max_results {
        let mut count = 0;
        file_results.retain(|r| {
            if count >= max {
                return false;
            }
            count += r.matches.len();
            true
        });
    }

    Ok(SearchResults {
        file_results,
        total_matches,
        files_searched,
        used_index,
        index_candidates,
    })
}

fn gather_files(opts: &SearchOptions) -> io::Result<(Vec<String>, bool, usize)> {
    if opts.use_index && index::index_exists() {
        if let Some(idx) = index::load_index() {
            let candidates = idx.candidates_for(&opts.pattern);
            let candidate_count = candidates.len();

            let files: Vec<String> = candidates
                .into_iter()
                .map(|i| idx.files[i].clone())
                .filter(|path| {
                    if let Some(ref ft) = opts.file_type {
                        path.ends_with(&format!(".{}", ft))
                    } else {
                        true
                    }
                })
                .collect();

            return Ok((files, true, candidate_count));
        }
    }

    let files: Vec<String> = WalkBuilder::new(&opts.path)
        .hidden(false)
        .ignore(true)
        .git_ignore(true)
        .build()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter(|e| {
            if let Some(ref ft) = opts.file_type {
                e.path().extension()
                    .and_then(|s| s.to_str())
                    .map(|ext| ext == ft.as_str())
                    .unwrap_or(false)
            } else {
                true
            }
        })
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();

    let count = files.len();
    Ok((files, false, count))
}

fn search_file(
    path: &str,
    opts: &SearchOptions,
    regex: Option<&Regex>,
    fuzzy: Option<&FuzzySearcher>,
) -> Option<FileResult> {
    let mut content = String::new();
    fs::File::open(path).ok()?.read_to_string(&mut content).ok()?;

    let lines: Vec<&str> = content.lines().collect();
    let mut matches = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if let Some(re) = regex {
            if let Some(m) = re.find(line) {
                matches.push(Match {
                    line_number: i + 1,
                    line: line.to_string(),
                    fuzzy_score: None,
                    match_ranges: vec![(m.start(), m.end())],
                });
            }
        } else if let Some(fz) = fuzzy {
            if let Some((score, indices)) = fz.match_indices(&opts.pattern, line) {
                let ranges = indices_to_ranges(&indices);
                matches.push(Match {
                    line_number: i + 1,
                    line: line.to_string(),
                    fuzzy_score: Some(score),
                    match_ranges: ranges,
                });
            }
        }
    }

    if matches.is_empty() {
        return None;
    }

    let (context_before, context_after) = if opts.context_lines > 0 {
        build_context(&lines, &matches, opts.context_lines)
    } else {
        (vec![], vec![])
    };

    // Gerçek tree-sitter AST bağlamı
    let ast_context = if opts.show_ast_context {
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        ast::get_ast_context(&content, matches[0].line_number, ext)
            .map(|ctx| format!("{} {}", ctx.kind, ctx.name))
            .or_else(|| find_ast_context_heuristic(&lines, matches[0].line_number))
    } else {
        None
    };

    Some(FileResult {
        path: path.to_string(),
        matches,
        context_before,
        context_after,
        ast_context,
    })
}

fn indices_to_ranges(indices: &[usize]) -> Vec<(usize, usize)> {
    if indices.is_empty() {
        return vec![];
    }
    let mut ranges = Vec::new();
    let mut start = indices[0];
    let mut prev = indices[0];
    for &idx in &indices[1..] {
        if idx != prev + 1 {
            ranges.push((start, prev + 1));
            start = idx;
        }
        prev = idx;
    }
    ranges.push((start, prev + 1));
    ranges
}

fn build_context(
    lines: &[&str],
    matches: &[Match],
    ctx: usize,
) -> (Vec<(usize, String)>, Vec<(usize, String)>) {
    if matches.is_empty() {
        return (vec![], vec![]);
    }
    let first = matches[0].line_number - 1;
    let last = matches.last().unwrap().line_number - 1;

    let before_start = first.saturating_sub(ctx);
    let after_end = (last + ctx + 1).min(lines.len());

    let before = (before_start..first)
        .map(|i| (i + 1, lines[i].to_string()))
        .collect();

    let after = ((last + 1)..after_end)
        .map(|i| (i + 1, lines[i].to_string()))
        .collect();

    (before, after)
}

// Desteklenmeyen diller için yedek heuristik
fn find_ast_context_heuristic(lines: &[&str], match_line: usize) -> Option<String> {
    let patterns = [
        r"^\s*(pub\s+)?(async\s+)?fn\s+\w+",
        r"^\s*(pub\s+)?struct\s+\w+",
        r"^\s*(pub\s+)?impl(\s+\w+)?\s+\w+",
        r"^\s*(pub\s+)?enum\s+\w+",
        r"^\s*(pub\s+)?trait\s+\w+",
        r"^\s*def\s+\w+",
        r"^\s*class\s+\w+",
        r"^\s*async\s+def\s+\w+",
        r"^\s*(export\s+)?(async\s+)?function\s+\w+",
        r"^\s*func\s+\w+",
    ];

    let compiled: Vec<Regex> = patterns
        .iter()
        .filter_map(|p| Regex::new(p).ok())
        .collect();

    let start = match_line.saturating_sub(1);
    for i in (0..start).rev() {
        let line = lines[i];
        for re in &compiled {
            if re.is_match(line) {
                return Some(line.trim().to_string());
            }
        }
        if i + 50 < start && line.trim().is_empty() {
            break;
        }
    }
    None
}