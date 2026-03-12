# AML - ANSI Markup Language

A lightweight, tag-based markup language for applying ANSI terminal styles to text.

## Quick Start

### CLI

```bash
# Apply styles to plain text
echo "Hello" | aml -st "fr mb"

# Parse markup
aml '<fr>Red</f> and <fg>Green</f>'

# Install
cargo install --git https://github.com/bilakshanp/aml
```

### Library

```rust
use aml::prelude::*;

let doc = Document::new("<fr>Red text</f>");
println!("{}", doc.render());
```

Add to `Cargo.toml`:

```toml
[dependencies]
aml = { git = "https://github.com/bilakshanp/aml" }
```

## Markup Syntax

Colors: `<f...>text</f>` (foreground), `<b...>text</b>` (background)

Modifiers: `<m...>text</m>` (bold, italic, underline, etc.)

Shorthand: `<s f... b... m...>text</s>` (all combined)

Color formats: ANSI (`fr`, `fG`), hex (`f#ff0000`), RGB (`f255,0,0`), 256-palette (`f198`)

## Features

- Multiple color formats
- Text modifiers (bold, italic, underline, etc.)
- Natural nesting
- Minimal escape sequences
- Detailed error messages
- CLI + library

For detailed documentation, see [docs/](docs/).
