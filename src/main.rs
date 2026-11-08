extern crate sanitize_filename;

use encoding_rs::SHIFT_JIS;
use std::io::{self, IsTerminal, Read, Write};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TextEncoding {
    Utf8,
    Cp932,
}

impl TextEncoding {
    fn parse(value: &str) -> io::Result<Self> {
        match value.to_ascii_lowercase().as_str() {
            "utf-8" | "utf8" => Ok(Self::Utf8),
            "cp932" => Ok(Self::Cp932),
            _ => Err(invalid_input(format!(
                "unsupported encoding: {value} (expected utf-8 or cp932)"
            ))),
        }
    }

    fn decode(self, bytes: &[u8]) -> io::Result<String> {
        match self {
            Self::Utf8 => String::from_utf8(bytes.to_vec())
                .map_err(|err| invalid_data(format!("invalid UTF-8 input: {err}"))),
            Self::Cp932 => {
                let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(bytes);
                if had_errors {
                    Err(invalid_data("input contains invalid CP932 data"))
                } else {
                    Ok(decoded.into_owned())
                }
            }
        }
    }

    fn encode(self, text: &str) -> io::Result<Vec<u8>> {
        match self {
            Self::Utf8 => Ok(text.as_bytes().to_vec()),
            Self::Cp932 => {
                let (encoded, _, had_errors) = SHIFT_JIS.encode(text);
                if had_errors {
                    Err(invalid_data("output contains characters not representable in CP932"))
                } else {
                    Ok(encoded.into_owned())
                }
            }
        }
    }
}

fn invalid_input(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn print_usage() {
    println!(
        "\
USAGE:
    sanitize-filename [OPTIONS] <filename>
    sanitize-filename [OPTIONS] < stdin
OPTIONS:
    -h, --help           Show this help message
    -V, --version        Show version information
    -r, --replace <str>  Replacement string
    --encoding <name>    Text encoding: utf-8 or cp932 (default: utf-8)
    --windows            Enable Windows filename rules
    --no-windows         Disable Windows filename rules
    --truncate           Truncate filenames to 255 bytes
    --no-truncate        Disable truncation

EXAMPLES:
    sanitize-filename --windows \"a:b?.txt\"
    sanitize-filename --windows -r \"_\" \"a:b?.txt\"
    sanitize-filename --windows --encoding cp932 -r \"_\" \"a:b?.txt\"
"
    );
}

fn main() -> io::Result<()> {
    let mut input = None;
    let mut set_replacement = false;
    let mut set_encoding = false;
    let mut replacement = None;
    let mut truncate = None;
    let mut windows = None;
    let mut encoding = TextEncoding::Utf8;

    for arg in std::env::args().skip(1) {
        if set_replacement {
            replacement = Some(arg);
            set_replacement = false;
        } else if set_encoding {
            encoding = TextEncoding::parse(&arg)?;
            set_encoding = false;
        } else if arg == "-h" || arg == "--help" {
            print_usage();
            return Ok(());
        } else if arg == "-V" || arg == "--version" {
            println!("sanitize-filename {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        } else if arg == "-r" || arg == "--replace" {
            set_replacement = true;
        } else if arg == "--encoding" {
            set_encoding = true;
        } else if arg == "--truncate" {
            truncate = Some(true);
        } else if arg == "--no-truncate" {
            truncate = Some(false);
        } else if arg == "--windows" {
            windows = Some(true);
        } else if arg == "--no-windows" {
            windows = Some(false);
        } else if arg == "-" {
            input = None;
        } else {
            input = Some(arg);
        }
    }

    if set_replacement {
        return Err(invalid_input("missing value for --replace"));
    }
    if set_encoding {
        return Err(invalid_input("missing value for --encoding"));
    }

    let input = if let Some(input) = input {
        input
    } else {
        let mut buffer = Vec::new();
        io::stdin().read_to_end(&mut buffer)?;
        encoding.decode(&buffer)?
    };

    let mut options = sanitize_filename::Options::default();
    if let Some(ref replacement) = replacement {
        options.replacement = replacement;
    }
    if let Some(windows) = windows {
        options.windows = windows;
    }
    if let Some(truncate) = truncate {
        options.truncate = truncate;
    }

    let output = sanitize_filename::sanitize_with_options(&input, options);
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    // Preserve Unicode output on a Windows console. Pipes and redirection use
    // the explicitly selected encoding, independent of the console code page.
    if stdout.is_terminal() {
        writeln!(handle, "{output}")
    } else {
        let mut text = output;
        text.push('\n');
        let bytes = encoding.encode(&text)?;
        handle.write_all(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::TextEncoding;

    #[test]
    fn utf8_roundtrip() {
        let text = "日本語_テスト.txt";
        assert_eq!(TextEncoding::Utf8.decode(&TextEncoding::Utf8.encode(text).unwrap()).unwrap(), text);
    }

    #[test]
    fn cp932_roundtrip() {
        let text = "日本語_テスト.txt";
        let bytes = TextEncoding::Cp932.encode(text).unwrap();
        assert_ne!(bytes, text.as_bytes());
        assert_eq!(TextEncoding::Cp932.decode(&bytes).unwrap(), text);
    }

    #[test]
    fn reject_invalid_input() {
        assert!(TextEncoding::Utf8.decode(&[0xff]).is_err());
        assert!(TextEncoding::Cp932.decode(&[0x82]).is_err());
    }

    #[test]
    fn reject_unrepresentable_cp932_output() {
        assert!(TextEncoding::Cp932.encode("emoji: 😀").is_err());
    }

    #[test]
    fn parse_encoding() {
        assert_eq!(TextEncoding::parse("cp932").unwrap(), TextEncoding::Cp932);
        assert_eq!(TextEncoding::parse("UTF-8").unwrap(), TextEncoding::Utf8);
        assert!(TextEncoding::parse("xyz").is_err());
    }
}
