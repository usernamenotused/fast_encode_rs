//! Encoding detection algorithms using statistical analysis and heuristics
//!
//! This module provides sophisticated encoding detection capabilities for
//! automatically identifying the character encoding of binary data.

use crate::Encoding;

/// Result of encoding detection with confidence score
#[derive(Debug, Clone)]
pub struct DetectionResult {
    /// Most likely encoding
    pub encoding: Encoding,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Whether a BOM was detected
    pub bom_detected: bool,
    /// All candidate encodings with their scores
    pub candidates: Vec<(Encoding, f64)>,
}

/// Encoding detector using multiple detection methods
pub struct EncodingDetector {
    /// Maximum bytes to analyze for detection
    max_sample_size: usize,
}

impl Default for EncodingDetector {
    fn default() -> Self {
        Self {
            max_sample_size: 8192,
        }
    }
}

impl EncodingDetector {
    /// Create a new encoding detector
    pub fn new() -> Self {
        Self::default()
    }

    /// Create detector with custom sample size
    pub fn with_sample_size(max_sample_size: usize) -> Self {
        Self { max_sample_size }
    }

    /// Detect encoding of the given data
    pub fn detect(&self, data: &[u8]) -> DetectionResult {
        // Limit sample size
        let sample = if data.len() > self.max_sample_size {
            &data[..self.max_sample_size]
        } else {
            data
        };

        // Check for BOM first (highest confidence)
        if let Some((encoding, _bom_len)) = self.detect_bom(sample) {
            return DetectionResult {
                encoding,
                confidence: 1.0,
                bom_detected: true,
                candidates: vec![(encoding, 1.0)],
            };
        }

        // Statistical detection
        let mut candidates = Vec::new();

        // UTF-8 detection
        if let Some(confidence) = self.detect_utf8(sample) {
            candidates.push((Encoding::UTF8, confidence));
        }

        // UTF-16 detection
        if let Some((encoding, confidence)) = self.detect_utf16(sample) {
            candidates.push((encoding, confidence));
        }

        // ASCII detection
        if let Some(confidence) = self.detect_ascii(sample) {
            candidates.push((Encoding::ASCII, confidence));
        }

        // Windows code page detection
        candidates.extend(self.detect_windows_codepages(sample));

        // ISO detection
        candidates.extend(self.detect_iso_encodings(sample));

        // EBCDIC detection
        if let Some(confidence) = self.detect_ebcdic(sample) {
            candidates.push((Encoding::EBCDIC_037, confidence));
        }

        // DOS/OEM detection
        candidates.extend(self.detect_dos_codepages(sample));

        // Sort by confidence
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Return best match or ASCII as fallback
        let (encoding, confidence) = candidates
            .first()
            .copied()
            .unwrap_or((Encoding::ASCII, 0.5));

        DetectionResult {
            encoding,
            confidence,
            bom_detected: false,
            candidates,
        }
    }

    /// Detect BOM (Byte Order Mark)
    fn detect_bom(&self, data: &[u8]) -> Option<(Encoding, usize)> {
        if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
            Some((Encoding::UTF8, 3))
        } else if data.starts_with(&[0xFF, 0xFE]) {
            Some((Encoding::UTF16LE, 2))
        } else if data.starts_with(&[0xFE, 0xFF]) {
            Some((Encoding::UTF16BE, 2))
        } else {
            None
        }
    }

    /// Detect UTF-8 encoding
    fn detect_utf8(&self, data: &[u8]) -> Option<f64> {
        let mut valid_sequences = 0;
        let mut total_bytes = 0;
        let mut i = 0;

        while i < data.len() {
            let byte = data[i];

            if byte < 0x80 {
                // ASCII character
                i += 1;
                total_bytes += 1;
            } else if (byte & 0xE0) == 0xC0 {
                // 2-byte sequence
                if i + 1 < data.len() && (data[i + 1] & 0xC0) == 0x80 {
                    valid_sequences += 1;
                    total_bytes += 2;
                    i += 2;
                } else {
                    return None; // Invalid UTF-8
                }
            } else if (byte & 0xF0) == 0xE0 {
                // 3-byte sequence
                if i + 2 < data.len()
                    && (data[i + 1] & 0xC0) == 0x80
                    && (data[i + 2] & 0xC0) == 0x80
                {
                    valid_sequences += 1;
                    total_bytes += 3;
                    i += 3;
                } else {
                    return None; // Invalid UTF-8
                }
            } else if (byte & 0xF8) == 0xF0 {
                // 4-byte sequence
                if i + 3 < data.len()
                    && (data[i + 1] & 0xC0) == 0x80
                    && (data[i + 2] & 0xC0) == 0x80
                    && (data[i + 3] & 0xC0) == 0x80
                {
                    valid_sequences += 1;
                    total_bytes += 4;
                    i += 4;
                } else {
                    return None; // Invalid UTF-8
                }
            } else {
                return None; // Invalid UTF-8
            }
        }

        if total_bytes == 0 {
            return Some(0.5); // All ASCII, could be UTF-8
        }

        // Higher confidence if we found multi-byte sequences
        let multibyte_ratio = valid_sequences as f64 / total_bytes as f64;
        Some(0.7 + multibyte_ratio * 0.3)
    }

    /// Detect UTF-16 encoding
    fn detect_utf16(&self, data: &[u8]) -> Option<(Encoding, f64)> {
        if data.len() < 2 || data.len() % 2 != 0 {
            return None;
        }

        let mut le_score = 0.0;
        let mut be_score = 0.0;
        let mut total_chars = 0;

        // Check both endianness interpretations
        for chunk in data.chunks_exact(2) {
            let le_char = u16::from_le_bytes([chunk[0], chunk[1]]);
            let be_char = u16::from_be_bytes([chunk[0], chunk[1]]);

            total_chars += 1;

            // Score based on likelihood of being valid text characters
            if le_char < 0x80 || (le_char >= 0x20 && le_char < 0x7F) {
                le_score += 1.0;
            }

            if be_char < 0x80 || (be_char >= 0x20 && be_char < 0x7F) {
                be_score += 1.0;
            }

            // Look for null bytes in wrong positions
            if chunk[0] == 0 && chunk[1] != 0 {
                be_score += 0.5; // Likely UTF-16BE
            } else if chunk[1] == 0 && chunk[0] != 0 {
                le_score += 0.5; // Likely UTF-16LE
            }
        }

        if total_chars == 0 {
            return None;
        }

        le_score /= total_chars as f64;
        be_score /= total_chars as f64;

        if le_score > be_score && le_score > 0.6 {
            Some((Encoding::UTF16LE, le_score * 0.8))
        } else if be_score > le_score && be_score > 0.6 {
            Some((Encoding::UTF16BE, be_score * 0.8))
        } else {
            None
        }
    }

    /// Detect ASCII encoding
    fn detect_ascii(&self, data: &[u8]) -> Option<f64> {
        if data.iter().all(|&b| b < 0x80) {
            Some(0.8) // High confidence for pure ASCII
        } else {
            None
        }
    }

    /// Detect Windows code pages
    fn detect_windows_codepages(&self, data: &[u8]) -> Vec<(Encoding, f64)> {
        let mut results = Vec::new();

        // Look for characteristic Windows-1252 bytes
        let windows_1252_chars = [
            0x80, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x99,
        ]; // €, ‚, ƒ, „, …, †, ‡, ˆ, ‰, Š, ™
        let cp1252_score = self.score_characteristic_bytes(data, &windows_1252_chars);

        if cp1252_score > 0.0 {
            results.push((Encoding::WINDOWS_1252, cp1252_score * 0.6));
        }

        // Windows-1250 (Central European) detection
        let windows_1250_chars = [0x8A, 0x8C, 0x8D, 0x8F, 0x9A, 0x9C, 0x9D, 0x9F]; // Š, Ś, Ť, Ź, š, ś, ť, ź
        let cp1250_score = self.score_characteristic_bytes(data, &windows_1250_chars);

        if cp1250_score > 0.0 {
            results.push((Encoding::WINDOWS_1250, cp1250_score * 0.6));
        }

        results
    }

    /// Detect ISO encodings
    fn detect_iso_encodings(&self, data: &[u8]) -> Vec<(Encoding, f64)> {
        let mut results = Vec::new();

        // ISO-8859-1 is a superset of ASCII with Latin characters
        let has_high_latin = data.iter().any(|&b| b >= 0xA0);

        if has_high_latin {
            results.push((Encoding::ISO_8859_1, 0.5));
        }

        // ISO-8859-15 (has Euro symbol at 0xA4)
        if data.contains(&0xA4) {
            results.push((Encoding::ISO_8859_15, 0.6));
        }

        results
    }

    /// Detect EBCDIC encoding
    fn detect_ebcdic(&self, data: &[u8]) -> Option<f64> {
        // Look for characteristic EBCDIC patterns
        let ebcdic_chars = [
            0x40, 0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9,
        ]; // space, 0-9
        let ebcdic_letters = [0xC1, 0xC2, 0xC3, 0xC4, 0xC5, 0x81, 0x82, 0x83, 0x84, 0x85]; // A-E, a-e

        let mut score = 0.0;
        score += self.score_characteristic_bytes(data, &ebcdic_chars);
        score += self.score_characteristic_bytes(data, &ebcdic_letters) * 2.0; // Letters are more distinctive

        // EBCDIC rarely has bytes in 0x00-0x3F range for printable text
        let low_bytes = data.iter().filter(|&&b| b < 0x40).count();
        if low_bytes as f64 / data.len() as f64 > 0.3 {
            score *= 0.5; // Reduce confidence
        }

        if score > 0.1 {
            Some(score.min(0.8))
        } else {
            None
        }
    }

    /// Detect DOS code pages
    fn detect_dos_codepages(&self, data: &[u8]) -> Vec<(Encoding, f64)> {
        let mut results = Vec::new();

        // CP437 has distinctive box-drawing characters
        let cp437_chars = [0xB0, 0xB1, 0xB2, 0xDB, 0xC9, 0xBB, 0xC8, 0xBC]; // Various box chars
        let cp437_score = self.score_characteristic_bytes(data, &cp437_chars);

        if cp437_score > 0.0 {
            results.push((Encoding::CP_437, cp437_score * 0.7));
        }

        results
    }

    /// Score presence of characteristic bytes for an encoding
    fn score_characteristic_bytes(&self, data: &[u8], chars: &[u8]) -> f64 {
        let mut found = 0;
        for &ch in chars {
            if data.contains(&ch) {
                found += 1;
            }
        }

        if found > 0 {
            found as f64 / chars.len() as f64
        } else {
            0.0
        }
    }

    /// Detect encoding with language hint
    pub fn detect_with_hint(&self, data: &[u8], language_hint: &str) -> DetectionResult {
        let mut result = self.detect(data);

        // Adjust confidence based on language hint
        match language_hint.to_lowercase().as_str() {
            "english" | "en" => {
                // Boost confidence for ASCII-compatible encodings
                if result.encoding.is_ascii_compatible() {
                    result.confidence = (result.confidence * 1.2).min(1.0);
                }
            }
            "german" | "de" | "french" | "fr" | "spanish" | "es" => {
                // Boost confidence for Latin encodings
                if matches!(
                    result.encoding,
                    Encoding::ISO_8859_1 | Encoding::ISO_8859_15 | Encoding::WINDOWS_1252
                ) {
                    result.confidence = (result.confidence * 1.3).min(1.0);
                }
            }
            "polish" | "pl" | "czech" | "cz" | "hungarian" | "hu" => {
                // Boost confidence for Central European encodings
                if matches!(
                    result.encoding,
                    Encoding::WINDOWS_1250 | Encoding::ISO_8859_2
                ) {
                    result.confidence = (result.confidence * 1.3).min(1.0);
                }
            }
            "russian" | "ru" | "cyrillic" => {
                // Would boost Cyrillic encodings if implemented
                if matches!(
                    result.encoding,
                    Encoding::WINDOWS_1251 | Encoding::ISO_8859_5
                ) {
                    result.confidence = (result.confidence * 1.3).min(1.0);
                }
            }
            _ => {} // No adjustment for unknown languages
        }

        result
    }
}
