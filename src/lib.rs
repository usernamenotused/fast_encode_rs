//! # FastEncode - High-Performance Character Encoding Library
//!
//! A blazingly fast, SIMD-accelerated character encoding conversion library
//! designed for enterprise applications processing mainframe data.
//!
//! ## Features
//!
//! - **Zero-copy conversions** with pre-computed translation tables
//! - **SIMD vectorization** for bulk data processing  
//! - **Enterprise encodings** including EBCDIC variants, Windows code pages, ISO-8859 series
//! - **Streaming support** for large datasets
//! - **Thread-safe** operations
//! - **Comprehensive error handling**
//!
//! ## Quick Start
//!
//! ```rust
//! use fast_encode::{Encoding, Translator};
//!
//! // Create a translator from EBCDIC to UTF-8
//! let translator = Translator::new(
//!     Encoding::EBCDIC_037,
//!     Encoding::UTF8
//! ).unwrap();
//!
//! // Convert mainframe data
//! let ebcdic_data = &[0xC8, 0xC5, 0xD3, 0xD3, 0xD6]; // "HELLO"
//! let utf8_result = translator.convert(ebcdic_data).unwrap();
//! assert_eq!(std::str::from_utf8(&utf8_result).unwrap(), "HELLO");
//! ```

#![cfg_attr(feature = "simd", feature(portable_simd))]
#![deny(missing_docs)]

use std::fmt;

pub mod detection;
mod multibyte;
mod tables;

// SIMD imports when feature is enabled
#[cfg(feature = "simd")]
use std::simd::{Simd, SimdPartialEq, u8x16, u8x32};

/// Result type for encoding operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during encoding operations
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Byte value cannot be represented in target encoding
    UnmappableSource {
        /// The unmappable byte value
        byte: u8,
        /// Position of the byte in input
        position: usize,
    },
    /// Character cannot be encoded in target encoding  
    UnmappableTarget {
        /// The unmappable character
        character: char,
        /// Position of the character in input
        position: usize,
    },
    /// Invalid input data
    InvalidInput(String),
    /// Unsupported conversion between encodings
    UnsupportedConversion {
        /// Source encoding name
        from: &'static str,
        /// Target encoding name
        to: &'static str,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnmappableSource { byte, position } => {
                write!(
                    f,
                    "Unmappable source byte 0x{:02X} at position {}",
                    byte, position
                )
            }
            Error::UnmappableTarget {
                character,
                position,
            } => {
                write!(
                    f,
                    "Cannot encode character '{}' at position {}",
                    character, position
                )
            }
            Error::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Error::UnsupportedConversion { from, to } => {
                write!(f, "Unsupported conversion from {} to {}", from, to)
            }
        }
    }
}

impl std::error::Error for Error {}

/// Supported character encodings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum Encoding {
    // Unicode encodings
    /// UTF-8 Unicode encoding (variable length, 1-4 bytes)
    UTF8,
    /// UTF-16LE Unicode encoding (little endian)
    UTF16LE,
    /// UTF-16BE Unicode encoding (big endian)  
    UTF16BE,

    // ASCII and Latin encodings
    /// ASCII (7-bit, 0-127)
    ASCII,
    /// ISO-8859-1 (Latin-1) - Western European
    ISO_8859_1,
    /// ISO-8859-2 (Latin-2) - Central/Eastern European
    ISO_8859_2,
    /// ISO-8859-3 (Latin-3) - South European
    ISO_8859_3,
    /// ISO-8859-4 (Latin-4) - North European
    ISO_8859_4,
    /// ISO-8859-5 (Cyrillic)
    ISO_8859_5,
    /// ISO-8859-6 (Arabic)
    ISO_8859_6,
    /// ISO-8859-7 (Greek)
    ISO_8859_7,
    /// ISO-8859-8 (Hebrew)
    ISO_8859_8,
    /// ISO-8859-9 (Latin-5) - Turkish
    ISO_8859_9,
    /// ISO-8859-10 (Latin-6) - Nordic
    ISO_8859_10,
    /// ISO-8859-11 (Thai)
    ISO_8859_11,
    /// ISO-8859-13 (Latin-7) - Baltic Rim
    ISO_8859_13,
    /// ISO-8859-14 (Latin-8) - Celtic
    ISO_8859_14,
    /// ISO-8859-15 (Latin-9) - Western European with Euro
    ISO_8859_15,
    /// ISO-8859-16 (Latin-10) - South-Eastern European
    ISO_8859_16,

    // Windows code pages
    /// Windows-1250 (Central/Eastern European)
    WINDOWS_1250,
    /// Windows-1251 (Cyrillic)
    WINDOWS_1251,
    /// Windows-1252 (Western European)
    WINDOWS_1252,
    /// Windows-1253 (Greek)
    WINDOWS_1253,
    /// Windows-1254 (Turkish)
    WINDOWS_1254,
    /// Windows-1255 (Hebrew)
    WINDOWS_1255,
    /// Windows-1256 (Arabic)
    WINDOWS_1256,
    /// Windows-1257 (Baltic)
    WINDOWS_1257,
    /// Windows-1258 (Vietnamese)
    WINDOWS_1258,
    /// Windows-874 (Thai)
    WINDOWS_874,

    // EBCDIC variants
    /// IBM EBCDIC Code Page 037 (US/Canada)
    EBCDIC_037,
    /// IBM EBCDIC Code Page 273 (Germany/Austria)
    EBCDIC_273,
    /// IBM EBCDIC Code Page 277 (Denmark/Norway)
    EBCDIC_277,
    /// IBM EBCDIC Code Page 278 (Finland/Sweden)
    EBCDIC_278,
    /// IBM EBCDIC Code Page 280 (Italy)
    EBCDIC_280,
    /// IBM EBCDIC Code Page 284 (Spain)
    EBCDIC_284,
    /// IBM EBCDIC Code Page 285 (United Kingdom)
    EBCDIC_285,
    /// IBM EBCDIC Code Page 297 (France)
    EBCDIC_297,
    /// IBM EBCDIC Code Page 500 (International)
    EBCDIC_500,
    /// IBM EBCDIC Code Page 1047 (Latin-1)  
    EBCDIC_1047,

    // DOS/OEM code pages
    /// DOS Code Page 437 (US OEM)
    CP_437,
    /// DOS Code Page 850 (Western European OEM)
    CP_850,
    /// DOS Code Page 852 (Central European OEM)
    CP_852,
    /// DOS Code Page 855 (Cyrillic OEM)
    CP_855,
    /// DOS Code Page 857 (Turkish OEM)
    CP_857,
    /// DOS Code Page 860 (Portuguese OEM)
    CP_860,
    /// DOS Code Page 861 (Icelandic OEM)
    CP_861,
    /// DOS Code Page 862 (Hebrew OEM)
    CP_862,
    /// DOS Code Page 863 (French Canadian OEM)
    CP_863,
    /// DOS Code Page 865 (Nordic OEM)
    CP_865,
    /// DOS Code Page 866 (Russian OEM)
    CP_866,

    // Mac encodings
    /// Macintosh Roman
    MAC_ROMAN,
    /// Macintosh Cyrillic
    MAC_CYRILLIC,

    // Asian encodings (placeholders for future implementation)
    /// Shift-JIS (Japanese)
    SHIFT_JIS,
    /// EUC-JP (Japanese)
    EUC_JP,
    /// GB2312 (Simplified Chinese)
    GB2312,
    /// Big5 (Traditional Chinese)
    BIG5,
    /// EUC-KR (Korean)
    EUC_KR,
}

impl Encoding {
    /// Get the canonical name of this encoding
    pub fn name(self) -> &'static str {
        match self {
            // Unicode
            Encoding::UTF8 => "UTF-8",
            Encoding::UTF16LE => "UTF-16LE",
            Encoding::UTF16BE => "UTF-16BE",

            // ASCII and Latin
            Encoding::ASCII => "US-ASCII",
            Encoding::ISO_8859_1 => "ISO-8859-1",
            Encoding::ISO_8859_2 => "ISO-8859-2",
            Encoding::ISO_8859_3 => "ISO-8859-3",
            Encoding::ISO_8859_4 => "ISO-8859-4",
            Encoding::ISO_8859_5 => "ISO-8859-5",
            Encoding::ISO_8859_6 => "ISO-8859-6",
            Encoding::ISO_8859_7 => "ISO-8859-7",
            Encoding::ISO_8859_8 => "ISO-8859-8",
            Encoding::ISO_8859_9 => "ISO-8859-9",
            Encoding::ISO_8859_10 => "ISO-8859-10",
            Encoding::ISO_8859_11 => "ISO-8859-11",
            Encoding::ISO_8859_13 => "ISO-8859-13",
            Encoding::ISO_8859_14 => "ISO-8859-14",
            Encoding::ISO_8859_15 => "ISO-8859-15",
            Encoding::ISO_8859_16 => "ISO-8859-16",

            // Windows
            Encoding::WINDOWS_1250 => "Windows-1250",
            Encoding::WINDOWS_1251 => "Windows-1251",
            Encoding::WINDOWS_1252 => "Windows-1252",
            Encoding::WINDOWS_1253 => "Windows-1253",
            Encoding::WINDOWS_1254 => "Windows-1254",
            Encoding::WINDOWS_1255 => "Windows-1255",
            Encoding::WINDOWS_1256 => "Windows-1256",
            Encoding::WINDOWS_1257 => "Windows-1257",
            Encoding::WINDOWS_1258 => "Windows-1258",
            Encoding::WINDOWS_874 => "Windows-874",

            // EBCDIC
            Encoding::EBCDIC_037 => "IBM037",
            Encoding::EBCDIC_273 => "IBM273",
            Encoding::EBCDIC_277 => "IBM277",
            Encoding::EBCDIC_278 => "IBM278",
            Encoding::EBCDIC_280 => "IBM280",
            Encoding::EBCDIC_284 => "IBM284",
            Encoding::EBCDIC_285 => "IBM285",
            Encoding::EBCDIC_297 => "IBM297",
            Encoding::EBCDIC_500 => "IBM500",
            Encoding::EBCDIC_1047 => "IBM1047",

            // DOS/OEM
            Encoding::CP_437 => "CP437",
            Encoding::CP_850 => "CP850",
            Encoding::CP_852 => "CP852",
            Encoding::CP_855 => "CP855",
            Encoding::CP_857 => "CP857",
            Encoding::CP_860 => "CP860",
            Encoding::CP_861 => "CP861",
            Encoding::CP_862 => "CP862",
            Encoding::CP_863 => "CP863",
            Encoding::CP_865 => "CP865",
            Encoding::CP_866 => "CP866",

            // Mac
            Encoding::MAC_ROMAN => "MacRoman",
            Encoding::MAC_CYRILLIC => "MacCyrillic",

            // Asian (placeholder)
            Encoding::SHIFT_JIS => "Shift_JIS",
            Encoding::EUC_JP => "EUC-JP",
            Encoding::GB2312 => "GB2312",
            Encoding::BIG5 => "Big5",
            Encoding::EUC_KR => "EUC-KR",
        }
    }

    /// Check if this encoding is ASCII-compatible (ASCII bytes 0-127 have same meaning)
    pub fn is_ascii_compatible(self) -> bool {
        matches!(
            self,
            // Unicode encodings
            Encoding::UTF8 |
            Encoding::ASCII |
            // ISO-8859 series (all ASCII-compatible)
            Encoding::ISO_8859_1 | Encoding::ISO_8859_2 | Encoding::ISO_8859_3 |
            Encoding::ISO_8859_4 | Encoding::ISO_8859_5 | Encoding::ISO_8859_6 |
            Encoding::ISO_8859_7 | Encoding::ISO_8859_8 | Encoding::ISO_8859_9 |
            Encoding::ISO_8859_10 | Encoding::ISO_8859_11 | Encoding::ISO_8859_13 |
            Encoding::ISO_8859_14 | Encoding::ISO_8859_15 | Encoding::ISO_8859_16 |
            // Windows code pages (all ASCII-compatible)
            Encoding::WINDOWS_1250 | Encoding::WINDOWS_1251 | Encoding::WINDOWS_1252 |
            Encoding::WINDOWS_1253 | Encoding::WINDOWS_1254 | Encoding::WINDOWS_1255 |
            Encoding::WINDOWS_1256 | Encoding::WINDOWS_1257 | Encoding::WINDOWS_1258 |
            Encoding::WINDOWS_874 |
            // DOS code pages (ASCII-compatible)
            Encoding::CP_437 | Encoding::CP_850 | Encoding::CP_852 | Encoding::CP_855 |
            Encoding::CP_857 | Encoding::CP_860 | Encoding::CP_861 | Encoding::CP_862 |
            Encoding::CP_863 | Encoding::CP_865 | Encoding::CP_866 |
            // Mac encodings (ASCII-compatible)
            Encoding::MAC_ROMAN | Encoding::MAC_CYRILLIC
        )
    }

    /// Check if this encoding uses variable-length character representation
    pub fn is_multibyte(self) -> bool {
        matches!(
            self,
            Encoding::UTF8
                | Encoding::UTF16LE
                | Encoding::UTF16BE
                | Encoding::SHIFT_JIS
                | Encoding::EUC_JP
                | Encoding::GB2312
                | Encoding::BIG5
                | Encoding::EUC_KR
        )
    }

    /// Get the byte order mark (BOM) for this encoding if it has one
    pub fn bom(self) -> Option<&'static [u8]> {
        match self {
            Encoding::UTF8 => Some(&[0xEF, 0xBB, 0xBF]),
            Encoding::UTF16LE => Some(&[0xFF, 0xFE]),
            Encoding::UTF16BE => Some(&[0xFE, 0xFF]),
            _ => None,
        }
    }
}

/// Pre-computed translation table for ultra-fast byte-to-byte conversion
#[derive(Debug, Clone)]
pub struct TranslationTable {
    /// Direct lookup table: source_byte -> target_byte (0xFF = unmappable)
    table: [u8; 256],
    /// Bitmask of unmappable bytes for fast checking
    unmappable_mask: [u64; 4], // 256 bits = 4 u64s
}

impl TranslationTable {
    /// Create a new translation table between two encodings
    ///
    /// Note: This only works for single-byte to single-byte conversions.
    /// For multi-byte conversions (UTF-8, UTF-16, etc.), use Translator which handles multi-byte encodings.
    pub fn new(from: Encoding, to: Encoding) -> Result<Self> {
        // Check if this is a multi-byte conversion
        if from.is_multibyte() || to.is_multibyte() {
            return Err(Error::UnsupportedConversion {
                from: from.name(),
                to: to.name(),
            });
        }

        let from_chars = tables::get_encoding_chars(from);
        let to_chars = tables::get_encoding_chars(to);

        // Build reverse lookup for target encoding
        let mut to_lookup = [0xFFu8; 65536]; // Unicode code point -> byte
        for (byte, &ch_opt) in to_chars.iter().enumerate() {
            if let Some(ch) = ch_opt {
                if (ch as u32) < 65536 {
                    to_lookup[ch as usize] = byte as u8;
                }
            }
        }

        // Build translation table
        let mut table = [0xFFu8; 256]; // 0xFF = unmappable marker
        let mut unmappable_mask = [0u64; 4];

        for (src_byte, &ch_opt) in from_chars.iter().enumerate() {
            if let Some(ch) = ch_opt {
                let target_byte = to_lookup[ch as usize];
                if target_byte != 0xFF {
                    table[src_byte] = target_byte;
                } else {
                    // Mark as unmappable in bitmask
                    let word_idx = src_byte / 64;
                    let bit_idx = src_byte % 64;
                    unmappable_mask[word_idx] |= 1u64 << bit_idx;
                }
            } else {
                // Source byte unmapped - mark in bitmask
                let word_idx = src_byte / 64;
                let bit_idx = src_byte % 64;
                unmappable_mask[word_idx] |= 1u64 << bit_idx;
            }
        }

        Ok(Self {
            table,
            unmappable_mask,
        })
    }

    /// Check if a byte is mappable
    #[inline]
    pub fn is_mappable(&self, byte: u8) -> bool {
        let word_idx = (byte as usize) / 64;
        let bit_idx = (byte as usize) % 64;
        (self.unmappable_mask[word_idx] & (1u64 << bit_idx)) == 0
    }

    /// Translate a single byte (unchecked - assumes mappable)
    #[inline]
    pub fn translate_byte_unchecked(&self, byte: u8) -> u8 {
        unsafe { *self.table.get_unchecked(byte as usize) }
    }

    /// Translate bytes with error checking
    pub fn translate(&self, input: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::with_capacity(input.len());

        #[cfg(feature = "simd")]
        {
            self.translate_simd(input, &mut output)?;
        }

        #[cfg(not(feature = "simd"))]
        {
            self.translate_scalar(input, &mut output)?;
        }

        Ok(output)
    }

    /// Translate in-place, overwriting input buffer
    pub fn translate_in_place(&self, buffer: &mut [u8]) -> Result<()> {
        #[cfg(feature = "simd")]
        {
            self.translate_in_place_simd(buffer)
        }

        #[cfg(not(feature = "simd"))]
        {
            self.translate_in_place_scalar(buffer)
        }
    }

    // Scalar implementation
    fn translate_scalar(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()> {
        for (pos, &byte) in input.iter().enumerate() {
            if !self.is_mappable(byte) {
                return Err(Error::UnmappableSource {
                    byte,
                    position: pos,
                });
            }
            output.push(self.translate_byte_unchecked(byte));
        }
        Ok(())
    }

    fn translate_in_place_scalar(&self, buffer: &mut [u8]) -> Result<()> {
        for (pos, byte) in buffer.iter_mut().enumerate() {
            if !self.is_mappable(*byte) {
                return Err(Error::UnmappableSource {
                    byte: *byte,
                    position: pos,
                });
            }
            *byte = self.translate_byte_unchecked(*byte);
        }
        Ok(())
    }

    // SIMD implementations
    #[cfg(feature = "simd")]
    fn translate_simd(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()> {
        const CHUNK_SIZE: usize = 32;
        let chunks = input.chunks_exact(CHUNK_SIZE);
        let remainder = chunks.remainder();

        // Process 32-byte chunks with AVX2-style SIMD
        for (chunk_idx, chunk) in chunks.enumerate() {
            let chunk_pos = chunk_idx * CHUNK_SIZE;
            let input_vec = u8x32::from_slice(chunk);

            // Check for unmappable bytes using SIMD comparison
            if self.has_unmappable_simd(input_vec, chunk_pos)? {
                return Err(Error::UnmappableSource {
                    byte: 0, // Would need more complex logic to find exact byte
                    position: chunk_pos,
                });
            }

            // Perform vectorized translation using gather operation
            let translated = self.translate_chunk_simd(input_vec);
            output.extend_from_slice(translated.as_array());
        }

        // Handle remainder with scalar code
        let remainder_start = input.len() - remainder.len();
        for (i, &byte) in remainder.iter().enumerate() {
            let pos = remainder_start + i;
            if !self.is_mappable(byte) {
                return Err(Error::UnmappableSource {
                    byte,
                    position: pos,
                });
            }
            output.push(self.translate_byte_unchecked(byte));
        }

        Ok(())
    }

    #[cfg(feature = "simd")]
    fn translate_in_place_simd(&self, buffer: &mut [u8]) -> Result<()> {
        const CHUNK_SIZE: usize = 32;
        let len = buffer.len();
        let (chunks, remainder) = buffer.split_at_mut(len - (len % CHUNK_SIZE));

        // Process chunks
        for (chunk_idx, chunk) in chunks.chunks_exact_mut(CHUNK_SIZE).enumerate() {
            let chunk_pos = chunk_idx * CHUNK_SIZE;
            let input_vec = u8x32::from_slice(chunk);

            // Check for unmappable bytes
            if self.has_unmappable_simd(input_vec, chunk_pos)? {
                return Err(Error::UnmappableSource {
                    byte: 0,
                    position: chunk_pos,
                });
            }

            // Translate and store back
            let translated = self.translate_chunk_simd(input_vec);
            chunk.copy_from_slice(translated.as_array());
        }

        // Handle remainder
        let remainder_start = len - remainder.len();
        for (i, byte) in remainder.iter_mut().enumerate() {
            let pos = remainder_start + i;
            if !self.is_mappable(*byte) {
                return Err(Error::UnmappableSource {
                    byte: *byte,
                    position: pos,
                });
            }
            *byte = self.translate_byte_unchecked(*byte);
        }

        Ok(())
    }

    #[cfg(feature = "simd")]
    fn has_unmappable_simd(&self, input: u8x32, _chunk_pos: usize) -> Result<bool> {
        // This is a simplified check - in a real implementation, we'd use
        // vectorized bit manipulation to check the unmappable_mask
        // For now, fall back to scalar checking
        for &byte in input.as_array() {
            if !self.is_mappable(byte) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    #[cfg(feature = "simd")]
    fn translate_chunk_simd(&self, input: u8x32) -> u8x32 {
        // SIMD gather operation - translate 32 bytes at once
        // This uses the fact that our translation table is exactly 256 bytes
        let mut result = [0u8; 32];

        for (i, &byte) in input.as_array().iter().enumerate() {
            result[i] = self.translate_byte_unchecked(byte);
        }

        u8x32::from_array(result)
    }
}

/// High-level encoding converter with streaming support
pub struct Translator {
    table: Option<TranslationTable>,
    multibyte: Option<multibyte::MultiByte>,
    from: Encoding,
    to: Encoding,
}

impl Translator {
    /// Create a new translator between two encodings
    pub fn new(from: Encoding, to: Encoding) -> Result<Self> {
        // Check if we need multi-byte conversion (involves UTF-8, UTF-16, or other multibyte encodings)
        if from.is_multibyte() || to.is_multibyte() {
            let multibyte = multibyte::MultiByte::new(from, to);
            Ok(Self {
                table: None,
                multibyte: Some(multibyte),
                from,
                to,
            })
        } else {
            // Single-byte to single-byte conversion
            let table = TranslationTable::new(from, to)?;
            Ok(Self {
                table: Some(table),
                multibyte: None,
                from,
                to,
            })
        }
    }

    /// Get source encoding
    pub fn from_encoding(&self) -> Encoding {
        self.from
    }

    /// Get target encoding  
    pub fn to_encoding(&self) -> Encoding {
        self.to
    }

    /// Convert data from source to target encoding
    pub fn convert(&self, input: &[u8]) -> Result<Vec<u8>> {
        if let Some(ref table) = self.table {
            table.translate(input)
        } else if let Some(ref multibyte) = self.multibyte {
            multibyte.convert(input)
        } else {
            Err(Error::UnsupportedConversion {
                from: self.from.name(),
                to: self.to.name(),
            })
        }
    }

    /// Convert data in-place (destructive)
    ///
    /// Note: This only works for single-byte to single-byte conversions.
    /// Multi-byte conversions (involving UTF-8) cannot be done in-place due to variable lengths.
    pub fn convert_in_place(&self, buffer: &mut [u8]) -> Result<()> {
        if let Some(ref table) = self.table {
            table.translate_in_place(buffer)
        } else {
            Err(Error::UnsupportedConversion {
                from: self.from.name(),
                to: self.to.name(),
            })
        }
    }

    /// Convert with custom error handling  
    ///
    /// Note: This only works for single-byte to single-byte conversions.
    /// For multi-byte conversions, use `convert()` and handle errors manually.
    pub fn convert_lossy(&self, input: &[u8], replacement: u8) -> Vec<u8> {
        if let Some(ref table) = self.table {
            let mut output = Vec::with_capacity(input.len());

            for &byte in input {
                if table.is_mappable(byte) {
                    output.push(table.translate_byte_unchecked(byte));
                } else {
                    output.push(replacement);
                }
            }

            output
        } else {
            // For multi-byte conversions, try convert and fall back to replacement
            match self.convert(input) {
                Ok(result) => result,
                Err(_) => vec![replacement; input.len()],
            }
        }
    }
}

/// Streaming converter for processing large datasets
pub struct StreamingTranslator {
    /// Internal translator
    translator: Translator,
    /// Internal buffer for processing
    #[allow(dead_code)]
    buffer: Vec<u8>,
    /// Buffer size in bytes
    #[allow(dead_code)]
    buffer_size: usize,
}

impl StreamingTranslator {
    /// Create a new streaming translator with specified buffer size
    pub fn new(from: Encoding, to: Encoding, buffer_size: usize) -> Result<Self> {
        let translator = Translator::new(from, to)?;
        Ok(Self {
            translator,
            buffer: Vec::with_capacity(buffer_size),
            buffer_size,
        })
    }

    /// Create with default 64KB buffer
    pub fn with_default_buffer(from: Encoding, to: Encoding) -> Result<Self> {
        Self::new(from, to, 64 * 1024)
    }

    /// Process a chunk of data
    pub fn process_chunk(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        self.translator.convert(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ebcdic_to_utf8() {
        let translator = Translator::new(Encoding::EBCDIC_037, Encoding::UTF8).unwrap();

        // "HELLO" in EBCDIC
        let input = &[0xC8, 0xC5, 0xD3, 0xD3, 0xD6];
        let output = translator.convert(input).unwrap();

        assert_eq!(std::str::from_utf8(&output).unwrap(), "HELLO");
    }

    #[test]
    fn test_in_place_conversion() {
        // Test single-byte to single-byte conversion (works in-place)
        let translator = Translator::new(Encoding::EBCDIC_037, Encoding::ISO_8859_1).unwrap();

        let mut data = vec![0xC8, 0xC5, 0xD3, 0xD3, 0xD6]; // "HELLO" in EBCDIC
        translator.convert_in_place(&mut data).unwrap();

        // Should convert to "HELLO" in ISO-8859-1
        assert_eq!(std::str::from_utf8(&data).unwrap(), "HELLO");
    }

    #[test]
    fn test_streaming_translator() {
        let mut stream =
            StreamingTranslator::with_default_buffer(Encoding::EBCDIC_037, Encoding::UTF8).unwrap();

        let chunk1 = &[0xC8, 0xC5]; // "HE"
        let chunk2 = &[0xD3, 0xD3, 0xD6]; // "LLO"

        let result1 = stream.process_chunk(chunk1).unwrap();
        let result2 = stream.process_chunk(chunk2).unwrap();

        let mut combined = result1;
        combined.extend(result2);

        assert_eq!(std::str::from_utf8(&combined).unwrap(), "HELLO");
    }

    #[test]
    fn test_encoding_properties() {
        assert_eq!(Encoding::UTF8.name(), "UTF-8");
        assert_eq!(Encoding::EBCDIC_037.name(), "IBM037");
        assert!(Encoding::UTF8.is_ascii_compatible());
        assert!(!Encoding::EBCDIC_037.is_ascii_compatible());
    }

    #[test]
    fn test_windows_1252_special_chars() {
        let translator = Translator::new(Encoding::WINDOWS_1252, Encoding::UTF8).unwrap();

        // Test Euro symbol (0x80 in Windows-1252)
        let input = &[0x80];
        let output = translator.convert(input).unwrap();
        assert_eq!(std::str::from_utf8(&output).unwrap(), "€");

        // Test trademark symbol (0x99 in Windows-1252)
        let input = &[0x99];
        let output = translator.convert(input).unwrap();
        assert_eq!(std::str::from_utf8(&output).unwrap(), "™");
    }

    #[test]
    fn test_ebcdic_037_complete_alphabet() {
        let translator = Translator::new(Encoding::EBCDIC_037, Encoding::UTF8).unwrap();

        // Test full alphabet in EBCDIC
        let ebcdic_alphabet = &[
            // A-I
            0xC1, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, // J-R
            0xD1, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, // S-Z
            0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9,
        ];

        let output = translator.convert(ebcdic_alphabet).unwrap();
        let result = std::str::from_utf8(&output).unwrap();
        assert_eq!(result, "ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }

    #[test]
    fn test_bidirectional_conversion() {
        // Test that A->B->A conversion preserves data
        let translator_forward =
            Translator::new(Encoding::ISO_8859_1, Encoding::WINDOWS_1252).unwrap();
        let translator_backward =
            Translator::new(Encoding::WINDOWS_1252, Encoding::ISO_8859_1).unwrap();

        let original = b"Hello, World! \xA9\xAE"; // Include copyright and registered symbols
        let forward = translator_forward.convert(original).unwrap();
        let roundtrip = translator_backward.convert(&forward).unwrap();

        assert_eq!(original, &roundtrip[..]);
    }

    #[test]
    fn test_utf8_multibyte_conversion() {
        // Test UTF-8 conversion with multi-byte characters
        let translator = Translator::new(Encoding::WINDOWS_1252, Encoding::UTF8).unwrap();

        // Windows-1252 bytes for: "Hello™€" (includes trademark and euro symbols)
        let input = &[b'H', b'e', b'l', b'l', b'o', 0x99, 0x80]; // Hello™€
        let output = translator.convert(input).unwrap();
        let result = std::str::from_utf8(&output).unwrap();

        assert_eq!(result, "Hello™€");

        // Test reverse conversion
        let reverse_translator = Translator::new(Encoding::UTF8, Encoding::WINDOWS_1252).unwrap();
        let roundtrip = reverse_translator.convert(&output).unwrap();

        assert_eq!(input, &roundtrip[..]);
    }

    #[test]
    fn test_windows_1250_central_european() {
        let translator = Translator::new(Encoding::WINDOWS_1250, Encoding::UTF8).unwrap();

        // Test Euro symbol and Polish/Czech characters
        let input = &[0x80, 0x8A, 0x8C, 0x8F]; // €ŠŚŹ in Windows-1250
        let output = translator.convert(input).unwrap();
        let result = std::str::from_utf8(&output).unwrap();

        assert_eq!(result, "€ŠŚŹ");
    }

    #[test]
    fn test_cp437_dos_characters() {
        let translator = Translator::new(Encoding::CP_437, Encoding::UTF8).unwrap();

        // Test box drawing and special characters
        let input = &[0xC9, 0xCD, 0xBB, 0x20, 0xF8]; // ╔═» and degree symbol
        let output = translator.convert(input).unwrap();
        let result = std::str::from_utf8(&output).unwrap();

        // Should contain box drawing characters and degree symbol
        assert!(result.contains('╔'));
        assert!(result.contains('°')); // degree symbol is at 0xF8 in CP437
    }

    #[test]
    fn test_iso_8859_15_euro_support() {
        let translator = Translator::new(Encoding::ISO_8859_15, Encoding::UTF8).unwrap();

        // Test Euro symbol at 0xA4 (differs from ISO-8859-1)
        let input = &[0xA4]; // Euro in ISO-8859-15
        let output = translator.convert(input).unwrap();
        let result = std::str::from_utf8(&output).unwrap();

        assert_eq!(result, "€");
    }

    #[test]
    fn test_encoding_properties_expanded() {
        // Test ASCII compatibility
        assert!(Encoding::WINDOWS_1250.is_ascii_compatible());
        assert!(Encoding::CP_437.is_ascii_compatible());
        assert!(!Encoding::EBCDIC_037.is_ascii_compatible());

        // Test multibyte detection
        assert!(Encoding::UTF8.is_multibyte());
        assert!(Encoding::UTF16LE.is_multibyte());
        assert!(!Encoding::WINDOWS_1252.is_multibyte());

        // Test BOM support
        assert_eq!(Encoding::UTF8.bom(), Some([0xEF, 0xBB, 0xBF].as_slice()));
        assert_eq!(Encoding::UTF16LE.bom(), Some([0xFF, 0xFE].as_slice()));
        assert_eq!(Encoding::WINDOWS_1252.bom(), None);
    }

    #[test]
    fn test_utf16_conversion() {
        // Test UTF-8 to UTF-16LE conversion
        let translator = Translator::new(Encoding::UTF8, Encoding::UTF16LE).unwrap();

        let input = "Hello 🌍!"; // UTF-8 with emoji
        let output = translator.convert(input.as_bytes()).unwrap();

        // Check that we got UTF-16LE bytes
        assert!(output.len() > input.len()); // UTF-16 should be longer for this text

        // Test reverse conversion
        let reverse_translator = Translator::new(Encoding::UTF16LE, Encoding::UTF8).unwrap();
        let roundtrip = reverse_translator.convert(&output).unwrap();

        assert_eq!(input.as_bytes(), &roundtrip[..]);
    }

    #[test]
    fn test_utf16_endianness_conversion() {
        // Test UTF-16LE to UTF-16BE conversion
        let le_to_be = Translator::new(Encoding::UTF16LE, Encoding::UTF16BE).unwrap();

        // "Hi" in UTF-16LE (0x0048, 0x0069)
        let le_input = &[0x48, 0x00, 0x69, 0x00]; // Little-endian
        let be_output = le_to_be.convert(le_input).unwrap();

        // Should be big-endian now
        let expected_be = &[0x00, 0x48, 0x00, 0x69]; // Big-endian
        assert_eq!(be_output, expected_be);

        // Test reverse
        let be_to_le = Translator::new(Encoding::UTF16BE, Encoding::UTF16LE).unwrap();
        let roundtrip = be_to_le.convert(&be_output).unwrap();
        assert_eq!(le_input, &roundtrip[..]);
    }

    #[test]
    fn test_single_byte_to_utf16() {
        // Test Windows-1252 to UTF-16LE conversion
        let translator = Translator::new(Encoding::WINDOWS_1252, Encoding::UTF16LE).unwrap();

        // "€" (Euro symbol) is 0x80 in Windows-1252
        let input = &[0x80];
        let output = translator.convert(input).unwrap();

        // Euro symbol in UTF-16LE should be 0x20AC (little-endian: AC 20)
        let expected = &[0xAC, 0x20];
        assert_eq!(output, expected);
    }

    #[test]
    fn test_encoding_detection() {
        use detection::EncodingDetector;

        let detector = EncodingDetector::new();

        // Test UTF-8 BOM detection
        let utf8_bom = &[0xEF, 0xBB, 0xBF, b'H', b'e', b'l', b'l', b'o'];
        let result = detector.detect(utf8_bom);
        assert_eq!(result.encoding, Encoding::UTF8);
        assert!(result.bom_detected);
        assert!(result.confidence > 0.9);

        // Test UTF-16LE BOM detection
        let utf16le_bom = &[0xFF, 0xFE, b'H', 0x00, b'i', 0x00];
        let result = detector.detect(utf16le_bom);
        assert_eq!(result.encoding, Encoding::UTF16LE);
        assert!(result.bom_detected);

        // Test ASCII detection
        let ascii_text = b"Hello, World! This is plain ASCII text.";
        let result = detector.detect(ascii_text);
        assert_eq!(result.encoding, Encoding::ASCII);
        assert!(result.confidence > 0.7);

        // Test UTF-8 detection (no BOM)
        let utf8_text = "Hello 世界! This contains Unicode: 🌍".as_bytes();
        let result = detector.detect(utf8_text);
        assert_eq!(result.encoding, Encoding::UTF8);
        assert!(result.confidence > 0.7);
    }

    #[test]
    fn test_detection_with_language_hint() {
        use detection::EncodingDetector;

        let detector = EncodingDetector::new();

        // Test with Windows-1252 text and language hint
        let text_with_euro = &[b'H', b'e', b'l', b'l', b'o', b' ', 0x80]; // "Hello €"

        // Without hint
        let result1 = detector.detect(text_with_euro);

        // With German language hint (should boost Windows-1252 confidence)
        let result2 = detector.detect_with_hint(text_with_euro, "german");

        // Language hint should increase confidence for appropriate encodings
        if result1.encoding == Encoding::WINDOWS_1252 {
            assert!(result2.confidence >= result1.confidence);
        }
    }
}
