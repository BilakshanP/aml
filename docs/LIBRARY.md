# AML Library Documentation

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
aml = { git = "https://github.com/bilakshanp/aml" }
```

## Basic Usage

### Parsing and Rendering

Parse markup and render to ANSI:

```rust
use aml::prelude::*;

fn main() {
    let markup = "<fr>Red text</f>";
    let doc = Document::new(markup);
    let output = doc.render();
    println!("{}", output);
}
```

### Error Handling

Handle parse errors gracefully:

```rust
use aml::prelude::*;

match Document::try_new(markup) {
    Ok(doc) => println!("{}", doc.render()),
    Err(errs) => {
        eprintln!("Parse error");
    }
}
```

With diagnostics feature for detailed error messages:

```toml
[dependencies]
aml = { git = "https://github.com/bilakshanp/aml", features = ["diagnostics"] }
```

```rust
use aml::prelude::*;

match Document::try_new(markup) {
    Ok(doc) => println!("{}", doc.render()),
    Err(errs) => {
        report(markup, "input", errs, &mut std::io::stdout()).ok();
    }
}
```

## Styler Feature

Apply styles to text without parsing markup.

```toml
[dependencies]
aml = { git = "https://github.com/bilakshanp/aml", features = ["styler"] }
```

### Runtime Style Application

```rust
use aml::styler::Style;

match Style::apply("fr mb", "Hello World") {
    Ok(styled) => println!("{}", styled),
    Err(_) => eprintln!("Invalid style"),
}
```

Supports all color formats:

```rust
Style::apply("fg", "Green")?;                      // ANSI
Style::apply("f#ff0000 mbi", "Red bold italic")?;  // Hex + modifiers
Style::apply("fY bB mu", "Yellow on blue")?;       // Bright colors
Style::apply("f255,128,0", "RGB")?;                // RGB decimal
Style::apply("f196", "Palette")?;                  // 256-palette
```

### CompiledStyle

Pre-compile a style for reuse:

```rust
use aml::styler::Style;

let error_style = Style::new("fr mb")?.compile();
let warning_style = Style::new("fY mb")?.compile();

println!("{}", error_style.paint("Error"));
println!("{}", warning_style.paint("Warning"));
```

## Macros Feature

Compile styles at compile-time with zero runtime overhead.

This, is the recommended method to create style which are used a lot in your codebase.

```toml
[dependencies]
aml = { git = "https://github.com/bilakshanp/aml", features = ["macros"] }
```

### style! Macro

Provides compile-time syntax checking.

```rust
use aml::style;
use aml::styler::CompiledStyle;

const GREEN: CompiledStyle = style!("fg");
const RED_BOLD: CompiledStyle = style!("fr mb");

fn main() {
    println!("{}", GREEN.paint("Success"));
    println!("{}", RED_BOLD.paint("Error"));
}
```

The macro expands to pre-compiled ANSI codes:

```rust
const ERROR_STYLE: CompiledStyle = style!("fr");
const WARNING_STYLE: CompiledStyle = style!("fY mb");
const SUCCESS_STYLE: CompiledStyle = style!("fg");

println!("{}: Build failed", ERROR_STYLE.paint("ERROR"));
println!("{}: Check config", WARNING_STYLE.paint("WARNING"));
println!("{}: Done success", SUCCESS_STYLE.paint("OK"));
```

Supports all color formats at compile time:

```rust
style!("f#ff0000")      // Hex
style!("f255,128,0")    // RGB decimal
style!("f196")          // 256-palette
style!("fG mbi")        // Bright ANSI + modifiers
```

## Markup Syntax

See [MARKUP.md](MARKUP.md) for full syntax documentation.

Quick reference:

```rust
// Foreground color
Document::new("<fr>red</f>")

// Background color
Document::new("<bk>black background</b>")

// Modifiers
Document::new("<mbi>bold italic</m>")

// Shorthand (all combined)
Document::new("<s fr bk mbi>red on black, bold italic</s>")

// Reset
Document::new("<fr>red <>reset</> red again</f>")

// Nesting
Document::new("<fr>red <fg>green</f> red</f>")
```

## Rendering

The `Document::render()` method outputs ANSI-escaped text:

```rust
let doc = Document::new("<fr>Red</f>");
let output = doc.render();
println!("{}", output);  // Displays red text in terminal
```

The renderer optimizes escape sequences:

- Minimal state transitions (only emits codes when styles change)
- Automatic deduplication of modifiers
- Efficient composition of nested styles

## Features Comparison

| Feature | Use Case |
|---------|----------|
| `Document::new()` | Parse and render complex markup |
| `Style::apply()` | Dynamic styles at runtime |
| `style!` macro | Static styles with zero overhead |

## API Overview

### Document

- `Document::new(input: &str) -> Self` - Parse and panic on error
- `Document::try_new(input: &str) -> Result<Self, Vec<Error>>` - Parse with error handling
- `document.render() -> String` - Render to ANSI-escaped string

### Style (with styler feature)

- `Style::new(spec: &str) -> Result<Self, Vec<Error>>` - Parse style spec
- `style.compile() -> CompiledStyle` - Compile to ANSI codes
- `Style::apply(spec, text) -> Result<String, Vec<Error>>` - Parse and apply in one step

### CompiledStyle

- `compiled_style.paint(text: &str) -> String` - Apply style to text

### Diagnostics (with diagnostics feature)

- `report(input, filename, errors, output)` - Print detailed error messages

## Examples

### Simple Markup

```rust
use aml::prelude::*;

let doc = Document::new("<s fr mb>bold red</s>");
println!("{}", doc.render());
```

### Error Reporting

```rust
use aml::prelude::*;

let input = "<fq>invalid</f>";
match Document::try_new(input) {
    Ok(doc) => println!("{}", doc.render()),
    Err(errs) => {
        report(input, "style.aml", errs, &mut std::io::stdout()).ok();
    }
}
```

### Compile-Time Styles

```rust
use aml::style;

struct Logger;

impl Logger {
    pub fn error(msg: &str) {
        let error = style!("fr mb").paint("ERROR");
        eprintln!("{}: {}", error, msg);
    }

    pub fn warn(msg: &str) {
        let warn = style!("fY mb").paint("WARN");
        eprintln!("{}: {}", warn, msg);
    }
}

fn main() {
    Logger::error("Something failed");
    Logger::warn("Check configuration");
}
```

## Performance

The renderer is optimized for minimal output:

- State tracking avoids redundant escape sequences
- Only emits codes when necessary
- Modifiers are deduplicated across layers

Compile-time macros (`style!`) produce zero-overhead code.

Runtime styling (`Style::apply`) parses at runtime but is still fast.

Complex markup parsing is efficient due to the optimized parser.
