use colored::*;
use crate::search::{FileResult, Match, SearchOptions, SearchResults};

pub fn print_results(results: &SearchResults, opts: &SearchOptions) {
    if results.file_results.is_empty() {
        println!("{}", "No matches found.".yellow());
        return;
    }

    for file_result in &results.file_results {
        print_file_result(file_result, opts);
    }
}

fn print_file_result(result: &FileResult, opts: &SearchOptions) {
    println!("\n{}", result.path.bold().underline().cyan());

    if let Some(ref ctx) = result.ast_context {
        println!("{} {}", "  ↳ in:".dimmed(), ctx.bright_yellow());
    }

    for (lineno, line) in &result.context_before {
        println!(
            "{}{}{}",
            format!("{:>6}", lineno).dimmed(),
            "│".dimmed(),
            format!(" {}", line).dimmed()
        );
    }

    for m in &result.matches {
        print_match(m, opts);
    }

    for (lineno, line) in &result.context_after {
        println!(
            "{}{}{}",
            format!("{:>6}", lineno).dimmed(),
            "│".dimmed(),
            format!(" {}", line).dimmed()
        );
    }
}

fn print_match(m: &Match, opts: &SearchOptions) {
    let lineno_str = format!("{:>6}", m.line_number).green().bold();
    let separator = "│".green();

    let highlighted = highlight_ranges(&m.line, &m.match_ranges, opts.fuzzy);

    let score_tag = if let Some(score) = m.fuzzy_score {
        format!(" [~{}]", score).bright_black().to_string()
    } else {
        String::new()
    };

    println!("{}{} {}{}", lineno_str, separator, highlighted, score_tag);
}

fn highlight_ranges(line: &str, ranges: &[(usize, usize)], is_fuzzy: bool) -> String {
    if ranges.is_empty() {
        return line.to_string();
    }

    let mut result = String::new();
    let mut pos = 0;

    for &(start, end) in ranges {
        let start = start.min(line.len());
        let end = end.min(line.len());

        if pos < start {
            result.push_str(&line[pos..start]);
        }
        if start < end {
            let matched = &line[start..end];
            if is_fuzzy {
                result.push_str(&matched.bright_magenta().bold().to_string());
            } else {
                result.push_str(&matched.bright_red().bold().to_string());
            }
        }
        pos = end;
    }

    if pos < line.len() {
        result.push_str(&line[pos..]);
    }

    result
}

pub fn print_summary(results: &SearchResults, elapsed_secs: f64) {
    println!();
    let index_info = if results.used_index {
        format!(
            " {} index hit ({} candidates)",
            "⚡",
            results.index_candidates
        )
    } else {
        String::new()
    };

    println!(
        "{}",
        format!(
            "  {} match{} in {} file{} — searched {} file{} in {:.3}s{}",
            results.total_matches,
            if results.total_matches == 1 { "" } else { "es" },
            results.file_results.len(),
            if results.file_results.len() == 1 { "" } else { "s" },
            results.files_searched,
            if results.files_searched == 1 { "" } else { "s" },
            elapsed_secs,
            index_info,
        )
        .bright_black()
    );
}