# FastEncode

High-performance character encoding conversion library and CLI for Rust, designed for enterprise and mainframe data processing.

---

## Features
- **Zero-copy conversions** with pre-computed translation tables
- **SIMD vectorization** for bulk data processing
- **Enterprise encodings**: EBCDIC, Windows code pages, ISO-8859, Mac, DOS, and Asian formats (scaffolded)
- **Streaming support** for large datasets
- **Thread-safe** operations
- **Comprehensive error handling**
- **Encoding detection** using BOM and statistical analysis

---

## Quick Start

### Library Usage
```rust
use fast_encode::{Encoding, Translator};

// Create a translator from EBCDIC to UTF-8
let translator = Translator::new(Encoding::EBCDIC_037, Encoding::UTF8).unwrap();
let ebcdic_data = &[0xC8, 0xC5, 0xD3, 0xD3, 0xD6]; // "HELLO"
let utf8_result = translator.convert(ebcdic_data).unwrap();
assert_eq!(std::str::from_utf8(&utf8_result).unwrap(), "HELLO");
```

### CLI Usage
```
cargo run --release -- convert -i input.txt -o output.txt --from WINDOWS_1252 --to UTF8
```

---

## Supported Encodings
- **Unicode**: UTF-8, UTF-16LE/BE
- **Windows**: 1250, 1251, 1252, etc.
- **ISO-8859**: 1, 2, 15, etc.
- **DOS/OEM**: CP437, CP850, etc.
- **Macintosh**: Mac Roman, Mac Cyrillic
- **EBCDIC**: 037, 500, 1047
- **Asian (scaffolded)**: Shift_JIS, EUC-JP, GB2312, BIG5, EUC-KR

---

## Encoding Detection

Detect encoding of binary/text data using BOM and statistical heuristics:
```rust
use fast_encode::detection::EncodingDetector;
let detector = EncodingDetector::new();
let result = detector.detect(b"Hello, World!");
println!("Detected: {:?}, confidence: {}", result.encoding, result.confidence);
```

---

## Adding Custom Encodings

1. **Add a new variant to the `Encoding` enum in `src/lib.rs`.**
2. **Create a static table in `src/tables.rs`:**
   - For single-byte encodings, define a `[Option<char>; 256]` array mapping each byte to a Unicode character.
   - For multi-byte encodings, document the need for a dedicated conversion function.
3. **Update `get_encoding_chars` in `src/tables.rs`** to return your new table.
4. **Implement conversion logic** in `src/multibyte.rs` if needed.
5. **Add tests** in `src/lib.rs` or a dedicated test module.

---

## Error Handling

All conversion and detection operations return a custom `Result<T, Error>` type. Errors include:
- Unmappable source/target bytes
- Invalid input data
- Unsupported conversions

---

## Streaming & Performance

- Use `StreamingTranslator` for large datasets.
- SIMD acceleration is available with the `simd` feature flag.

---

## Asian Encodings

Asian encodings are scaffolded and need real byte-to-Unicode tables. See:
- [ICU Project](https://github.com/unicode-org/icu)
- [Python encodings](https://github.com/python/cpython/tree/main/Lib/encodings)
- [Wikipedia: Character encoding](https://en.wikipedia.org/wiki/Character_encoding)

---

## Contributing

Pull requests for new encodings, bug fixes, and documentation are welcome!

---

## License

MIT
