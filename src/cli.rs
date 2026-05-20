use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "dg",
    version = "0.1.0",
    about = "deepgrep — trigram indeks, AST bağlamı ve bulanık eşleşme ile akıllı kod arama",
    after_help = "ÖRNEKLER:
    dg 'fn main'                    # mevcut dizinde ara
    dg 'TODO' src/ -C 2             # 2 satır bağlam göster
    dg 'bağlanti' --fuzzy           # bulanık eşleşme
    dg 'fn parse' --ast             # AST bağlamı göster
    dg index                        # indeks oluştur
    dg watch                        # dosya değişikliklerini izle
    dg 'fn main' --type rs          # sadece Rust dosyaları"
)]
pub struct Cli {
    /// Aranacak ifade
    pub pattern: Option<String>,

    /// Aranacak dizin (varsayılan: mevcut dizin)
    pub path: Option<String>,

    /// Büyük/küçük harf duyarsız arama
    #[arg(short = 'i', long = "ignore-case")]
    pub ignore_case: bool,

    /// Bulanık eşleşmeyi etkinleştir
    #[arg(short = 'f', long = "fuzzy")]
    pub fuzzy: bool,

    /// Bulanık eşleşme eşiği 0-100 (varsayılan: 60)
    #[arg(long = "fuzzy-threshold", default_value = "60")]
    pub fuzzy_threshold: i64,

    /// Eşleşme etrafında N satır bağlam göster
    #[arg(short = 'C', long = "context")]
    pub context: Option<usize>,

    /// AST bağlamını göster (hangi fonksiyon/struct içinde)
    #[arg(short = 'A', long = "ast")]
    pub ast: bool,

    /// İndeks olsa bile kullanma
    #[arg(long = "no-index")]
    pub no_index: bool,

    /// Sonuç sayısını sınırla
    #[arg(short = 'm', long = "max-results")]
    pub max_results: Option<usize>,

    /// Dosya uzantısına göre filtrele (ör: rs, py, js)
    #[arg(short = 't', long = "type")]
    pub file_type: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Trigram indeksi oluştur
    Index {
        /// İndekslenecek dizin (varsayılan: mevcut dizin)
        path: Option<String>,
    },
    /// Dosya değişikliklerini izle, indeksi otomatik güncelle
    Watch {
        /// İzlenecek dizin (varsayılan: mevcut dizin)
        path: Option<String>,
    },
    /// İndeksi sil
    Clean,
}