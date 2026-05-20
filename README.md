# deepgrep (`dg`) 🔍

> Rust ile yazılmış, **trigram indeksleme**, **AST bağlam gösterimi** ve **bulanık eşleşme** özelliklerine sahip akıllı bir kod arama aracı.

---

## ripgrep'e Göre Farkları

| Özellik | ripgrep | deepgrep |
|---|---|---|
| Arama algoritması | Regex ile ham metin tarama | Regex + trigram indeks + bulanık eşleşme |
| İndeks | ❌ Her seferinde tüm dosyaları tarar | ✅ Kalıcı trigram indeksi |
| Tekrar eden aramalar | Her seferinde O(n) | İndeks sonrası O(1) |
| Bulanık eşleşme | ❌ | ✅ Yazım hatalarını tolere eder |
| AST bağlamı | ❌ Sadece eşleşen satırı gösterir | ✅ Hangi fonksiyon/struct içinde olduğunu gösterir |
| Mimari | Tek geçişli SIMD regex | Trigram ön filtre → regex doğrulama |

---

## Trigram İndeksi Nasıl Çalışır?

Trigram, art arda gelen 3 karakterlik dizilerdir. Örneğin "merhaba" şu trigramları üretir: mer, erh, rha, hab, aba.

deepgrep, ilk çalıştırmada her dosyayı trigramlarına göre indeksler ve bunu diske kaydeder. Sonraki aramalarda sorgu trigramlara ayrılır, sadece ilgili dosyalar diskten okunur. Bu yaklaşım Google Code Search ve Zoekt gibi endüstri standardı araçlarda da kullanılmaktadır.

Örnek: 50.000 dosyalık bir projede "parse_args" aranırken:
- ripgrep: 50.000 dosyanın tamamını okuyup tarar
- deepgrep: trigram indeksini sorgular, az sayıda aday dosya bulur, yalnızca onları tarar

---

## AST Bağlamı

deepgrep, eşleşen satırdan geriye doğru tarayarak en yakın kapsayıcı bildirimi bulur.

Örnek çıktı:
  src/cli.rs
    ↳ içinde: pub fn parse_args(input: &str) -> Args
      42 | let args = parse_args(input);

ripgrep bu özelliği sunmaz çünkü dosyanın fonksiyon veya struct sınırlarını bilmek
için dilin sözdizimini parse etmek gerekir. Bu, ripgrep'in dilden bağımsız
ve satır odaklı ham metin okuma mimarisine ters düşer.

Desteklenen diller: Rust, Python, JavaScript/TypeScript, Go, C/C++

---

## Bulanık Eşleşme

skim bulanık eşleşme algoritması kullanılır (fzf ile aynı). Yazım hatalarıyla bile eşleşme bulur:

  dg "bağlanti" --fuzzy    # "bağlantı" bulur
  dg "fonksiyo" --fuzzy    # "fonksiyon" bulur

ripgrep bu özelliği sunmaz çünkü kesin kurallara dayanan regex standartlarını
bozar ve SIMD tabanlı arama motorunun çalışma prensibini yavaşlatır.

---

## Kurulum

Gereksinimler: Rust araç zinciri 1.70 ve üzeri (https://rustup.rs)

Kaynak koddan derleme:

  git clone https://github.com/EmirHKL/Deepgrep
  cd Deepgrep
  cargo build --release

Binary .\target\release\dg.exe konumunda oluşur.

---

## Kullanım

Temel arama:
  dg "fn main"           # mevcut dizinde ara
  dg "TODO" src/         # belirli dizinde ara
  dg "hata" -i           # büyük/küçük harf duyarsız
  dg "fn parse" -t rs    # sadece Rust dosyaları
  dg "config" -m 20      # en fazla 20 sonuç

Bağlam satırları:
  dg "panic!" -C 3       # eşleşme etrafında 3 satır göster

Bulanık arama:
  dg "bağlanti" --fuzzy
  dg "hataYonet" --fuzzy --fuzzy-threshold 50

AST bağlamı:
  dg "unwrap()" --ast    # hangi fonksiyonda olduğunu gösterir

Trigram indeksi:
  dg index               # indeksi bir kez oluştur
  dg "SearchOptions"     # sonraki aramalar daha az dosya tarar
  dg clean               # indeksi sil

---

## Mimari

  deepgrep/
  ├── src/
  │   ├── main.rs     # Giriş noktası, CLI yönlendirme
  │   ├── cli.rs      # Argüman tanımları (clap)
  │   ├── index.rs    # Trigram indeksi: oluştur, yükle, sorgula
  │   ├── search.rs   # Arama motoru: dosya toplama, regex, paralel tarama
  │   ├── fuzzy.rs    # Bulanık eşleşme (skim algoritması)
  │   └── output.rs   # Renkli terminal çıktısı
  ├── Cargo.toml
  └── README.md

Temel tasarım kararları:
- Trigram ön filtresi: Her dosyayı okumak yerine önce indeks sorgulanır,
  yalnızca aday dosyalar taranır
- Paralellik: Dosya taraması rayon ile paralelleştirilmiştir
- gitignore desteği: .gitignore ve gizli dosyalar otomatik atlanır
- Kalıcı indeks: .deepgrep/index.json olarak saklanır,
  çalıştırmalar arasında korunur

---

## Benzer Araçlarla Karşılaştırma

| Araç      | Dil  | İndeks | Bulanık | AST |
|-----------|------|--------|---------|-----|
| grep      | C    | ❌     | ❌      | ❌  |
| ripgrep   | Rust | ❌     | ❌      | ❌  |
| ast-grep  | Rust | ❌     | ❌      | ✅  |
| Zoekt     | Go   | ✅     | ❌      | ❌  |
| deepgrep  | Rust | ✅     | ✅      | ✅  |

deepgrep, bu tablodaki araçlar arasında üç özelliği birden sunan tek araçtır.

---

## Lisans

MIT