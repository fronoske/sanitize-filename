
extern crate sanitize_filename;

fn print_usage() {
    println!(
        "\
USAGE:
    sanitize-filename [OPTIONS] <filename>
    sanitize-filename [OPTIONS] < stdin

OPTIONS:
    -h, --help           Show this help message
    -r, --replace <str>  Replacement string
    --windows            Enable Windows filename rules
    --no-windows         Disable Windows filename rules
    --truncate           Truncate filenames to 255 bytes
    --no-truncate        Disable truncation

EXAMPLES:
    sanitize-filename --windows \"a:b?.txt\"
    sanitize-filename --windows -r \"_\" \"a:b?.txt\"
"
    );
}

fn main() -> Result<(), ::std::io::Error> {
    let mut input = None;
    let mut set_replacement = false;
    let mut replacement = None;
    let mut truncate = None;
    let mut windows = None;

    for arg in ::std::env::args().skip(1) {
        if set_replacement {
            replacement = Some(arg);
            set_replacement = false;
        } else if arg == "-h" || arg == "--help" {
            print_usage();
            return Ok(());
        } else if arg == "-r" || arg == "--replace" {
            set_replacement = true;
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

    let input = if let Some(input) = input {
        input
    } else {
        let mut buffer = String::new();
        ::std::io::Read::read_to_string(
            &mut ::std::io::stdin(),
            &mut buffer
        )?;
        buffer
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

    let output =
        sanitize_filename::sanitize_with_options(&input, options);

    println!("{}", &output);

    Ok(())
}
