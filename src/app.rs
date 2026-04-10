use egui::{Color32, Rect, RichText, Stroke, pos2, vec2};

use crate::data::{ARTISTS, GENRES};
use crate::types::{Section, SectionKind, SongProject};

// ─── Drag state for the timeline ─────────────────────────────────────────────

enum TimelineDrag {
    Resizing {
        idx:           usize,
        start_x:       f32,
        original_bars: u32,
        bar_px:        f32,
    },
    Reordering {
        idx:            usize,
        /// cumulative horizontal delta since drag start
        offset_x:       f32,
    },
}

// ─── Left-panel tab ──────────────────────────────────────────────────────────

#[derive(PartialEq)]
enum LeftTab {
    Genres,
    Artists,
}

// ─── App state ───────────────────────────────────────────────────────────────

pub struct SongMapApp {
    left_tab:         LeftTab,
    selected_genre:   usize,
    selected_artist:  usize,
    bpm:              f32,
    sections:         Vec<Section>,
    selected_section: Option<usize>,
    project_name:     String,

    // Right-panel "add" form
    add_kind: SectionKind,
    add_bars: u32,

    // Timeline drag/resize
    timeline_drag: Option<TimelineDrag>,

    // Feedback messages
    status_msg: String,
}

impl SongMapApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        let bpm      = GENRES[0].default_bpm as f32;
        let sections = GENRES[0].default_sections();
        Self {
            left_tab:         LeftTab::Genres,
            selected_genre:   0,
            selected_artist:  0,
            bpm,
            sections,
            selected_section: None,
            project_name:     "Mon projet".to_string(),
            add_kind:         SectionKind::Verse,
            add_bars:         16,
            timeline_drag:    None,
            status_msg:       String::new(),
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn total_bars(&self) -> u32 {
        self.sections.iter().map(|s| s.bars).sum()
    }

    fn total_secs(&self) -> f32 {
        self.sections.iter().map(|s| s.duration_secs(self.bpm)).sum()
    }

    fn beat_secs(&self) -> f32 { 60.0 / self.bpm }
    fn bar_secs (&self) -> f32 { self.beat_secs() * 4.0 }

    fn fmt_time(secs: f32) -> String {
        let m = (secs / 60.0) as u32;
        let s = (secs % 60.0) as u32;
        format!("{:02}:{:02}", m, s)
    }

    fn current_genre_name(&self) -> &str {
        if self.left_tab == LeftTab::Artists {
            ARTISTS[self.selected_artist].genre_hint
        } else {
            GENRES[self.selected_genre].name
        }
    }

    fn current_tips(&self) -> &'static [&'static str] {
        if self.left_tab == LeftTab::Artists {
            ARTISTS[self.selected_artist].tips
        } else {
            GENRES[self.selected_genre].tips
        }
    }

    // ── Save / Load ───────────────────────────────────────────────────────────

    fn save_project(&mut self) {
        let project = SongProject::from((
            self.project_name.as_str(),
            self.bpm,
            self.current_genre_name(),
            self.sections.as_slice(),
        ));
        let json = match serde_json::to_string_pretty(&project) {
            Ok(j) => j,
            Err(e) => { self.status_msg = format!("Erreur serialisation: {}", e); return; }
        };
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name(format!("{}.songmap.json", self.project_name))
            .add_filter("SongMap JSON", &["json"])
            .save_file()
        {
            match std::fs::write(&path, &json) {
                Ok(_)  => self.status_msg = format!("Sauvegarde : {}", path.display()),
                Err(e) => self.status_msg = format!("Erreur ecriture: {}", e),
            }
        }
    }

    fn load_project(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("SongMap JSON", &["json"])
            .pick_file()
        {
            let content = match std::fs::read_to_string(&path) {
                Ok(c)  => c,
                Err(e) => { self.status_msg = format!("Erreur lecture: {}", e); return; }
            };
            match serde_json::from_str::<SongProject>(&content) {
                Ok(proj) => {
                    self.project_name = proj.name.clone();
                    self.bpm          = proj.bpm;
                    self.sections     = proj.to_sections();
                    self.selected_section = None;
                    self.status_msg   = format!("Charge : {}", path.display());
                }
                Err(e) => { self.status_msg = format!("JSON invalide: {}", e); }
            }
        }
    }
}

// ─── eframe::App ─────────────────────────────────────────────────────────────

impl eframe::App for SongMapApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());

        // ── Bottom panel : tips (full width) ─────────────────────────────────
        egui::TopBottomPanel::bottom("tips_panel")
            .min_height(90.0)
            .max_height(140.0)
            .resizable(true)
            .show(ctx, |ui| self.panel_tips(ui));

        // ── Left panel : genres / artists + BPM ──────────────────────────────
        egui::SidePanel::left("left_panel")
            .min_width(200.0)
            .max_width(240.0)
            .show(ctx, |ui| self.panel_left(ui));

        // ── Right panel : section editor + add ───────────────────────────────
        egui::SidePanel::right("right_panel")
            .min_width(210.0)
            .max_width(250.0)
            .show(ctx, |ui| self.panel_right(ui));

        // ── Central panel : timeline + detail list ───────────────────────────
        egui::CentralPanel::default()
            .show(ctx, |ui| self.panel_central(ui));
    }
}

// ─── Bottom tips panel ───────────────────────────────────────────────────────

impl SongMapApp {
    fn panel_tips(&self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        let title = if self.left_tab == LeftTab::Artists {
            format!("Conseils — {}", ARTISTS[self.selected_artist].name)
        } else {
            format!("Conseils — {}", GENRES[self.selected_genre].name)
        };
        ui.label(RichText::new(title).strong());
        ui.add_space(4.0);

        let tips = self.current_tips();
        // Display tips in a grid to use full width
        let cols = if tips.len() > 3 { 2 } else { 1 };
        let per_col = (tips.len() + cols - 1) / cols;

        ui.horizontal(|ui| {
            for col in 0..cols {
                ui.vertical(|ui| {
                    for i in (col * per_col)..((col + 1) * per_col).min(tips.len()) {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("  >  ")
                                    .color(Color32::from_rgb(80, 150, 255))
                                    .strong(),
                            );
                            ui.label(RichText::new(tips[i]).color(Color32::from_gray(220)));
                        });
                    }
                });
                if col + 1 < cols {
                    ui.separator();
                }
            }
        });
    }
}

// ─── Left panel ──────────────────────────────────────────────────────────────

impl SongMapApp {
    fn panel_left(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.heading("SongMap");
        ui.separator();

        // Tab selector
        ui.horizontal(|ui| {
            if ui.selectable_label(self.left_tab == LeftTab::Genres,  "Genres").clicked() {
                self.left_tab = LeftTab::Genres;
            }
            if ui.selectable_label(self.left_tab == LeftTab::Artists, "Artistes").clicked() {
                self.left_tab = LeftTab::Artists;
            }
        });
        ui.add_space(4.0);

        egui::ScrollArea::vertical()
            .id_salt("left_scroll")
            .max_height(ui.available_height() - 200.0)
            .show(ui, |ui| {
                match self.left_tab {
                    LeftTab::Genres  => self.genre_list(ui),
                    LeftTab::Artists => self.artist_list(ui),
                }
            });

        ui.separator();
        self.bpm_section(ui);
    }

    fn genre_list(&mut self, ui: &mut egui::Ui) {
        for i in 0..GENRES.len() {
            let selected = i == self.selected_genre && self.left_tab == LeftTab::Genres;
            let text = RichText::new(GENRES[i].name).color(if selected {
                Color32::WHITE
            } else {
                Color32::from_gray(190)
            });
            let btn = egui::Button::new(text).fill(if selected {
                Color32::from_rgb(38, 68, 118)
            } else {
                Color32::TRANSPARENT
            });
            if ui.add_sized([ui.available_width(), 24.0], btn).clicked() {
                self.selected_genre  = i;
                self.left_tab        = LeftTab::Genres;
                self.bpm             = GENRES[i].default_bpm as f32;
                self.sections        = GENRES[i].default_sections();
                self.selected_section = None;
            }
        }
    }

    fn artist_list(&mut self, ui: &mut egui::Ui) {
        for i in 0..ARTISTS.len() {
            let selected = i == self.selected_artist && self.left_tab == LeftTab::Artists;
            let text = RichText::new(ARTISTS[i].name).color(if selected {
                Color32::WHITE
            } else {
                Color32::from_gray(190)
            });
            let sub = RichText::new(ARTISTS[i].genre_hint)
                .small()
                .color(Color32::from_gray(120));
            let btn = egui::Button::new(text).fill(if selected {
                Color32::from_rgb(60, 38, 100)
            } else {
                Color32::TRANSPARENT
            });
            ui.push_id(i + 1000, |ui| {
                if ui.add_sized([ui.available_width(), 24.0], btn).clicked() {
                    self.selected_artist  = i;
                    self.left_tab         = LeftTab::Artists;
                    self.bpm              = ARTISTS[i].default_bpm as f32;
                    self.sections         = ARTISTS[i].default_sections();
                    self.selected_section = None;
                }
                if selected {
                    ui.label(sub);
                }
            });
        }
    }

    fn bpm_section(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.label(RichText::new("BPM").strong());
        ui.add_space(2.0);

        let (lo, hi) = if self.left_tab == LeftTab::Artists {
            ARTISTS[self.selected_artist].bpm_range
        } else {
            GENRES[self.selected_genre].bpm_range
        };

        let mut bpm_int = self.bpm.round() as u32;
        ui.horizontal(|ui| {
            if ui.add(egui::DragValue::new(&mut bpm_int).range(lo..=hi).speed(1.0)).changed() {
                self.bpm = bpm_int as f32;
            }
            ui.label(RichText::new("BPM").color(Color32::from_gray(140)));
            if ui.small_button("R").on_hover_text("Revenir au BPM par defaut").clicked() {
                self.bpm = if self.left_tab == LeftTab::Artists {
                    ARTISTS[self.selected_artist].default_bpm as f32
                } else {
                    GENRES[self.selected_genre].default_bpm as f32
                };
            }
        });
        ui.add(egui::Slider::new(&mut self.bpm, lo as f32..=hi as f32).show_value(false));

        ui.add_space(8.0);
        ui.label(RichText::new("Conversion").strong());
        ui.add_space(2.0);

        let beat = self.beat_secs();
        let bar  = self.bar_secs();
        egui::Grid::new("conv")
            .num_columns(2)
            .striped(true)
            .spacing([8.0, 2.0])
            .show(ui, |ui| {
                ui.label("1 beat");
                ui.label(format!("{:.3} s", beat));
                ui.end_row();
                ui.label("1 mesure");
                ui.label(format!("{:.3} s", bar));
                ui.end_row();
                for n in [2u32, 4, 8, 16, 32, 64] {
                    ui.label(format!("{} mesures", n));
                    let d = bar * n as f32;
                    ui.label(if d >= 60.0 {
                        Self::fmt_time(d)
                    } else {
                        format!("{:.1} s", d)
                    });
                    ui.end_row();
                }
            });
    }
}

// ─── Right panel (section editor + add + save/load) ──────────────────────────

enum SectionOp { Select(usize), MoveUp(usize), MoveDown(usize), Remove(usize) }

impl SongMapApp {
    fn panel_right(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.heading("Sections");
        ui.separator();

        let n = self.sections.len();
        let mut op: Option<SectionOp> = None;

        egui::ScrollArea::vertical()
            .id_salt("sec_list")
            .max_height(ui.available_height() - 230.0)
            .show(ui, |ui| {
                for i in 0..n {
                    let selected = self.selected_section == Some(i);
                    let color    = self.sections[i].color();
                    let name     = self.sections[i].name.clone();
                    let bars     = self.sections[i].bars;

                    ui.push_id(i, |ui| {
                        ui.horizontal(|ui| {
                            let (r, _) = ui.allocate_exact_size(vec2(5.0, 20.0), egui::Sense::hover());
                            ui.painter().rect_filled(r, 2.0, color);

                            let text = RichText::new(&name).color(if selected { Color32::WHITE } else { Color32::from_gray(200) });
                            let btn  = egui::Button::new(text).fill(if selected { Color32::from_rgb(38, 58, 98) } else { Color32::TRANSPARENT });
                            if ui.add_sized([72.0, 20.0], btn).clicked() {
                                op = Some(SectionOp::Select(i));
                            }
                            ui.label(RichText::new(format!("{}B", bars)).small().color(Color32::from_gray(120)));

                            if i > 0     && ui.small_button("^").clicked() { op = Some(SectionOp::MoveUp(i));   }
                            if i + 1 < n && ui.small_button("v").clicked() { op = Some(SectionOp::MoveDown(i)); }
                            if ui.small_button("X").on_hover_text("Supprimer").clicked() { op = Some(SectionOp::Remove(i)); }
                        });
                    });
                }
            });

        // Apply list operation
        if let Some(op) = op {
            match op {
                SectionOp::Select(i) => {
                    self.selected_section = if self.selected_section == Some(i) { None } else { Some(i) };
                }
                SectionOp::MoveUp(i) => {
                    self.sections.swap(i - 1, i);
                    if self.selected_section == Some(i) { self.selected_section = Some(i - 1); }
                }
                SectionOp::MoveDown(i) => {
                    self.sections.swap(i, i + 1);
                    if self.selected_section == Some(i) { self.selected_section = Some(i + 1); }
                }
                SectionOp::Remove(i) => {
                    self.sections.remove(i);
                    if self.selected_section == Some(i) { self.selected_section = None; }
                }
            }
        }

        // Edit selected section
        if let Some(idx) = self.selected_section {
            if idx < self.sections.len() {
                ui.separator();
                ui.label(RichText::new("Modifier").strong());

                ui.horizontal(|ui| {
                    ui.label("Nom :");
                    ui.text_edit_singleline(&mut self.sections[idx].name);
                });
                ui.horizontal(|ui| {
                    ui.label("Mesures :");
                    ui.add(egui::DragValue::new(&mut self.sections[idx].bars).range(1..=256));
                });
                ui.horizontal(|ui| {
                    ui.label("Type :");
                    egui::ComboBox::from_id_salt("kind_edit")
                        .selected_text(self.sections[idx].kind.label())
                        .show_ui(ui, |ui| {
                            for k in SectionKind::all() {
                                if ui.selectable_label(self.sections[idx].kind == *k, k.label()).clicked() {
                                    self.sections[idx].kind = k.clone();
                                }
                            }
                        });
                });
                let dur = self.sections[idx].duration_secs(self.bpm);
                ui.label(RichText::new(format!("{:.1}s  ({})", dur, Self::fmt_time(dur))).small().color(Color32::from_gray(140)));
            }
        }

        ui.separator();
        ui.label(RichText::new("Ajouter").strong());

        egui::ComboBox::from_id_salt("add_kind")
            .selected_text(self.add_kind.label())
            .show_ui(ui, |ui| {
                for k in SectionKind::all() {
                    if ui.selectable_label(self.add_kind == *k, k.label()).clicked() {
                        self.add_kind = k.clone();
                    }
                }
            });
        ui.horizontal(|ui| {
            ui.label("Mesures :");
            ui.add(egui::DragValue::new(&mut self.add_bars).range(1..=256));
        });
        if ui.button("  +  Ajouter  ").clicked() {
            self.sections.push(Section::new(self.add_kind.clone(), self.add_bars));
        }

        ui.add_space(4.0);
        if ui.button("Reinitialiser").on_hover_text("Revenir au pattern par defaut").clicked() {
            self.sections = if self.left_tab == LeftTab::Artists {
                ARTISTS[self.selected_artist].default_sections()
            } else {
                GENRES[self.selected_genre].default_sections()
            };
            self.selected_section = None;
        }

        ui.separator();
        ui.label(RichText::new("Projet").strong());

        ui.horizontal(|ui| {
            ui.label("Nom :");
            ui.text_edit_singleline(&mut self.project_name);
        });
        ui.horizontal(|ui| {
            if ui.button("Sauvegarder").clicked() { self.save_project(); }
            if ui.button("Importer").clicked()    { self.load_project(); }
        });

        if !self.status_msg.is_empty() {
            ui.add_space(4.0);
            ui.label(RichText::new(&self.status_msg).small().color(Color32::from_rgb(120, 200, 120)));
        }
    }
}

// ─── Central panel ───────────────────────────────────────────────────────────

impl SongMapApp {
    fn panel_central(&mut self, ui: &mut egui::Ui) {
        // Header
        let label = if self.left_tab == LeftTab::Artists {
            format!("{} — {}  |  {} BPM",
                ARTISTS[self.selected_artist].name,
                ARTISTS[self.selected_artist].genre_hint,
                self.bpm as u32)
        } else {
            format!("{}  |  {} BPM", GENRES[self.selected_genre].name, self.bpm as u32)
        };

        ui.horizontal(|ui| {
            ui.heading(&label);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!(
                    "{} mesures  |  {}",
                    self.total_bars(),
                    Self::fmt_time(self.total_secs())
                )).color(Color32::from_gray(160)));
            });
        });

        ui.separator();

        // ── Interactive timeline ──────────────────────────────────────────────
        self.draw_timeline(ui);

        ui.add_space(8.0);

        // ── Detail list ───────────────────────────────────────────────────────
        ui.label(RichText::new("Structure detaillee").strong());
        ui.add_space(4.0);

        let bar_secs       = self.bar_secs();
        let mut cumul_bars = 0u32;

        egui::ScrollArea::vertical().id_salt("detail").show(ui, |ui| {
            for i in 0..self.sections.len() {
                let dur        = self.sections[i].duration_secs(self.bpm);
                let color      = self.sections[i].color();
                let start_time = cumul_bars as f32 * bar_secs;
                cumul_bars    += self.sections[i].bars;
                let selected   = self.selected_section == Some(i);

                let bg = if selected {
                    Color32::from_rgb(32, 48, 78)
                } else {
                    Color32::from_rgb(18, 18, 24)
                };

                egui::Frame::none()
                    .fill(bg)
                    .stroke(Stroke::new(1.0, Color32::from_rgb(40, 40, 55)))
                    .inner_margin(egui::Margin::symmetric(6.0, 4.0))
                    .outer_margin(egui::Margin::symmetric(0.0, 1.0))
                    .rounding(egui::Rounding::same(4.0))
                    .show(ui, |ui| {
                        // ── Header row ──────────────────────────────────
                        ui.horizontal(|ui| {
                            let (r, _) = ui.allocate_exact_size(vec2(11.0, 11.0), egui::Sense::hover());
                            ui.painter().rect_filled(r, 3.0, color);
                            ui.label(RichText::new(format!("{:2}.", i + 1)).color(Color32::from_gray(110)));
                            ui.label(RichText::new(&self.sections[i].name).strong().color(Color32::WHITE));
                            ui.label(RichText::new(format!("— {} mesures", self.sections[i].bars)).color(Color32::from_gray(150)));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(
                                    RichText::new(format!("{:.0}s  [{}]", dur, Self::fmt_time(start_time)))
                                        .small()
                                        .color(Color32::from_gray(110)),
                                );
                            });
                        });

                        // ── Notes row ────────────────────────────────────
                        ui.add_space(2.0);
                        let note_hint = "Notes, samples, idees...";
                        let note_empty = self.sections[i].notes.is_empty();
                        let te = egui::TextEdit::multiline(&mut self.sections[i].notes)
                            .hint_text(note_hint)
                            .desired_rows(if note_empty && !selected { 1 } else { 2 })
                            .desired_width(ui.available_width())
                            .frame(false)
                            .text_color(Color32::from_gray(200))
                            .font(egui::FontId::proportional(12.0));
                        ui.add(te);
                    });
            }
        });
    }

    // ── Timeline drawing + interaction ────────────────────────────────────────

    fn draw_timeline(&mut self, ui: &mut egui::Ui) {
        const TL_HEIGHT: f32 = 76.0;
        const RESIZE_ZONE: f32 = 10.0;

        let available_width = ui.available_width();
        let total_bars      = self.total_bars().max(1) as f32;

        // Allocate the full timeline rect with drag sense
        let (resp, painter) = ui.allocate_painter(
            vec2(available_width, TL_HEIGHT),
            egui::Sense::click_and_drag(),
        );
        let rect   = resp.rect;
        let bar_px = rect.width() / total_bars;

        // ── Compute section rects ─────────────────────────────────────────────
        let section_rects: Vec<Rect> = {
            let mut x = rect.left();
            self.sections.iter().map(|s| {
                let w = (bar_px * s.bars as f32 - 1.0).max(2.0);
                let r = Rect::from_min_size(pos2(x, rect.top()), vec2(w, TL_HEIGHT));
                x += bar_px * s.bars as f32;
                r
            }).collect()
        };

        // ── Cursor hint ───────────────────────────────────────────────────────
        if let Some(hover) = resp.hover_pos() {
            for sr in &section_rects {
                if sr.contains(hover) && hover.x >= sr.right() - RESIZE_ZONE {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                    break;
                }
            }
        }

        // ── Drag start ────────────────────────────────────────────────────────
        if resp.drag_started() {
            if let Some(pos) = resp.interact_pointer_pos() {
                for (i, sr) in section_rects.iter().enumerate() {
                    if !sr.contains(pos) { continue; }
                    if pos.x >= sr.right() - RESIZE_ZONE {
                        self.timeline_drag = Some(TimelineDrag::Resizing {
                            idx:           i,
                            start_x:       pos.x,
                            original_bars: self.sections[i].bars,
                            bar_px,
                        });
                    } else {
                        self.selected_section = Some(i);
                        self.timeline_drag    = Some(TimelineDrag::Reordering {
                            idx: i, offset_x: 0.0,
                        });
                    }
                    break;
                }
            }
        }

        // ── Drag update ───────────────────────────────────────────────────────
        if resp.dragged() {
            let delta = resp.drag_delta().x;
            match &mut self.timeline_drag {
                Some(TimelineDrag::Resizing { idx, start_x, original_bars, bar_px: bp }) => {
                    let cur_x      = resp.interact_pointer_pos().map(|p| p.x).unwrap_or(*start_x);
                    let delta_bars = ((cur_x - *start_x) / *bp).round() as i32;
                    let new_bars   = (*original_bars as i32 + delta_bars).max(1) as u32;
                    let idx        = *idx;
                    self.sections[idx].bars = new_bars;
                }
                Some(TimelineDrag::Reordering { idx, offset_x }) => {
                    *offset_x += delta;
                    // Live swap: move section if it crosses neighbour midpoint
                    let off    = *offset_x;
                    let cur    = *idx;
                    let n      = self.sections.len();
                    if off < 0.0 && cur > 0 {
                        let left_w = bar_px * self.sections[cur - 1].bars as f32;
                        if off.abs() > left_w / 2.0 {
                            self.sections.swap(cur - 1, cur);
                            *idx      = cur - 1;
                            *offset_x = off + left_w;
                            self.selected_section = Some(cur - 1);
                        }
                    } else if off > 0.0 && cur + 1 < n {
                        let right_w = bar_px * self.sections[cur + 1].bars as f32;
                        if off > right_w / 2.0 {
                            self.sections.swap(cur, cur + 1);
                            *idx      = cur + 1;
                            *offset_x = off - right_w;
                            self.selected_section = Some(cur + 1);
                        }
                    }
                }
                None => {}
            }
        }

        // ── Drag end ──────────────────────────────────────────────────────────
        if resp.drag_stopped() {
            self.timeline_drag = None;
        }

        // ── Click (no drag) ───────────────────────────────────────────────────
        if resp.clicked() {
            if let Some(pos) = resp.interact_pointer_pos() {
                for (i, sr) in section_rects.iter().enumerate() {
                    if sr.contains(pos) {
                        self.selected_section =
                            if self.selected_section == Some(i) { None } else { Some(i) };
                        break;
                    }
                }
            }
        }

        // ── Draw ─────────────────────────────────────────────────────────────
        painter.rect_filled(rect, 6.0, Color32::from_rgb(20, 20, 30));

        for (i, (section, &sr)) in self.sections.iter().zip(section_rects.iter()).enumerate() {
            let selected = self.selected_section == Some(i);
            let c        = section.color();
            let fill     = if selected {
                Color32::from_rgb(c.r().saturating_add(40), c.g().saturating_add(40), c.b().saturating_add(40))
            } else {
                c
            };

            painter.rect_filled(sr, 4.0, fill);

            if selected {
                painter.rect_stroke(sr, 4.0, Stroke::new(2.0, Color32::WHITE));
            }

            // Resize handle
            let handle_x = sr.right() - RESIZE_ZONE / 2.0;
            painter.line_segment(
                [pos2(handle_x, sr.top() + 8.0), pos2(handle_x, sr.bottom() - 8.0)],
                Stroke::new(1.5, Color32::from_rgba_premultiplied(255, 255, 255, 60)),
            );

            // Label
            let w = sr.width();
            if w >= 18.0 {
                let (text, fsize) = if w >= 60.0 {
                    (format!("{}\n{}B", section.name, section.bars), 11.0)
                } else if w >= 28.0 {
                    (format!("{}B", section.bars), 10.0)
                } else {
                    (String::new(), 0.0)
                };
                if fsize > 0.0 {
                    painter.text(
                        sr.center(),
                        egui::Align2::CENTER_CENTER,
                        &text,
                        egui::FontId::proportional(fsize),
                        Color32::WHITE,
                    );
                }
            }

            // Delete button on selected blocks
            if selected {
                let btn_size = 14.0;
                let btn_rect = Rect::from_min_size(
                    pos2(sr.right() - btn_size - 2.0, sr.top() + 2.0),
                    vec2(btn_size, btn_size),
                );
                painter.rect_filled(btn_rect, 3.0, Color32::from_rgb(180, 40, 40));
                painter.text(
                    btn_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "x",
                    egui::FontId::proportional(10.0),
                    Color32::WHITE,
                );
                // Detect click on delete button
                if resp.clicked() {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        if btn_rect.contains(pos) {
                            self.sections.remove(i);
                            self.selected_section = None;
                            // Section list changed — stop drawing to avoid index OOB
                            return;
                        }
                    }
                }
            }
        }
    }
}
