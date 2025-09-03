//! # FastEncode CLI - Enterprise Character Encoding Converter
//!
//! Command-line interface for high-performance character encoding conversions
//! supporting mainframe data processing and enterprise file formats.

#[cfg(feature = "cli")]
use std::fs;
#[cfg(feature = "cli")]
use std::io::{self, Read, Write};
#[cfg(feature = "cli")]
use std::path::PathBuf;

#[cfg(feature = "cli")]
use anyhow::{Context, Result};
#[cfg(feature = "cli")]
use clap::{Args, Parser, Subcommand, ValueEnum};
#[cfg(feature = "cli")]
use serde::Serialize;

use fast_encode::detection::EncodingDetector;
use fast_encode::{Encoding, Error as EncodeError, Translator};

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("CLI features disabled. Enable with --features cli");
    std::process::exit(1);
}

/// FastEncode: High-performance character encoding converter
#[cfg(feature = "cli")]
#[derive(Parser)]
#[command(name = "fast-encode")]
#[command(version, about, long_about = None)]
#[command(author = "FastEncode Contributors")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Output format (text, json)
    #[arg(long, global = true, default_value = "text")]
    format: OutputFormat,
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
enum Commands {
    /// Convert files between character encodings
    Convert(ConvertArgs),

    /// Detect encoding of input files
    Detect(DetectArgs),

    /// List all supported encodings
    List(ListArgs),

    /// Validate that a file is properly encoded
    Validate(ValidateArgs),

    /// Display detailed information about an encoding
    Info(InfoArgs),
}

#[cfg(feature = "cli")]
#[derive(Args)]
struct ConvertArgs {
    /// Source encoding
    #[arg(short = 'f', long = "from")]
    from: EncodingArg,

    /// Target encoding  
    #[arg(short = 't', long = "to")]
    to: EncodingArg,

    /// Input file (stdin if not specified)
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Output file (stdout if not specified)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Convert in-place (overwrite input file)
    #[arg(long, conflicts_with = "output")]
    in_place: bool,

    /// Use lossy conversion with replacement character
    #[arg(long)]
    lossy: bool,

    /// Replacement character for lossy conversion (default: ?)
    #[arg(long, default_value = "?")]
    replacement: String,

    /// Strip BOM from input
    #[arg(long)]
    strip_bom: bool,

    /// Add BOM to output  
    #[arg(long)]
    add_bom: bool,

    /// Process files recursively (if input is directory)
    #[arg(short, long)]
    recursive: bool,

    /// Buffer size for large files (KB)
    #[arg(long, default_value = "64")]
    buffer_size: usize,
}

#[cfg(feature = "cli")]
#[derive(Args)]
struct DetectArgs {
    /// Input file (stdin if not specified)
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Show confidence scores
    #[arg(long)]
    confidence: bool,

    /// Maximum bytes to read for detection
    #[arg(long, default_value = "8192")]
    sample_size: usize,

    /// Language hint for better detection accuracy
    #[arg(long)]
    language: Option<String>,
}

#[cfg(feature = "cli")]
#[derive(Args)]
struct ListArgs {
    /// Filter by category (unicode, windows, iso, ebcdic, dos, mac, asian)
    #[arg(short, long)]
    category: Option<String>,

    /// Show only ASCII-compatible encodings
    #[arg(long)]
    ascii_compatible: bool,

    /// Show only multibyte encodings
    #[arg(long)]
    multibyte: bool,

    /// Show encoding details
    #[arg(long)]
    details: bool,
}

#[cfg(feature = "cli")]
#[derive(Args)]
struct ValidateArgs {
    /// Input file (stdin if not specified)
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Expected encoding
    #[arg(short, long)]
    encoding: EncodingArg,

    /// Show position of first error
    #[arg(long)]
    show_errors: bool,
}

#[cfg(feature = "cli")]
#[derive(Args)]
struct InfoArgs {
    /// Encoding to describe
    encoding: EncodingArg,

    /// Show character mapping samples
    #[arg(long)]
    samples: bool,
}

#[cfg(feature = "cli")]
#[derive(Clone, Debug, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[cfg(feature = "cli")]
#[derive(Clone, Debug)]
enum EncodingArg {
    Encoding(Encoding),
}

impl std::str::FromStr for EncodingArg {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let encoding = match s.to_uppercase().as_str() {
            "UTF8" | "UTF-8" => Encoding::UTF8,
            "UTF16LE" | "UTF-16LE" => Encoding::UTF16LE,
            "UTF16BE" | "UTF-16BE" => Encoding::UTF16BE,
            "ASCII" | "US-ASCII" => Encoding::ASCII,

            // ISO-8859 series
            "ISO88591" | "ISO-8859-1" | "LATIN1" => Encoding::ISO_8859_1,
            "ISO88592" | "ISO-8859-2" | "LATIN2" => Encoding::ISO_8859_2,
            "ISO88593" | "ISO-8859-3" | "LATIN3" => Encoding::ISO_8859_3,
            "ISO88594" | "ISO-8859-4" | "LATIN4" => Encoding::ISO_8859_4,
            "ISO88595" | "ISO-8859-5" => Encoding::ISO_8859_5,
            "ISO88596" | "ISO-8859-6" => Encoding::ISO_8859_6,
            "ISO88597" | "ISO-8859-7" => Encoding::ISO_8859_7,
            "ISO88598" | "ISO-8859-8" => Encoding::ISO_8859_8,
            "ISO88599" | "ISO-8859-9" | "LATIN5" => Encoding::ISO_8859_9,
            "ISO885910" | "ISO-8859-10" | "LATIN6" => Encoding::ISO_8859_10,
            "ISO885911" | "ISO-8859-11" => Encoding::ISO_8859_11,
            "ISO885913" | "ISO-8859-13" | "LATIN7" => Encoding::ISO_8859_13,
            "ISO885914" | "ISO-8859-14" | "LATIN8" => Encoding::ISO_8859_14,
            "ISO885915" | "ISO-8859-15" | "LATIN9" => Encoding::ISO_8859_15,
            "ISO885916" | "ISO-8859-16" | "LATIN10" => Encoding::ISO_8859_16,

            // Windows code pages
            "WINDOWS1250" | "WIN1250" | "CP1250" => Encoding::WINDOWS_1250,
            "WINDOWS1251" | "WIN1251" | "CP1251" => Encoding::WINDOWS_1251,
            "WINDOWS1252" | "WIN1252" | "CP1252" => Encoding::WINDOWS_1252,
            "WINDOWS1253" | "WIN1253" | "CP1253" => Encoding::WINDOWS_1253,
            "WINDOWS1254" | "WIN1254" | "CP1254" => Encoding::WINDOWS_1254,
            "WINDOWS1255" | "WIN1255" | "CP1255" => Encoding::WINDOWS_1255,
            "WINDOWS1256" | "WIN1256" | "CP1256" => Encoding::WINDOWS_1256,
            "WINDOWS1257" | "WIN1257" | "CP1257" => Encoding::WINDOWS_1257,
            "WINDOWS1258" | "WIN1258" | "CP1258" => Encoding::WINDOWS_1258,
            "WINDOWS874" | "WIN874" | "CP874" => Encoding::WINDOWS_874,

            // EBCDIC
            "EBCDIC037" | "IBM037" | "CP037" => Encoding::EBCDIC_037,
            "EBCDIC273" | "IBM273" | "CP273" => Encoding::EBCDIC_273,
            "EBCDIC277" | "IBM277" | "CP277" => Encoding::EBCDIC_277,
            "EBCDIC278" | "IBM278" | "CP278" => Encoding::EBCDIC_278,
            "EBCDIC280" | "IBM280" | "CP280" => Encoding::EBCDIC_280,
            "EBCDIC284" | "IBM284" | "CP284" => Encoding::EBCDIC_284,
            "EBCDIC285" | "IBM285" | "CP285" => Encoding::EBCDIC_285,
            "EBCDIC297" | "IBM297" | "CP297" => Encoding::EBCDIC_297,
            "EBCDIC500" | "IBM500" | "CP500" => Encoding::EBCDIC_500,
            "EBCDIC1047" | "IBM1047" | "CP1047" => Encoding::EBCDIC_1047,

            // DOS/OEM
            "CP437" | "DOS437" => Encoding::CP_437,
            "CP850" | "DOS850" => Encoding::CP_850,
            "CP852" | "DOS852" => Encoding::CP_852,
            "CP855" | "DOS855" => Encoding::CP_855,
            "CP857" | "DOS857" => Encoding::CP_857,
            "CP860" | "DOS860" => Encoding::CP_860,
            "CP861" | "DOS861" => Encoding::CP_861,
            "CP862" | "DOS862" => Encoding::CP_862,
            "CP863" | "DOS863" => Encoding::CP_863,
            "CP865" | "DOS865" => Encoding::CP_865,
            "CP866" | "DOS866" => Encoding::CP_866,

            // Mac
            "MACROMAN" | "MAC-ROMAN" => Encoding::MAC_ROMAN,
            "MACCYRILLIC" | "MAC-CYRILLIC" => Encoding::MAC_CYRILLIC,

            // Asian (placeholders)
            "SHIFTJIS" | "SHIFT-JIS" | "SHIFT_JIS" => Encoding::SHIFT_JIS,
            "EUCJP" | "EUC-JP" | "EUC_JP" => Encoding::EUC_JP,
            "GB2312" => Encoding::GB2312,
            "BIG5" => Encoding::BIG5,
            "EUCKR" | "EUC-KR" | "EUC_KR" => Encoding::EUC_KR,

            _ => anyhow::bail!("Unknown encoding: {}", s),
        };

        Ok(EncodingArg::Encoding(encoding))
    }
}

#[cfg(feature = "cli")]
#[derive(Serialize)]
struct ConversionResult {
    success: bool,
    bytes_processed: usize,
    bytes_written: usize,
    errors: Vec<String>,
    processing_time_ms: u64,
}

#[derive(Serialize)]
struct DetectionResult {
    detected_encoding: Option<String>,
    confidence: f64,
    bom_detected: bool,
    sample_size: usize,
}

#[cfg(feature = "cli")]
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Convert(ref args) => convert_command(args, &cli)?,
        Commands::Detect(ref args) => detect_command(args, &cli)?,
        Commands::List(ref args) => list_command(args, &cli)?,
        Commands::Validate(ref args) => validate_command(args, &cli)?,
        Commands::Info(ref args) => info_command(args, &cli)?,
    }

    Ok(())
}

#[cfg(feature = "cli")]
fn convert_command(args: &ConvertArgs, cli: &Cli) -> Result<()> {
    let start_time = std::time::Instant::now();

    let EncodingArg::Encoding(from_encoding) = &args.from;
    let EncodingArg::Encoding(to_encoding) = &args.to;

    if cli.verbose {
        eprintln!(
            "Converting from {} to {}",
            from_encoding.name(),
            to_encoding.name()
        );
    }

    let translator = Translator::new(*from_encoding, *to_encoding).with_context(|| {
        format!(
            "Failed to create translator from {} to {}",
            from_encoding.name(),
            to_encoding.name()
        )
    })?;

    // Read input
    let input_data = if let Some(ref input_path) = args.input {
        if cli.verbose {
            eprintln!("Reading from: {}", input_path.display());
        }
        fs::read(input_path)
            .with_context(|| format!("Failed to read input file: {}", input_path.display()))?
    } else {
        if cli.verbose {
            eprintln!("Reading from stdin");
        }
        let mut buffer = Vec::new();
        io::stdin()
            .read_to_end(&mut buffer)
            .context("Failed to read from stdin")?;
        buffer
    };

    let mut processed_data = input_data;

    // Handle BOM stripping
    if args.strip_bom {
        if let Some(bom) = from_encoding.bom() {
            if processed_data.starts_with(bom) {
                processed_data = processed_data[bom.len()..].to_vec();
                if cli.verbose {
                    eprintln!("Stripped BOM ({} bytes)", bom.len());
                }
            }
        }
    }

    // Convert
    let output_data = if args.lossy {
        let replacement_byte = args
            .replacement
            .chars()
            .next()
            .context("Invalid replacement character")? as u8;
        translator.convert_lossy(&processed_data, replacement_byte)
    } else {
        translator
            .convert(&processed_data)
            .context("Conversion failed")?
    };

    // Handle BOM addition
    let final_data = if args.add_bom {
        if let Some(bom) = to_encoding.bom() {
            let mut result = bom.to_vec();
            result.extend(output_data);
            result
        } else {
            output_data
        }
    } else {
        output_data
    };

    // Write output
    if args.in_place {
        if let Some(ref input_path) = args.input {
            fs::write(input_path, &final_data).with_context(|| {
                format!("Failed to write to input file: {}", input_path.display())
            })?;
            if cli.verbose {
                eprintln!("Updated file in-place: {}", input_path.display());
            }
        } else {
            anyhow::bail!("Cannot use --in-place without input file");
        }
    } else if let Some(ref output_path) = args.output {
        fs::write(output_path, &final_data)
            .with_context(|| format!("Failed to write output file: {}", output_path.display()))?;
        if cli.verbose {
            eprintln!("Wrote to: {}", output_path.display());
        }
    } else {
        io::stdout()
            .write_all(&final_data)
            .context("Failed to write to stdout")?;
    }

    let processing_time = start_time.elapsed();

    if cli.verbose {
        eprintln!(
            "Processed {} bytes -> {} bytes in {:?}",
            processed_data.len(),
            final_data.len(),
            processing_time
        );
    }

    // Output result in requested format
    match cli.format {
        OutputFormat::Json => {
            let result = ConversionResult {
                success: true,
                bytes_processed: processed_data.len(),
                bytes_written: final_data.len(),
                errors: Vec::new(),
                processing_time_ms: processing_time.as_millis() as u64,
            };
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        OutputFormat::Text => {
            if cli.verbose || args.output.is_none() {
                eprintln!("✓ Conversion completed successfully");
            }
        }
    }

    Ok(())
}

#[cfg(feature = "cli")]
fn detect_command(args: &DetectArgs, cli: &Cli) -> Result<()> {
    // Read sample data
    let sample_data = if let Some(ref input_path) = args.input {
        let mut file = std::fs::File::open(input_path)
            .with_context(|| format!("Failed to open input file: {}", input_path.display()))?;
        let mut buffer = vec![0u8; args.sample_size];
        let bytes_read = file.read(&mut buffer)?;
        buffer.truncate(bytes_read);
        buffer
    } else {
        let mut buffer = vec![0u8; args.sample_size];
        let bytes_read = io::stdin().read(&mut buffer)?;
        buffer.truncate(bytes_read);
        buffer
    };

    // Use sophisticated detection algorithm
    let detector = EncodingDetector::with_sample_size(args.sample_size);
    let detection_result = if let Some(ref language) = args.language {
        detector.detect_with_hint(&sample_data, language)
    } else {
        detector.detect(&sample_data)
    };

    match cli.format {
        OutputFormat::Json => {
            let mut candidates_json = Vec::new();
            for (encoding, confidence) in &detection_result.candidates {
                candidates_json.push(serde_json::json!({
                    "encoding": encoding.name(),
                    "confidence": confidence
                }));
            }

            let result = serde_json::json!({
                "detected_encoding": detection_result.encoding.name(),
                "confidence": detection_result.confidence,
                "bom_detected": detection_result.bom_detected,
                "sample_size": sample_data.len(),
                "candidates": candidates_json
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        OutputFormat::Text => {
            println!("Detected encoding: {}", detection_result.encoding.name());
            println!("Confidence: {:.1}%", detection_result.confidence * 100.0);

            if detection_result.bom_detected {
                println!("BOM detected: Yes");
            }

            println!("Sample size: {} bytes", sample_data.len());

            if args.confidence && detection_result.candidates.len() > 1 {
                println!("\nAll candidates:");
                for (encoding, confidence) in &detection_result.candidates {
                    println!("  {}: {:.1}%", encoding.name(), confidence * 100.0);
                }
            }
        }
    }

    Ok(())
}

#[cfg(feature = "cli")]
fn list_command(args: &ListArgs, cli: &Cli) -> Result<()> {
    let all_encodings = [
        (Encoding::UTF8, "unicode", "UTF-8 Unicode"),
        (Encoding::UTF16LE, "unicode", "UTF-16 Little Endian"),
        (Encoding::UTF16BE, "unicode", "UTF-16 Big Endian"),
        (Encoding::ASCII, "ascii", "US-ASCII (7-bit)"),
        (Encoding::ISO_8859_1, "iso", "ISO-8859-1 (Latin-1)"),
        (
            Encoding::ISO_8859_15,
            "iso",
            "ISO-8859-15 (Latin-9 with Euro)",
        ),
        (
            Encoding::WINDOWS_1250,
            "windows",
            "Windows-1250 (Central European)",
        ),
        (
            Encoding::WINDOWS_1252,
            "windows",
            "Windows-1252 (Western European)",
        ),
        (Encoding::CP_437, "dos", "DOS CP437 (US OEM)"),
        (Encoding::CP_850, "dos", "DOS CP850 (Western European OEM)"),
        (
            Encoding::EBCDIC_037,
            "ebcdic",
            "IBM EBCDIC CP037 (US/Canada)",
        ),
        (Encoding::MAC_ROMAN, "mac", "Macintosh Roman"),
    ];

    let filtered_encodings: Vec<_> = all_encodings
        .iter()
        .filter(|(encoding, category, _)| {
            if let Some(ref filter_cat) = args.category {
                if *category != filter_cat.as_str() {
                    return false;
                }
            }

            if args.ascii_compatible && !encoding.is_ascii_compatible() {
                return false;
            }

            if args.multibyte && !encoding.is_multibyte() {
                return false;
            }

            true
        })
        .collect();

    match cli.format {
        OutputFormat::Json => {
            let encodings_info: Vec<_> = filtered_encodings
                .iter()
                .map(|(encoding, category, description)| {
                    serde_json::json!({
                        "name": encoding.name(),
                        "category": category,
                        "description": description,
                        "ascii_compatible": encoding.is_ascii_compatible(),
                        "multibyte": encoding.is_multibyte(),
                        "has_bom": encoding.bom().is_some()
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&encodings_info)?);
        }
        OutputFormat::Text => {
            println!("Supported Encodings ({} total):", filtered_encodings.len());
            println!();

            for (encoding, category, description) in filtered_encodings {
                println!(
                    "{:15} {:10} {}",
                    encoding.name(),
                    format!("[{}]", category),
                    description
                );

                if args.details {
                    println!(
                        "                ASCII Compatible: {}",
                        if encoding.is_ascii_compatible() {
                            "Yes"
                        } else {
                            "No"
                        }
                    );
                    println!(
                        "                Multibyte: {}",
                        if encoding.is_multibyte() { "Yes" } else { "No" }
                    );
                    if let Some(bom) = encoding.bom() {
                        println!("                BOM: {:02X?}", bom);
                    }
                    println!();
                }
            }
        }
    }

    Ok(())
}

#[cfg(feature = "cli")]
fn validate_command(args: &ValidateArgs, _cli: &Cli) -> Result<()> {
    let EncodingArg::Encoding(encoding) = &args.encoding;

    // Read input
    let input_data = if let Some(ref input_path) = args.input {
        fs::read(input_path)
            .with_context(|| format!("Failed to read input file: {}", input_path.display()))?
    } else {
        let mut buffer = Vec::new();
        io::stdin().read_to_end(&mut buffer)?;
        buffer
    };

    // Try to convert to UTF-8 to validate
    let translator = Translator::new(*encoding, Encoding::UTF8)?;

    match translator.convert(&input_data) {
        Ok(_) => {
            println!("✓ File is valid {}", encoding.name());
            std::process::exit(0);
        }
        Err(e) => {
            println!("✗ File is not valid {}", encoding.name());

            if args.show_errors {
                match e {
                    EncodeError::UnmappableSource { byte, position } => {
                        println!(
                            "  Error at position {}: unmappable byte 0x{:02X}",
                            position, byte
                        );
                    }
                    EncodeError::UnmappableTarget {
                        character,
                        position,
                    } => {
                        println!(
                            "  Error at position {}: unmappable character '{}'",
                            position, character
                        );
                    }
                    _ => println!("  Error: {}", e),
                }
            }

            std::process::exit(1);
        }
    }
}

#[cfg(feature = "cli")]
fn info_command(args: &InfoArgs, cli: &Cli) -> Result<()> {
    let EncodingArg::Encoding(encoding) = &args.encoding;

    match cli.format {
        OutputFormat::Json => {
            let info = serde_json::json!({
                "name": encoding.name(),
                "ascii_compatible": encoding.is_ascii_compatible(),
                "multibyte": encoding.is_multibyte(),
                "bom": encoding.bom().map(|b| format!("{:02X?}", b)),
                "description": get_encoding_description(*encoding)
            });
            println!("{}", serde_json::to_string_pretty(&info)?);
        }
        OutputFormat::Text => {
            println!("Encoding Information: {}", encoding.name());
            println!("Description: {}", get_encoding_description(*encoding));
            println!(
                "ASCII Compatible: {}",
                if encoding.is_ascii_compatible() {
                    "Yes"
                } else {
                    "No"
                }
            );
            println!(
                "Multibyte: {}",
                if encoding.is_multibyte() { "Yes" } else { "No" }
            );

            if let Some(bom) = encoding.bom() {
                println!("BOM: {:02X?}", bom);
            } else {
                println!("BOM: None");
            }

            if args.samples {
                println!("\nCharacter Samples:");
                print_character_samples(*encoding);
            }
        }
    }

    Ok(())
}

#[cfg(feature = "cli")]
fn get_encoding_description(encoding: Encoding) -> &'static str {
    match encoding {
        Encoding::UTF8 => "Unicode Transformation Format 8-bit, variable-length encoding",
        Encoding::UTF16LE => "Unicode Transformation Format 16-bit, little-endian",
        Encoding::UTF16BE => "Unicode Transformation Format 16-bit, big-endian",
        Encoding::ASCII => "American Standard Code for Information Interchange (7-bit)",
        Encoding::ISO_8859_1 => "Latin alphabet No. 1, Western European",
        Encoding::ISO_8859_15 => "Latin alphabet No. 9, Western European with Euro symbol",
        Encoding::WINDOWS_1250 => "Windows code page for Central and Eastern European languages",
        Encoding::WINDOWS_1252 => "Windows code page for Western European languages",
        Encoding::CP_437 => "Original IBM PC character set with box-drawing characters",
        Encoding::EBCDIC_037 => "IBM Extended Binary Coded Decimal Interchange Code (US/Canada)",
        Encoding::MAC_ROMAN => "Classic Macintosh Roman character encoding",
        _ => "Character encoding for specific language/regional support",
    }
}

#[cfg(feature = "cli")]
fn print_character_samples(encoding: Encoding) {
    // Print some sample characters from the encoding
    let samples = match encoding {
        Encoding::UTF8 | Encoding::ASCII => {
            vec![(0x41, "A"), (0x61, "a"), (0x30, "0"), (0x21, "!")]
        }
        Encoding::WINDOWS_1252 => vec![(0x80, "€"), (0x99, "™"), (0xA9, "©"), (0xAE, "®")],
        Encoding::CP_437 => vec![(0xC9, "╔"), (0xCD, "═"), (0xBB, "»"), (0xF8, "°")],
        Encoding::EBCDIC_037 => vec![(0xC1, "A"), (0x81, "a"), (0xF0, "0"), (0x5A, "!")],
        _ => vec![(0x41, "A"), (0x61, "a")],
    };

    for (byte, desc) in samples {
        println!("  0x{:02X} -> {}", byte, desc);
    }
}
