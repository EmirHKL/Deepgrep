mod cli;
mod index;
mod search;
mod output;
mod fuzzy;
mod ast;
mod watcher;

use clap::Parser;
use cli::{Cli, Commands};
use std::time::Instant;

fn main() {
    let cli = Cli::parse();
    let start = Instant::now();

    match cli.command {
        Some(Commands::Index { path }) => {
            let path = path.unwrap_or_else(|| ".".to_string());
            println!("🔍 Trigram indeksi oluşturuluyor: {}", path);
            match index::build_index(&path) {
                Ok(stats) => {
                    println!(
                        "✅ İndeks hazır: {} dosya, {} trigram, {:.2}s",
                        stats.file_count,
                        stats.trigram_count,
                        start.elapsed().as_secs_f64()
                    );
                }
                Err(e) => eprintln!("❌ İndeks hatası: {}", e),
            }
        }

        Some(Commands::Clean) => {
            match index::clean_index() {
                Ok(_) => println!("🗑️  İndeks silindi."),
                Err(e) => eprintln!("❌ {}", e),
            }
        }

        Some(Commands::Watch { path }) => {
            let path = path.unwrap_or_else(|| ".".to_string());
            // Önce indeks yoksa oluştur
            if !index::index_exists() {
                println!("🔍 İlk indeks oluşturuluyor...");
                match index::build_index(&path) {
                    Ok(stats) => println!(
                        "✅ İndeks hazır: {} dosya, {} trigram",
                        stats.file_count, stats.trigram_count
                    ),
                    Err(e) => eprintln!("❌ İndeks hatası: {}", e),
                }
            }
            watcher::watch_and_reindex(&path);
        }

        None => {
            let pattern = match &cli.pattern {
                Some(p) => p.clone(),
                None => {
                    eprintln!("❌ Pattern girilmedi. Kullanım: dg <pattern> [yol]");
                    std::process::exit(1);
                }
            };

            let path = cli.path.as_deref().unwrap_or(".");

            let opts = search::SearchOptions {
                pattern: pattern.clone(),
                path: path.to_string(),
                case_insensitive: cli.ignore_case,
                fuzzy: cli.fuzzy,
                fuzzy_threshold: cli.fuzzy_threshold,
                context_lines: cli.context.unwrap_or(0),
                show_ast_context: cli.ast,
                use_index: !cli.no_index,
                max_results: cli.max_results,
                file_type: cli.file_type.clone(),
            };

            match search::run_search(&opts) {
                Ok(results) => {
                    let elapsed = start.elapsed().as_secs_f64();
                    output::print_results(&results, &opts);
                    output::print_summary(&results, elapsed);
                }
                Err(e) => {
                    eprintln!("❌ Arama hatası: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}