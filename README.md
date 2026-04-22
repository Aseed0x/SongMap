# SongMap

**A desktop tool for structuring and visualizing song compositions.**

SongMap helps musicians and producers map out their tracks section by section — from intro to outro — with real-time BPM-based timing, an interactive timeline, and genre-aware templates. Whether you're writing a pop song or a techno set, SongMap gives you a clear visual blueprint before you touch a DAW.

---

## Features

- **Interactive timeline** — drag to resize or reorder sections, click to select and edit
- **13+ section types** — Intro, Verse, Pre-Chorus, Chorus, Hook, Bridge, Drop, Peak, BuildUp, Breakdown, Solo, Outro, Interlude, and Custom
- **Per-section notes** — jot down ideas, samples, or production notes directly on each section
- **BPM engine** — enter your BPM and get instant beat/bar duration tables (up to 64 bars)
- **22 genre templates** — Pop, Hip-Hop, Trap, Rock, Metal, EDM, House, Techno, Hard Techno, Acid Techno, Hardcore/Gabber, Hardstyle, Psytrance, Industrial Techno, DnB, Jazz, R&B/Soul, Lo-fi, Reggae, and more
- **10 artist templates** — Daft Punk, Aphex Twin, The Prodigy, deadmau5, Jeff Mills, Gesaffelstein, Kavinsky, Nina Kraviz, Marie Vaunt, Chemical Brothers
- **Genre-specific tips** — context-aware production advice for each genre
- **Save / Load** — projects saved as human-readable JSON

---

## Screenshots

> _Coming soon_

---

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) (stable, edition 2021)

### Build from source

```bash
git clone https://github.com/your-username/SongMap.git
cd SongMap
cargo build --release
```

The binary will be in `target/release/songmap`.

### Run

```bash
cargo run --release
```

---

## Usage

1. **Pick a genre or artist template** from the left panel — this sets a sensible BPM default and a starter section layout
2. **Adjust the BPM** — all bar durations recalculate live
3. **Build your structure** — add, rename, resize, and reorder sections using the timeline or the section list on the right
4. **Add notes** per section for samples, synth ideas, or anything else
5. **Save your project** as a `.json` file and reload it later

---

## Project Structure

```
src/
├── main.rs     # Entry point, window configuration
├── app.rs      # UI layout, interaction logic, timeline rendering
├── types.rs    # Section, SectionKind, project serialization
└── data.rs     # Genre and artist template definitions
build.rs        # Build script — generates the app icon
assets/
└── icon.png    # Generated equalizer-bar icon
```

---

## Tech Stack

| Library | Role |
|---|---|
| [egui](https://github.com/emilk/egui) + [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) | Immediate-mode GUI |
| [serde](https://serde.rs/) + serde_json | Project serialization (JSON) |
| [rfd](https://github.com/PolyMeilex/rfd) | Native file dialogs |
| [image](https://github.com/image-rs/image) | Icon generation at build time |

---

## License

MIT — do whatever you want, credit appreciated.
