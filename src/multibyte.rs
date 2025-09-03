//! Multi-byte encoding support for UTF-8, UTF-16, and other variable-length encodings
//!
//! This module handles conversions to/from encodings where characters can span multiple bytes.

use crate::{Encoding, Error, Result};

/// Multi-byte translator for handling UTF-8 and other variable-length encodings
pub struct MultiByte {
    from: Encoding,
    to: Encoding,
}

impl MultiByte {
    /// Create a new multi-byte translator
    pub fn new(from: Encoding, to: Encoding) -> Self {
        Self { from, to }
    }

    /// Convert single-byte encoding to UTF-8
    pub fn to_utf8(&self, input: &[u8]) -> Result<Vec<u8>> {
        if !matches!(self.to, Encoding::UTF8) {
            return Err(Error::UnsupportedConversion {
                from: self.from.name(),
                to: self.to.name(),
            });
        }

        let from_chars = crate::tables::get_encoding_chars(self.from);
        let mut output = Vec::new();

        for (pos, &byte) in input.iter().enumerate() {
            if let Some(ch) = from_chars[byte as usize] {
                // Convert Unicode character to UTF-8 bytes
                let mut buf = [0u8; 4];
                let utf8_bytes = ch.encode_utf8(&mut buf).as_bytes();
                output.extend_from_slice(utf8_bytes);
            } else {
                return Err(Error::UnmappableSource {
                    byte,
                    position: pos,
                });
            }
        }

        Ok(output)
    }

    /// Convert UTF-8 to single-byte encoding  
    pub fn from_utf8(&self, input: &[u8]) -> Result<Vec<u8>> {
        if !matches!(self.from, Encoding::UTF8) {
            return Err(Error::UnsupportedConversion {
                from: self.from.name(),
                to: self.to.name(),
            });
        }

        let to_chars = crate::tables::get_encoding_chars(self.to);

        // Build reverse lookup table: char -> byte
        let mut char_to_byte = std::collections::HashMap::new();
        for (byte, &ch_opt) in to_chars.iter().enumerate() {
            if let Some(ch) = ch_opt {
                char_to_byte.insert(ch, byte as u8);
            }
        }

        let utf8_str = std::str::from_utf8(input)
            .map_err(|_| Error::InvalidInput("Invalid UTF-8 sequence".to_string()))?;

        let mut output = Vec::new();
        for (char_pos, ch) in utf8_str.char_indices() {
            if let Some(&byte) = char_to_byte.get(&ch) {
                output.push(byte);
            } else {
                return Err(Error::UnmappableTarget {
                    character: ch,
                    position: char_pos,
                });
            }
        }

        Ok(output)
    }

    /// Convert UTF-16 to UTF-8
    pub fn utf16_to_utf8(&self, input: &[u8]) -> Result<Vec<u8>> {
        if !matches!(self.from, Encoding::UTF16LE | Encoding::UTF16BE) {
            return Err(Error::UnsupportedConversion {
                from: self.from.name(),
                to: self.to.name(),
            });
        }

        // Ensure we have an even number of bytes for UTF-16
        if input.len() % 2 != 0 {
            return Err(Error::InvalidInput(
                "UTF-16 data must have even number of bytes".to_string(),
            ));
        }

        let mut utf16_chars = Vec::new();

        // Convert bytes to UTF-16 code units based on endianness
        for chunk in input.chunks_exact(2) {
            let code_unit = match self.from {
                Encoding::UTF16LE => u16::from_le_bytes([chunk[0], chunk[1]]),
                Encoding::UTF16BE => u16::from_be_bytes([chunk[0], chunk[1]]),
                _ => unreachable!(),
            };
            utf16_chars.push(code_unit);
        }

        // Convert UTF-16 code units to UTF-8
        let utf8_string = String::from_utf16(&utf16_chars)
            .map_err(|_| Error::InvalidInput("Invalid UTF-16 sequence".to_string()))?;

        Ok(utf8_string.into_bytes())
    }

    /// Convert UTF-8 to UTF-16
    pub fn utf8_to_utf16(&self, input: &[u8]) -> Result<Vec<u8>> {
        if !matches!(self.to, Encoding::UTF16LE | Encoding::UTF16BE) {
            return Err(Error::UnsupportedConversion {
                from: self.from.name(),
                to: self.to.name(),
            });
        }

        // Parse UTF-8 input
        let utf8_str = std::str::from_utf8(input)
            .map_err(|_| Error::InvalidInput("Invalid UTF-8 sequence".to_string()))?;

        // Convert to UTF-16 code units
        let utf16_chars: Vec<u16> = utf8_str.encode_utf16().collect();

        // Convert code units to bytes based on endianness
        let mut output = Vec::with_capacity(utf16_chars.len() * 2);

        for code_unit in utf16_chars {
            match self.to {
                Encoding::UTF16LE => output.extend_from_slice(&code_unit.to_le_bytes()),
                Encoding::UTF16BE => output.extend_from_slice(&code_unit.to_be_bytes()),
                _ => unreachable!(),
            }
        }

        Ok(output)
    }

    /// Convert between any two encodings via UTF-8 intermediate
    pub fn convert(&self, input: &[u8]) -> Result<Vec<u8>> {
        match (self.from, self.to) {
            // UTF-16 to UTF-8
            (Encoding::UTF16LE | Encoding::UTF16BE, Encoding::UTF8) => self.utf16_to_utf8(input),

            // UTF-8 to UTF-16
            (Encoding::UTF8, Encoding::UTF16LE | Encoding::UTF16BE) => self.utf8_to_utf16(input),

            // UTF-16 to UTF-16 (endianness conversion)
            (Encoding::UTF16LE | Encoding::UTF16BE, Encoding::UTF16LE | Encoding::UTF16BE) => {
                if self.from == self.to {
                    // Same encoding, just copy
                    Ok(input.to_vec())
                } else {
                    // Convert via UTF-8 for simplicity
                    let utf8_intermediate =
                        MultiByte::new(self.from, Encoding::UTF8).utf16_to_utf8(input)?;
                    MultiByte::new(Encoding::UTF8, self.to).utf8_to_utf16(&utf8_intermediate)
                }
            }

            // UTF-16 to single-byte encoding
            (Encoding::UTF16LE | Encoding::UTF16BE, _) => {
                let utf8_intermediate = self.utf16_to_utf8(input)?;
                MultiByte::new(Encoding::UTF8, self.to).from_utf8(&utf8_intermediate)
            }

            // Single-byte encoding to UTF-16
            (_, Encoding::UTF16LE | Encoding::UTF16BE) => {
                let utf8_intermediate = MultiByte::new(self.from, Encoding::UTF8).to_utf8(input)?;
                self.utf8_to_utf16(&utf8_intermediate)
            }

            // Direct UTF-8 output
            (_, Encoding::UTF8) => self.to_utf8(input),

            // Direct UTF-8 input
            (Encoding::UTF8, _) => self.from_utf8(input),

            // Single-byte to single-byte via UTF-8
            _ => {
                let utf8_intermediate = MultiByte::new(self.from, Encoding::UTF8).to_utf8(input)?;
                MultiByte::new(Encoding::UTF8, self.to).from_utf8(&utf8_intermediate)
            }
        }
    }
}
