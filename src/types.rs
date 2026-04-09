use egui::Color32;
use serde::{Deserialize, Serialize};

// ─── Section kinds ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SectionKind {
    Intro,
    Verse,
    PreChorus,
    Chorus,
    Hook,
    Bridge,
    Drop,
    BuildUp,
    Breakdown,
    Solo,
    Outro,
    Interlude,
    Custom,
}

impl SectionKind {
    pub fn color(&self) -> Color32 {
        match self {
            SectionKind::Intro =>     Color32::from_rgb(100, 100, 120),
            SectionKind::Verse =>     Color32::from_rgb(55,  105, 175),
            SectionKind::PreChorus => Color32::from_rgb(190, 130,  20),
            SectionKind::Chorus =>    Color32::from_rgb(195,  45,  45),
            SectionKind::Hook =>      Color32::from_rgb(170,  35,  85),
            SectionKind::Bridge =>    Color32::from_rgb(45,  140,  65),
            SectionKind::Drop =>      Color32::from_rgb(130,  40, 195),
            SectionKind::BuildUp =>   Color32::from_rgb(210, 100,  15),
            SectionKind::Breakdown => Color32::from_rgb(35,   85, 145),
            SectionKind::Solo =>      Color32::from_rgb(0,   180, 185),
            SectionKind::Outro =>     Color32::from_rgb(75,   75,  95),
            SectionKind::Interlude => Color32::from_rgb(85,  145, 145),
            SectionKind::Custom =>    Color32::from_rgb(130, 130, 130),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SectionKind::Intro =>     "Intro",
            SectionKind::Verse =>     "Verse",
            SectionKind::PreChorus => "Pre-Chorus",
            SectionKind::Chorus =>    "Chorus",
            SectionKind::Hook =>      "Hook",
            SectionKind::Bridge =>    "Bridge",
            SectionKind::Drop =>      "Drop",
            SectionKind::BuildUp =>   "Build-up",
            SectionKind::Breakdown => "Breakdown",
            SectionKind::Solo =>      "Solo",
            SectionKind::Outro =>     "Outro",
            SectionKind::Interlude => "Interlude",
            SectionKind::Custom =>    "Custom",
        }
    }

    /// Key used in JSON import/export
    pub fn to_key(&self) -> &'static str {
        match self {
            SectionKind::Intro =>     "Intro",
            SectionKind::Verse =>     "Verse",
            SectionKind::PreChorus => "PreChorus",
            SectionKind::Chorus =>    "Chorus",
            SectionKind::Hook =>      "Hook",
            SectionKind::Bridge =>    "Bridge",
            SectionKind::Drop =>      "Drop",
            SectionKind::BuildUp =>   "BuildUp",
            SectionKind::Breakdown => "Breakdown",
            SectionKind::Solo =>      "Solo",
            SectionKind::Outro =>     "Outro",
            SectionKind::Interlude => "Interlude",
            SectionKind::Custom =>    "Custom",
        }
    }

    pub fn from_key(s: &str) -> SectionKind {
        match s {
            "Intro"                    => SectionKind::Intro,
            "Verse"                    => SectionKind::Verse,
            "PreChorus" | "Pre-Chorus" => SectionKind::PreChorus,
            "Chorus"                   => SectionKind::Chorus,
            "Hook"                     => SectionKind::Hook,
            "Bridge"                   => SectionKind::Bridge,
            "Drop"                     => SectionKind::Drop,
            "BuildUp"   | "Build-up"   => SectionKind::BuildUp,
            "Breakdown"                => SectionKind::Breakdown,
            "Solo"                     => SectionKind::Solo,
            "Outro"                    => SectionKind::Outro,
            "Interlude"                => SectionKind::Interlude,
            _                          => SectionKind::Custom,
        }
    }

    pub fn all() -> &'static [SectionKind] {
        &[
            SectionKind::Intro,
            SectionKind::Verse,
            SectionKind::PreChorus,
            SectionKind::Chorus,
            SectionKind::Hook,
            SectionKind::Bridge,
            SectionKind::Drop,
            SectionKind::BuildUp,
            SectionKind::Breakdown,
            SectionKind::Solo,
            SectionKind::Outro,
            SectionKind::Interlude,
            SectionKind::Custom,
        ]
    }
}

// ─── Section ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Section {
    pub name: String,
    pub bars: u32,
    pub kind: SectionKind,
}

impl Section {
    pub fn new(kind: SectionKind, bars: u32) -> Self {
        let name = kind.label().to_string();
        Self { name, bars, kind }
    }

    pub fn color(&self) -> Color32 {
        self.kind.color()
    }

    pub fn duration_secs(&self, bpm: f32) -> f32 {
        self.bars as f32 * 4.0 * 60.0 / bpm
    }
}

// ─── Serialisation (JSON import / export) ────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SongProject {
    pub name: String,
    pub bpm:  f32,
    #[serde(default)]
    pub genre: String,
    pub sections: Vec<SectionSave>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SectionSave {
    pub name: String,
    pub kind: String,
    pub bars: u32,
}

impl SongProject {
    pub fn to_sections(&self) -> Vec<Section> {
        self.sections
            .iter()
            .map(|s| {
                let kind = SectionKind::from_key(&s.kind);
                Section { name: s.name.clone(), bars: s.bars, kind }
            })
            .collect()
    }
}

impl From<(&str, f32, &str, &[Section])> for SongProject {
    fn from((name, bpm, genre, sections): (&str, f32, &str, &[Section])) -> Self {
        SongProject {
            name:  name.to_string(),
            bpm,
            genre: genre.to_string(),
            sections: sections
                .iter()
                .map(|s| SectionSave {
                    name: s.name.clone(),
                    kind: s.kind.to_key().to_string(),
                    bars: s.bars,
                })
                .collect(),
        }
    }
}
