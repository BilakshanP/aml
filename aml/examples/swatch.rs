const RESET: &str = "\x1b[0m";

fn rgb_for_code(code: u32) -> (u32, u32, u32) {
    match code {
        0..=15 => {
            let level = if code > 8 {
                255
            } else if code == 7 {
                229
            } else {
                205
            };

            let r = if code == 8 {
                127
            } else if code == 12 {
                92
            } else if (code & 1) != 0 {
                level
            } else {
                0
            };

            let g = if code == 8 {
                127
            } else if code == 12 {
                92
            } else if (code & 2) != 0 {
                level
            } else {
                0
            };

            let b = if code == 8 {
                127
            } else if code == 4 {
                238
            } else if (code & 4) != 0 {
                level
            } else {
                0
            };

            (r, g, b)
        }

        16..=231 => {
            let idx = code - 16;
            let chan = |c| if c != 0 { c * 40 + 55 } else { 0 };
            (chan(idx / 36), chan((idx % 36) / 6), chan(idx % 6))
        }

        232..=255 => {
            let level = (code - 232) * 10 + 8;
            (level, level, level)
        }

        _ => (0, 0, 0),
    }
}

fn swatch(code: u32) -> String {
    let (r, g, b) = rgb_for_code(code);
    let luma = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
    let label = if luma > 128.0 { 0 } else { 15 };

    format!("\x1b[48;5;{code}m\x1b[38;5;{label}m{code:4} {RESET}")
}

fn print_system_colours() {
    print!("\n System colours (0–15)\n\n ");
    (0..8).for_each(|c| print!("{}", swatch(c)));
    print!("\n ");
    (8..16).for_each(|c| print!("{}", swatch(c)));
    println!();
}

fn print_colour_cube() {
    println!("\n Colour Cube (16–231)\n");

    for green_group in [0..3u32, 3..6u32] {
        for red in 0..6u32 {
            for green in green_group.clone() {
                print!(" ");

                for blue in 0..6u32 {
                    print!("{}", swatch(16 + red * 36 + green * 6 + blue));
                }

                print!(" ");
            }

            println!();
        }

        println!();
    }
}

fn print_grayscale() {
    print!(" Grayscale Ramp (232–255)\n\n ");

    for i in 0..24u32 {
        print!("{}", swatch(232 + i));

        if (i + 1) % 12 == 0 {
            print!("\n ");
        }
    }
}

fn main() {
    println!("\n This demonstrates all 256 Fixed colours of the terminal.");

    print_system_colours();
    print_colour_cube();
    print_grayscale();

    println!("{RESET}");
}
