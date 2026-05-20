use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::index;

pub fn watch_and_reindex(root: &str) {
    println!("👁️  Dosya değişiklikleri izleniyor: {}", root);
    println!("    İndeks otomatik güncellenecek. Çıkmak için Ctrl+C\n");

    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
    watcher
        .watch(Path::new(root), RecursiveMode::Recursive)
        .unwrap();

    let mut last_reindex = Instant::now();
    let debounce = Duration::from_secs(2);

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                let is_relevant = matches!(
                    event.kind,
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                );

                if !is_relevant {
                    continue;
                }

                if last_reindex.elapsed() < debounce {
                    continue;
                }

                for path in &event.paths {
                    let path_str = path.to_string_lossy();
                    if path_str.contains("target") || path_str.contains(".deepgrep") {
                        continue;
                    }
                    println!("📝 Değişiklik algılandı: {}", path_str);
                }

                print!("🔄 İndeks güncelleniyor...");
                match index::build_index(root) {
                    Ok(stats) => {
                        println!(
                            " ✅ {} dosya, {} trigram",
                            stats.file_count, stats.trigram_count
                        );
                    }
                    Err(e) => println!(" ❌ Hata: {}", e),
                }

                last_reindex = Instant::now();
            }
            Ok(Err(e)) => eprintln!("İzleme hatası: {}", e),
            Err(e) => {
                eprintln!("Kanal hatası: {}", e);
                break;
            }
        }
    }
}