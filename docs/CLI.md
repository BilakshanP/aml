# AML CLI Documentation

## Installation

```bash
cargo install --git https://github.com/bilakshanp/aml
```

To uninstall:

```bash
cargo uninstall aml-cli
```

## Usage

### Basic Syntax

```bash
aml [OPTIONS] <MARKUP|--file <FILE>|--stdin>
```

### Input Sources

One of the following is required:

- `<MARKUP>` - Direct markup string as argument
- `-s, --stdin` - Read markup from stdin
- `-f, --file <FILE>` - Read markup from file

### Options

- `-t, --style <STYLE>` - Apply style to input text (treats input as raw text, no parsing)
- `-r, --raw` - Show ANSI escape codes (debug mode)
- `-n, --no-newline` - Omit trailing newline

## Examples

### Direct Markup

```bash
aml '<fr>Red</f> and <fg>Green</f>'
aml '<s fr mb>bold red</s>'
aml '<f#ff0000>Hex red</f>'
```

### From File

```bash
aml -f styles.aml
aml -nf document.aml    # No trailing newline
```

### From Stdin

```bash
echo '<fr>Hello</f>' | aml -s
cat file.txt | aml -s
```

### Apply Style to Raw Text

The `-t, --style` flag applies a style without parsing the input as markup:

```bash
echo "Error" | aml -st "fr mb"          # Red bold
echo "Success" | aml -st "fg"           # Green
echo "Warning" | aml -st "fY mb"        # Yellow bold
```

With file input:

```bash
aml -f logfile.txt -t "fr"                # Style entire file red
```

With direct argument:

```bash
aml "Plain text" -t "fg mb"               # Green bold
```

### Debug Mode

View the ANSI escape codes:

```bash
aml --raw '<fr>Red</f>'
aml '<mb>Bold</m>' --raw
```

### Composition

```bash
printf '%s' "$(aml -n '<mb>prefix</m>') rest of line"
```

## Style Specification Format

Used with the `-t/--style` flag.

### Colors

**ANSI colours** (lowercase for normal, uppercase for bright):
- Normal: `r`, `g`, `b`, `c`, `m`, `y`, `w`, `k`
- Bright: `R`, `G`, `B`, `C`, `M`, `Y`, `W`, `K`

**Hex**: `#f`, `#ff0000`, `#abc` (short forms auto-expand)

**RGB decimal**: `255,0,128` (no spaces, 0-255 per channel)

**256-palette**: `0` to `255`

### Modifiers

Stack letters together: `mbu` = bold + underline

- `b` - Bold
- `d` - Dim
- `i` - Italic
- `u` - Underline
- `l` - Double underline
- `k` - Blink
- `r` - Rapid blink
- `v` - Invert
- `h` - Hide
- `s` - Strike
- `o` - Overline

### Format

Foreground: `f<color>`
Background: `b<color>`
Modifiers: `m<letters>`

Combine with spaces:

```bash
aml -st "fr mb"               # Red foreground, bold
aml -st "fG bk"               # Bright green foreground, black background
aml -st "f#ff00ff mbu"        # Magenta, bold, underline
```

## Error Handling

Parse errors show the exact location:

```
Error: found 'q' expected HEX color, RGB color, ANSI color, or fixed color
   ╭─[ input:1:3 ]
   │
 1 │ <fq>oops</f>
   │   ┬
   │   ╰── found 'q' expected HEX color, RGB color, ANSI color, or fixed color
───╯
```

Exit codes:
- `0` - Success
- `1` - Parse error or invalid input
