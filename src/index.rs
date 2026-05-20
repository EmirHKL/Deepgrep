use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};

const INDEX_DIR: &str = ".deepgrep";
const INDEX_FILE: &str = ".deepgrep/index.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct TrigramIndex {
    pub map: HashMap<String, Vec<usize>>,
    pub files: Vec<String>,
    pub mtimes: HashMap<String, u64>,
}

pub struct IndexStats {
    pub file_count: usize,
    pub trigram_count: usize,
}

impl TrigramIndex {
    pub fn new() -> Self {
        TrigramIndex {
            map: HashMap::new(),
            files: Vec::new(),
            mtimes: HashMap::new(),
        }
    }

    pub fn extract_trigrams(text: &str) -> Vec<String> {
        let chars: Vec<char> = text.chars().collect();
        let mut trigrams = Vec::new();
        if chars.len() < 3 {
            return trigrams;
        }
        for i in 0..chars.len() - 2 {
            let tri: String = chars[i..i + 3].iter().collect();
            if tri.chars().any(|c| !c.is_whitespace()) {
                trigrams.push(tri.to_lowercase());
            }
        }
        trigrams
    }

    pub fn candidates_for(&self, query: &str) -> Vec<usize> {
        let query_lower = query.to_lowercase();
        let trigrams = Self::extract_trigrams(&query_lower);

        if trigrams.is_empty() {
            return (0..self.files.len()).collect();
        }

        let mut sorted_trigrams = trigrams.clone();
        sorted_trigrams.sort_by_key(|t| {
            self.map.get(t).map(|v| v.len()).unwrap_or(0)
        });

        let first = match self.map.get(&sorted_trigrams[0]) {
            Some(list) => list.clone(),
            None => return vec![],
        };

        let mut candidates: Vec<usize> = first;

        for tri in &sorted_trigrams[1..] {
            match self.map.get(tri) {
                Some(list) => {
                    candidates = intersect(&candidates, list);
                    if candidates.is_empty() {
                        return vec![];
                    }
                }
                None => return vec![],
            }
        }

        candidates
    }
}

fn intersect(a: &[usize], b: &[usize]) -> Vec<usize> {
    let mut i = 0;
    let mut j = 0;
    let mut result = Vec::new();
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Equal => {
                result.push(a[i]);
                i += 1;
                j += 1;
            }
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
        }
    }
    result
}

pub fn build_index(root: &str) -> io::Result<IndexStats> {
    let mut index = TrigramIndex::new();
    let mut file_id = 0usize;

    let walker = WalkBuilder::new(root)
        .hidden(false)
        .ignore(true)
        .git_ignore(true)
        .build();

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }

        let path = entry.path().to_string_lossy().to_string();

        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if metadata.len() > 10 * 1024 * 1024 {
            continue;
        }

        let mut content = String::new();
        match fs::File::open(&path).and_then(|mut f| f.read_to_string(&mut content)) {
            Ok(_) => {}
            Err(_) => continue,
        }

        let mtime = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        index.files.push(path.clone());
        index.mtimes.insert(path.clone(), mtime);

        let trigrams = TrigramIndex::extract_trigrams(&content);
        let mut seen = std::collections::HashSet::new();
        for tri in trigrams {
            if seen.insert(tri.clone()) {
                index.map.entry(tri).or_default().push(file_id);
            }
        }

        file_id += 1;
    }

    for list in index.map.values_mut() {
        list.sort_unstable();
    }

    let stats = IndexStats {
        file_count: index.files.len(),
        trigram_count: index.map.len(),
    };

    fs::create_dir_all(INDEX_DIR)?;
    let json = serde_json::to_string(&index)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(INDEX_FILE, json)?;

    Ok(stats)
}

pub fn load_index() -> Option<TrigramIndex> {
    let data = fs::read_to_string(INDEX_FILE).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn clean_index() -> io::Result<()> {
    if Path::new(INDEX_FILE).exists() {
        fs::remove_file(INDEX_FILE)?;
    }
    Ok(())
}

pub fn index_exists() -> bool {
    Path::new(INDEX_FILE).exists()
}