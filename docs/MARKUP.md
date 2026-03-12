# AML Markup Syntax

## Overview

AML is a tag-based markup language for terminal styling. Tags follow the form `<tag>content</tag>` and can nest arbitrarily.

## Text and Escapes

Any plain text outside of tags is rendered as-is.

### Escape Sequences

| Escape | Result |
|--------|--------|
| `\<` | Literal `<` |
| `\\` | Literal `\` |
| `\n` | Newline (LF) |
| `\t` | Tab |
| `\r` | Carriage return (CR) |
| `\0` | Null character (NUL) |
| `\e` | Escape character (ESC) |
| `\c` | Control Sequence Introducer (`\e[`) |
| `\x` | CSI Reset (`\e[0m`) |

Example:

```
This is \<not a tag\>
This has a\ttab
```

## Tag Types

### Foreground Color: `<f...>...</f>`

Sets the text foreground color.

```
<fr>red text</f>
<fG>bright green</f>
<f#ff0000>hex red</f>
<f255,0,0>RGB red</f>
<f196>palette index</f>
```

### Background Color: `<b...>...</b>`

Sets the text background color. Same color syntax as foreground.

```
<bk>black background</b>
<b#ffffff>white background</b>
<bY>bright yellow background</b>
```

### Modifiers: `<m...>...</m>`

Applies text modifiers. Write modifier letters directly after `m`.

```
<mb>bold</m>
<mi>italic</m>
<mbi>bold italic</m>
<mbu>bold underline</m>
```

| Letter | Effect |
|--------|--------|
| `b` | Bold |
| `d` | Dim |
| `i` | Italic |
| `u` | Underline |
| `l` | Double underline |
| `k` | Blink |
| `r` | Rapid blink |
| `v` | Invert (swap foreground/background) |
| `h` | Hide |
| `s` | Strike |
| `o` | Overline |

### Shorthand: `<s ...>...</s>`

Combines foreground, background, and modifiers in a single tag. Separate components with spaces.

```
<s fr bk mbi>red on black, bold italic</s>
<s f#ff0000 mb>hex red, bold</s>
<s f#fff mbu>white, bold, underline</s>
<s bk>just black background</s>
```

The shorthand can contain:
- One foreground color: `f<color>`
- One background color: `b<color>`
- One modifier group: `m<letters>`

All are optional, but at least one must be present.

### Reset: `<>...</>`

Performs a hard SGR reset, clearing all active styles at that level.

```
<fr>red <>reset</> still red</f>
<mbi>bold italic <>reset</> back to bi</m>
```

Reset clears foreground, background, and all modifiers. Styles from outer tags are restored.

### Raw SGR: `<!...>...</!>`

Emits raw ANSI SGR codes directly. The codes are written as a string between `!` and `>`.

```
<!1;31m>bold red via raw codes</!>
<!38;2;255;0;0m>24-bit RGB red</!>
```

A reset (`\e[0m`) is automatically appended when the raw tag closes.

## Colors

### ANSI Standard (8 colors)

Single letter. Lowercase for normal brightness, uppercase for bright.

Standard: `r` (red), `g` (green), `b` (blue), `c` (cyan), `m` (magenta), `y` (yellow), `w` (white), `k` (black)

Bright: `R`, `G`, `B`, `C`, `M`, `Y`, `W`, `K`

```
<fr>red</f>           # Normal red
<fR>bright red</f>    # Bright red
<fc>cyan</f>          # Normal cyan
<fC>bright cyan</f>   # Bright cyan
```

### Hex Colors

Format: `#` followed by 1, 2, 3, or 6 hex digits.

Short forms are auto-expanded:
- `#f` expands to `#ffffff`
- `#de` expands to `#dedede`
- `#abc` expands to `#aabbcc`

```
<f#f00>red</f>           # Short form
<f#ff0000>red</f>        # Full form
<f#abc>light gray</f>    # Short form
```

### RGB Decimal

Format: `R,G,B` where each component is 0-255, no spaces.

```
<f255,0,0>red</f>
<f0,255,0>green</f>
<f0,0,255>blue</f>
<f255,128,64>orange</f>
```

### 256-Color Palette

Format: integer 0-255.

```
<f0>black</f>
<f196>light blue</f>
<f226>bright yellow</f>
```

## Nesting

Tags nest naturally. Inner styles layer on top of outer ones.

```
<fr>red <mb>bold red</m> back to normal red</f>
<mbi>bold italic <fg>green bold italic</f> still bold italic</m>
<s fr bk>red on black <mu>also underlined</m> still red on black</s>
```

When an inner tag changes a style (e.g., foreground color), it overrides the outer style. When the inner tag closes, the outer style is restored.

### Reset Inside Tags

Reset clears styles within its scope, then restores parent styles:

```
<fr>red <>reset</> still red</f>
<mbi>bold italic <>reset</> back to bi</m>
<s fr bk mbi>red on black bold italic <>reset</> same as before</s>
```

## Examples

### Simple Colors

```
<fr>Error</f>
<fg>Success</f>
<fy>Warning</fy>
```

### Complex Styling

```
<s fr bk mb>CRITICAL: /var/log/app.log is full</s>
```

### Nested Styles

```
<mb>Bold <fg>green bold</f> back to regular bold</mb>
```

### Composition

```
<fr>ERROR</>: Database connection failed
<fg>OK</>: All systems operational
```

### With Escapes

```
This is \<not a tag\>
Path: /usr/local/bin
```

## Best Practices

1. Use shorthand (`<s>`) for simple cases with multiple attributes
2. Use individual tags (`<f>`, `<b>`, `<m>`) when nesting complex styles
3. Use `<>` to clearly mark style boundaries
4. Escape `<` with `\<` if you need literal angle brackets
5. Avoid deeply nested tags when possible for readability

## Limitations

- Tags must be properly closed
- Unmatched tags result in parse errors
- Modifiers must be valid letters (no validation of invalid combinations)
- Color values must be valid (invalid formats are parse errors)
