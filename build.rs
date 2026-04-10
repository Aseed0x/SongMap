use image::{ImageBuffer, Rgba};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let size = 256u32;
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(size, size);

    // ── Background : dark blue-black ──────────────────────────────────────────
    let bg = Rgba([16u8, 16, 26, 255]);
    for p in img.pixels_mut() {
        *p = bg;
    }

    // ── 5 vertical equalizer bars (app colour palette) ────────────────────────
    // Heights, colours, and positions designed to be readable at 16x16–256x256
    let bar_w: u32 = 30;
    let gap:   u32 = 12;
    let n:     u32 = 5;
    let total_w = n * bar_w + (n - 1) * gap;          // 198
    let x_off  = (size - total_w) / 2;                // 29
    let bottom = size - 22;
    let radius = 6u32;

    // Heights (pixels from bottom) — tallest in center
    let heights: [u32; 5] = [130, 175, 215, 160, 110];

    // Colours: gray / orange / magenta / blue / gray  (section-kind palette)
    let colours: [Rgba<u8>; 5] = [
        Rgba([110, 110, 130, 255]), // Intro / Outro — gris
        Rgba([210, 100,  15, 255]), // Build-up — orange
        Rgba([180,  20, 120, 255]), // Peak — magenta
        Rgba([ 55, 105, 175, 255]), // Breakdown — bleu
        Rgba([110, 110, 130, 255]), // Intro / Outro — gris
    ];

    for i in 0..n as usize {
        let x0 = x_off + i as u32 * (bar_w + gap);
        let x1 = x0 + bar_w;
        let y1 = bottom;
        let y0 = bottom - heights[i];
        let col = colours[i];

        for y in y0..y1 {
            for x in x0..x1 {
                // Rounded top corners only
                let in_top_left = x < x0 + radius && y < y0 + radius;
                let in_top_right = x >= x1 - radius && y < y0 + radius;

                if in_top_left || in_top_right {
                    let cx = if in_top_left { x0 + radius } else { x1 - radius };
                    let cy = y0 + radius;
                    let dx = (x as i32 - cx as i32).unsigned_abs();
                    let dy = (y as i32 - cy as i32).unsigned_abs();
                    if dx * dx + dy * dy > radius * radius {
                        continue;
                    }
                }

                img.put_pixel(x, y, col);
            }
        }
    }

    // ── Outer rounded-square mask (icon shape) ────────────────────────────────
    let corner: u32 = 40;
    for y in 0..size {
        for x in 0..size {
            let in_tl = x < corner && y < corner;
            let in_tr = x >= size - corner && y < corner;
            let in_bl = x < corner && y >= size - corner;
            let in_br = x >= size - corner && y >= size - corner;

            if in_tl || in_tr || in_bl || in_br {
                let cx: u32 = if x < corner { corner } else { size - corner };
                let cy: u32 = if y < corner { corner } else { size - corner };
                let dx = (x as i32 - cx as i32).unsigned_abs();
                let dy = (y as i32 - cy as i32).unsigned_abs();
                if dx * dx + dy * dy > corner * corner {
                    img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                }
            }
        }
    }

    std::fs::create_dir_all("assets").unwrap();
    img.save("assets/icon.png").unwrap();
}
