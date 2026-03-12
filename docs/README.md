# AML Documentation

Welcome to the AML (ANSI Markup Language) documentation. Choose a topic below:

## Getting Started

- **[CLI Documentation](CLI.md)** - Command-line interface usage and examples
- **[Library Documentation](LIBRARY.md)** - Rust API, features, and examples
- **[Markup Syntax](MARKUP.md)** - Complete markup language reference

## Quick Links

### I want to...

**Use the CLI**
- Apply styles to text: See [CLI.md - Apply Style to Raw Text](CLI.md#apply-style-to-raw-text)
- Parse markup files: See [CLI.md - From File](CLI.md#from-file)
- View escape codes: See [CLI.md - Debug Mode](CLI.md#debug-mode)

**Use the library**
- Parse and render markup: See [LIBRARY.md - Parsing and Rendering](LIBRARY.md#parsing-and-rendering)
- Apply styles at runtime: See [LIBRARY.md - Styler Feature](LIBRARY.md#styler-feature)
- Compile styles at compile-time: See [LIBRARY.md - Macros Feature](LIBRARY.md#macros-feature)

**Learn the syntax**
- Write markup: See [MARKUP.md - Tag Types](MARKUP.md#tag-types)
- Use colors: See [MARKUP.md - Colors](MARKUP.md#colors)
- Nest styles: See [MARKUP.md - Nesting](MARKUP.md#nesting)

## Features

- Multiple color formats (ANSI, hex, RGB, 256-palette)
- Text modifiers (bold, italic, underline, etc.)
- Natural nesting and composition
- Minimal escape sequences (optimized output)
- Detailed error messages with visual spans
- Both CLI tool and Rust library
- Compile-time macros for zero-overhead styles

## Installation

### CLI
```bash
cargo install --git https://github.com/bilakshanp/aml
```

### Library
```toml
[dependencies]
aml = { git = "https://github.com/bilakshanp/aml" }
```

## Examples

### CLI - Apply style to plain text
```bash
echo "Error" | aml -s -t "fr mb"
```

### CLI - Parse markup
```bash
aml '<fr>Red</f> and <fg>Green</f>'
```

### Library - Parse and render
```rust
use aml::prelude::*;

let doc = Document::new("<fr>Red text</f>");
println!("{}", doc.render());
```

### Library - Apply style at runtime
```rust
use aml::styler::Style;

Style::apply("fr mb", "Bold red")?;
```

### Library - Compile-time styles
```rust
use aml::style;

let red = style!("fr mb");
println!("{}Error{}", red, aml::render::RESET);
```

## References

- [VT100](https://vt100.net/emu/)
- [VT220](https://vt100.net/docs/vt220-rm/)
- [Wikipedia ANSI Escape Codes](https://en.wikipedia.org/wiki/ANSI_escape_code)
- [Kitty Protocol Extensions](https://sw.kovidgoyal.net/kitty/protocol-extensions/)
- [XTerm Control Sequences (Thomas Dickey)](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html)
- [ECMA-48 Standard](https://ecma-international.org/publications-and-standards/standards/ecma-48/)
- [WezTerm Escape Sequences](https://wezterm.org/escape-sequences.html#operating-system-command-sequences)
- [ISO/IEC 8613-6 (1994)](https://cdn.standards.iteh.ai/samples/22943/27430574bc77421e9904253bb0dd6339/ISO-IEC-8613-6-1994.pdf)
