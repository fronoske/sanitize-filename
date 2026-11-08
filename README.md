# sanitize-filename

A basic filename sanitizer, based on Node's [sanitize-filename](https://www.npmjs.com/package/sanitize-filename).

Use like:

```rust
extern crate sanitize_filename;

fn main() {
    println!("{}", sanitize_filename::sanitize("some-user-defined/../../../string"));
    // prints some-user-defined......string
}
```

You can also configure a few basic options:
```rust
let options = sanitize_filename::Options {
    truncate: true, // true by default, truncates to 255 bytes
    windows: true, // default value depends on the OS, removes reserved names like `con` from start of strings on Windows
    replacement: "" // str to replace sanitized chars/strings
};

let sanitized = sanitize_filename::sanitize_with_options("some-user-defined/../../../string", options);
```

Also provides a basic command line binary. Use like:
```bash
cargo install sanitize-filename
sanitize-filename my_filename.txt
```

```
Pass a file name to clean to the program (also reads STDIN)

FLAGS:
    -h, --help                 Show help
    -V, --version              Show version
    -r, --replace <r>          Replacement characters
    --encoding <name>          Text encoding: utf-8 or cp932 (default: utf-8)
    --windows, --no-windows    Whether to handle filenames for Windows
    --truncate, --no-truncate  Whether to truncate file names to 255 bytes
```

### CLI text encoding

The `--encoding utf-8|cp932` option controls byte decoding from standard input and byte encoding to standard output when redirected to a pipe or file. The default is `utf-8`, preserving existing behavior. Windows command-line arguments remain Unicode, independent of this option. When writing directly to a terminal, the CLI uses normal Unicode console output instead of encoding arbitrary CP932 bytes.

`cp932` is implemented using `encoding_rs::SHIFT_JIS` (a Windows-31J-compatible mapping, not a guarantee of byte-for-byte CP932 parity). Invalid input sequences and characters that cannot be encoded cause a nonzero exit code rather than silent substitutions. Both input and output use the same encoding; mixed-encoding conversion is not supported.

Examples in Windows CMD:

```cmd
sanitize-filename --windows --encoding cp932 -r "_" "日本語:テスト?.txt"
sanitize-filename --windows --encoding cp932 -r "_" < input-cp932.txt > output-cp932.txt
```

`--encoding` expects a separate argument (for example `--encoding cp932`, not `--encoding=cp932`).
