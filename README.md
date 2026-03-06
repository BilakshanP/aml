# AML - ANSI Markup Language

This library parses a lightweight tag-based markup language for applying ANSI terminal styles to text.

This workspace has two components:
- AML (Library): Contains the core logic of parsing and rendering.
- AML Cli (Binary): A simple cli which exposes the core functionality of the library.

---

## AML (Library)

### Examples

Run examples using:

```sh
cargo run --example <filename>
```

---

## AML Cli (Binary)

`aml` is a command-line renderer for the AML markup language. It reads a markup string or file, parses it, and writes the styled ANSI output to stdout.

### Usage

```sh
aml [OPTIONS] <MARKUP|--file <FILE>>
```

### Examples

```sh
# render a markup string directly
aml '<f198>Hello</f><f#ff0>,</f> <fB>World</f><mk>!!</m>'

# read from a `.aml` file
aml -nf ./hello-world.aml

# read from `stdin` or piped input
echo 'Hi' | aml -ns

# composition with other commands
printf '%s' "$(aml -n '<mb>prefix</m>') rest of line"

# inspect emitted escape sequences
aml --raw '<mbi>bold italic</m>'

# check out the help menu for detailed information
aml --help
```

### Error Reporting

Parse errors are printed to stderr with a visual span highlight showing exactly where in the input the problem occurred. The process exits with code `1` on failure and `0` on success.

```
Error: found 'q' expected HEX colour, RGB colour, ANSI colour, or Fixed
   ╭─[ input:1:3 ]
   │
 1 │ <fq>oops</f>
   │   ┬
   │   ╰── found 'q' expected HEX colour, RGB colour, ANSI colour, or Fixed
───╯
```

### Installation

```sh
cargo install --path aml-cli # executable `aml`

# or

cargo install --git https://github.com/bilakshanp/aml
```

To uninstall execute:

```sh
cargo uninstall aml-cli
```

---

## Markup Syntax Specification

### Text

Any plain text outside of tags is rendered as-is.

Supported escape sequences: `\<, \\, \n, \t, \r, \0, \e, \x`.

| Name | Escape | Sequence |
| --- | --- | --- |
| Left Angular Bracket | `\<` | `<` |
| Backslash | `\\` | `\` |
| Newline | `\n` | `LF` |
| Tab | `\t` | `Tab` |
| Carriage Return | `\r` | `CR` |
| Null Character | `\0` | `NUL` |
| Escape | `\e` | `ESC` |
| Control Sequence Introducer | `\c` | `\e[` |
| CSI Reset | `\x` | `\e[0m` |

Note: Multicharacter escape sequences are handled differently.

### Tags

All tags follow the form `<tag …>content</tag>`. Tags can be nested arbitrarily.

1. Reset — `<>…</>`

Performs a hard SGR reset before rendering the children, clearing all active styles.

```
<>this text is unstyled</>
```

2. Foreground colour — `<f…>…</f>`

```
<fr>red text</f>
<fG>bright green text</f>
<f#ff8800>orange text</f>
<f0,128,255>RGB blue text</f>
<f214>256-palette index 214</f>
```

3. Background colour — `<b…>…</b>`

Same colour syntax as foreground.

```
<bk>black background</b>
<b#abc>shorthand hex background</b>
```

4. Modifiers — `<m…>…</m>`

One or more modifier letters written directly after `m`.

```
<mu>underline</m>
<mbi>bold and italic</m>
<milovu>modifiers love you</m>
```

5. Shorthand — `<s …>…</s>`

Combines foreground, background, and modifiers in a single tag. Each component is prefixed and separated by spaces. You can supply one, two, or all three.

```
<s fr bk mbi>bold italic red-on-black</s>
<s f#ff0000 mbs>bold, struck, hex-red foreground</s>
<s mb>bright blue background only</s>
```

6. Raw SGR — `<!…>…</!>`

Emits raw SGR code bytes verbatim. The codes are written as a string between `!` and `>`. A reset is automatically appended on close.

```
<!1;31m>bold red via raw codes</!>
```

### Colour Formats

| Format | Syntax | Example | Notes |
| --- | --- | --- | --- |
| ANSI standard | single letter | `r` `g` `b` `c` `m` `y` `w` `k` | lowercase = normal |
| ANSI bright | uppercase letter | `R` `G` `B` `C` `M` `Y` `W` `K` | uppercase = bright |
| Hex | `#` + 1/2/3/6 hex digits | `#f00`, `#ff0000`, `#abc` | short forms are expanded |
| RGB decimal | `R,G,B` | `0,255,128` | 0–255 per channel, no spaces |
| 256-palette | integer | `198` | 0–255 index |

Short hex forms expand as follows: `#f` → `#ffffff`, `#de` → `#dedede` (repeated), `#abc` → `#abcabc`.

### Modifier Letters

| Letter | Effect |
|---|---|
| `b` | Bold |
| `d` | Dim |
| `i` | Italic |
| `u` | Underline |
| `l` | Double underline |
| `k` | Blink |
| `r` | Rapid blink |
| `v` | Invert (swap fg/bg) |
| `h` | Hide |
| `s` | Strike |
| `o` | Overline |

Multiple modifier letters are written together with no separator: `bus` = bold + underline + strike.

### Nesting

Tags nest naturally. Inner styles layer on top of outer ones; a `<>…</>` reset clears everything at that level.

```
<fr>red <mbi>bold italic red</m> back to red</f>
<fr>red <>reset inside</> still red</f>
```

## Resources & References

- **VT100**: https://vt100.net/emu/
- **VT220**: https://vt100.net/docs/vt220-rm/
- **Wikipedia ANSI Escape Codes**: https://en.wikipedia.org/wiki/ANSI_escape_code
- **Kitty Protocol Extensions**: https://sw.kovidgoyal.net/kitty/protocol-extensions/
- **XTerm Control Sequences** (Thomas Dickey): https://invisible-island.net/xterm/ctlseqs/ctlseqs.html
- **ECMA-48 Standard**: https://ecma-international.org/publications-and-standards/standards/ecma-48/
- **WezTerm Escape Sequences**: https://wezterm.org/escape-sequences.html#operating-system-command-sequences
- **ISO/IEC 8613-6 (1994)**: https://cdn.standards.iteh.ai/samples/22943/27430574bc77421e9904253bb0dd6339/ISO-IEC-8613-6-1994.pdf
