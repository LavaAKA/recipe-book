#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod recipe;

use eframe::egui::{self, Align, Color32, CornerRadius, FontFamily, FontId, Layout, Margin, RichText, Sense, Stroke, Vec2};
use recipe::{display_unit, scaled_amount, Ingredient, Recipe};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

// ------------------------------------------------------------------ theme

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct Theme {
    name: &'static str,
    dark: bool,
    bg: Color32,
    sidebar: Color32,
    card: Color32,
    border: Color32,
    text: Color32,
    muted: Color32,
    accent: Color32,
    /// Accent used for text (darker in light themes, lighter in dark ones).
    accent_dark: Color32,
    accent_soft: Color32,
    hover: Color32,
    on_accent: Color32,
    done: Color32,
    danger: Color32,
}

const fn c(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

#[allow(clippy::too_many_arguments)]
const fn theme(
    name: &'static str, dark: bool, bg: u32, sidebar: u32, card: u32, border: u32, text: u32, muted: u32,
    accent: u32, accent_dark: u32, accent_soft: u32, hover: u32, on_accent: u32, done: u32, danger: u32,
) -> Theme {
    Theme {
        name, dark, bg: c(bg), sidebar: c(sidebar), card: c(card), border: c(border), text: c(text),
        muted: c(muted), accent: c(accent), accent_dark: c(accent_dark), accent_soft: c(accent_soft),
        hover: c(hover), on_accent: c(on_accent), done: c(done), danger: c(danger),
    }
}

/// First five are light, last five are dark.
const THEMES: [Theme; 10] = [
    //     name        dark   bg        sidebar   card      border    text      muted     accent    acc_dark  acc_soft  hover     on_acc    done      danger
    theme("Cream",     false, 0xFAF7F2, 0xF2ECE2, 0xFFFFFF, 0xE7DFD3, 0x2B2623, 0x8C8176, 0xC4503A, 0xA63E2A, 0xF8E3DB, 0xEAE2D5, 0xFFFFFF, 0x6F9A78, 0xB43A3A),
    theme("Sage",      false, 0xF4F7F2, 0xE8EFE4, 0xFFFFFF, 0xD9E3D3, 0x22301F, 0x6F8068, 0x3F7D4E, 0x2F6A3E, 0xDDEFD9, 0xDCE6D6, 0xFFFFFF, 0x3F7D4E, 0xB43A3A),
    theme("Ocean",     false, 0xF3F7FB, 0xE5EDF5, 0xFFFFFF, 0xD6E1EC, 0x1D2A38, 0x6A7B8D, 0x2F6FB5, 0x1F5594, 0xDCEAF8, 0xD9E4F0, 0xFFFFFF, 0x3C9A74, 0xB43A3A),
    theme("Lavender",  false, 0xF8F5FC, 0xEEE8F6, 0xFFFFFF, 0xE2D9EF, 0x2A2433, 0x80748F, 0x7C4DCC, 0x5F37A8, 0xEBE0FA, 0xE6DEF1, 0xFFFFFF, 0x5E9E78, 0xB43A3A),
    theme("Honey",     false, 0xFFFBF0, 0xF7EFD6, 0xFFFFFF, 0xEEE3BF, 0x2F2A1A, 0x8A8062, 0xB86E00, 0x8F5400, 0xFCEBC4, 0xF1E6C4, 0xFFFFFF, 0x6F9A5A, 0xB43A3A),
    theme("Midnight",  true,  0x14171C, 0x0F1216, 0x1D2128, 0x2C323B, 0xE6E9EE, 0x8A94A3, 0x5B9DF0, 0x8DBBF7, 0x22344D, 0x232A33, 0x0B1626, 0x4CAF7A, 0xF07178),
    theme("Espresso",  true,  0x1A1613, 0x14110F, 0x241E1A, 0x3A3029, 0xF0E6DC, 0x9C8D7F, 0xE07A5F, 0xF0A58F, 0x4A2A22, 0x2E2621, 0x1A0F0B, 0x6FBF8A, 0xF07178),
    theme("Forest",    true,  0x111A15, 0x0C130F, 0x18241D, 0x27382E, 0xE3EEE6, 0x83988A, 0x5FBF84, 0x8EDBAA, 0x1F4030, 0x1F2E25, 0x08160D, 0x5FBF84, 0xF07178),
    theme("Plum",      true,  0x18121C, 0x120D16, 0x211826, 0x35283C, 0xEFE6F3, 0x9887A3, 0xB57BEE, 0xD1A8F6, 0x3A2650, 0x2A2030, 0x17091F, 0x6FBF8A, 0xF07178),
    theme("Charcoal",  true,  0x18181A, 0x111113, 0x222225, 0x333337, 0xECECEE, 0x8E8E96, 0xF0A030, 0xF6C06A, 0x3F3015, 0x2B2B2F, 0x1C1303, 0x6FBF8A, 0xF07178),
];

static THEME: AtomicUsize = AtomicUsize::new(0);

fn t() -> &'static Theme {
    &THEMES[THEME.load(Ordering::Relaxed).min(THEMES.len() - 1)]
}

fn theme_file() -> PathBuf {
    data_path().with_file_name("theme.txt")
}

fn load_theme() {
    if let Ok(name) = std::fs::read_to_string(theme_file()) {
        if let Some(i) = THEMES.iter().position(|th| th.name.eq_ignore_ascii_case(name.trim())) {
            THEME.store(i, Ordering::Relaxed);
        }
    }
}

fn set_theme(ctx: &egui::Context, i: usize) {
    THEME.store(i, Ordering::Relaxed);
    let _ = std::fs::write(theme_file(), THEMES[i].name);
    apply_theme(ctx);
}

const CONTENT_WIDTH: f32 = 880.0;

fn serif(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("serif".into()))
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "lora".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../assets/Lora.ttf"))),
    );
    // Lora first, then the default fonts for any glyph Lora lacks (arrows, symbols).
    let mut list = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    list.insert(0, "lora".to_owned());
    fonts.families.insert(FontFamily::Name("serif".into()), list);
    ctx.set_fonts(fonts);
}

fn apply_theme(ctx: &egui::Context) {
    let mut v = if t().dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    v.panel_fill = t().bg;
    v.window_fill = t().bg;
    v.extreme_bg_color = t().card; // text edit background
    v.faint_bg_color = t().sidebar;
    v.window_corner_radius = CornerRadius::same(6);
    v.window_stroke = Stroke::new(1.0, t().border);
    v.window_shadow = egui::Shadow::NONE;
    v.popup_shadow = egui::Shadow::NONE;
    v.selection.bg_fill = t().accent_soft;
    v.selection.stroke = Stroke::new(1.0, t().accent);
    v.hyperlink_color = t().accent_dark;

    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, t().border);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, t().text);
    for w in [&mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        w.corner_radius = CornerRadius::same(4);
    }
    // Flat buttons: hairline outline, fill only on hover.
    v.widgets.inactive.bg_fill = t().card;
    v.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, t().border);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, t().text);
    v.widgets.hovered.bg_fill = t().hover;
    v.widgets.hovered.weak_bg_fill = t().hover;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, t().muted);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, t().text);
    v.widgets.active.bg_fill = t().hover;
    v.widgets.active.weak_bg_fill = t().hover;
    v.widgets.active.bg_stroke = Stroke::new(1.0, t().text);
    v.widgets.active.fg_stroke = Stroke::new(1.0, t().text);
    ctx.set_visuals(v);

    ctx.global_style_mut(|s| {
        s.spacing.item_spacing = Vec2::new(8.0, 6.0);
        s.spacing.button_padding = Vec2::new(10.0, 4.0);
        s.spacing.interact_size.y = 26.0;
        s.text_styles.insert(egui::TextStyle::Body, FontId::proportional(15.0));
        s.text_styles.insert(egui::TextStyle::Button, FontId::proportional(14.0));
        s.text_styles.insert(egui::TextStyle::Small, FontId::proportional(12.5));
        s.text_styles.insert(egui::TextStyle::Heading, serif(28.0));
    });
}

// ------------------------------------------------------------------ small widgets

/// Section heading with a hairline under it.
fn heading(ui: &mut egui::Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(text).font(serif(22.0)).color(t().text));
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 1.0), Sense::hover());
    ui.painter().hline(rect.x_range(), rect.center().y, Stroke::new(1.0, t().border));
    ui.add_space(6.0);
}

fn small_label(ui: &mut egui::Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(text).size(13.0).color(t().muted));
}

/// A text link: plain text, underlined on hover.
fn link(ui: &mut egui::Ui, text: &str, color: Color32) -> egui::Response {
    let resp = ui.add(egui::Label::new(RichText::new(text).size(14.5).color(color)).sense(Sense::click()));
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        ui.painter().hline(resp.rect.x_range(), resp.rect.bottom() - 2.0, Stroke::new(1.0, color));
    }
    resp
}

/// A page tab: serif text with an underline when active.
fn tab(ui: &mut egui::Ui, text: &str, active: bool) -> egui::Response {
    let color = if active { t().text } else { t().muted };
    let resp = ui.add(egui::Label::new(RichText::new(text).font(serif(18.0)).color(color)).sense(Sense::click()));
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if active {
        ui.painter().hline(resp.rect.x_range(), resp.rect.bottom() + 1.0, Stroke::new(2.0, t().accent));
    }
    resp
}

fn primary_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.to_string()).color(t().on_accent))
        .fill(t().accent)
        .stroke(Stroke::NONE)
        .corner_radius(4)
}

fn small_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.to_string()).size(14.0))
}

/// Small tag button: outlined when off, filled when on.
fn chip(ui: &mut egui::Ui, text: &str, selected: bool) -> egui::Response {
    ui.scope(|ui| {
        ui.spacing_mut().button_padding = Vec2::new(8.0, 1.0);
        ui.spacing_mut().interact_size.y = 22.0;
        let (fill, fg, stroke) = if selected { (t().accent_soft, t().accent_dark, t().accent) } else { (Color32::TRANSPARENT, t().muted, t().border) };
        ui.add(egui::Button::new(RichText::new(text).size(13.0).color(fg)).fill(fill).stroke(Stroke::new(1.0, stroke)))
    })
    .inner
}

// ------------------------------------------------------------------ data

fn data_path() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("XDG_DATA_HOME"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("RecipeBook").join("recipes.json")
}

fn load_recipes() -> Vec<Recipe> {
    match std::fs::read_to_string(data_path()) {
        Ok(text) => recipe::parse_json(text.trim_start_matches('\u{feff}')).unwrap_or_else(|_| {
            // Unreadable file: keep a copy so the next save can't destroy it.
            let _ = std::fs::copy(data_path(), data_path().with_extension("json.bak"));
            Vec::new()
        }),
        Err(_) => vec![recipe::sample()],
    }
}

struct ImportDialog {
    text: String,
    error: Option<String>,
}

struct RecipeApp {
    recipes: Vec<Recipe>,
    selected: usize,
    editing: bool,
    /// Servings the user wants to cook for (view mode only, not saved).
    target: u32,
    done_ingredients: HashSet<usize>,
    done_steps: HashSet<usize>,
    filter: String,
    /// Sidebar tag filter.
    active_tag: Option<String>,
    /// Text box for adding a custom tag in the editor.
    new_tag: String,
    import: Option<ImportDialog>,
    confirm_delete: bool,
    status: String,
    dirty: bool,
}

impl RecipeApp {
    fn new() -> Self {
        let recipes = load_recipes();
        let target = recipes.first().map(|r| r.servings).unwrap_or(4);
        Self {
            recipes,
            selected: 0,
            editing: false,
            target,
            done_ingredients: HashSet::new(),
            done_steps: HashSet::new(),
            filter: String::new(),
            active_tag: None,
            new_tag: String::new(),
            import: None,
            confirm_delete: false,
            status: String::new(),
            dirty: false,
        }
    }

    fn select(&mut self, i: usize) {
        self.selected = i;
        self.target = self.recipes[i].servings;
        self.done_ingredients.clear();
        self.done_steps.clear();
        self.editing = false;
        self.confirm_delete = false;
    }

    fn add_recipe(&mut self, r: Recipe) {
        self.recipes.push(r);
        self.select(self.recipes.len() - 1);
        self.dirty = true;
    }

    fn save(&mut self) {
        let path = data_path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let result = serde_json::to_string_pretty(&self.recipes)
            .map_err(|e| e.to_string())
            .and_then(|json| std::fs::write(&path, json).map_err(|e| e.to_string()));
        if let Err(e) = result {
            self.status = format!("Couldn't save: {e}");
        }
        self.dirty = false;
    }

    fn import_file(&mut self) {
        let Some(path) = rfd::FileDialog::new().add_filter("Recipes", &["json", "txt", "md"]).pick_file() else {
            return;
        };
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                let text = text.trim_start_matches('\u{feff}').to_string();
                let is_json = path.extension().is_some_and(|e| e.eq_ignore_ascii_case("json"));
                let result = if is_json { recipe::parse_json(&text) } else { recipe::parse_text(&text).map(|r| vec![r]) };
                match result {
                    Ok(list) => {
                        let n = list.len();
                        for r in list {
                            self.add_recipe(r);
                        }
                        self.status = format!("Imported {n} recipe{}.", if n == 1 { "" } else { "s" });
                        self.import = None;
                    }
                    Err(e) => self.set_import_error(e),
                }
            }
            Err(e) => self.set_import_error(format!("Couldn't read file: {e}")),
        }
    }

    fn set_import_error(&mut self, e: String) {
        match &mut self.import {
            Some(d) => d.error = Some(e),
            None => self.status = e,
        }
    }

    fn export_current(&mut self) {
        let Some(r) = self.recipes.get(self.selected) else { return };
        let default = format!("{}.json", r.name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-"));
        if let Some(path) = rfd::FileDialog::new().set_file_name(default).add_filter("Recipe JSON", &["json"]).save_file() {
            let res = serde_json::to_string_pretty(r)
                .map_err(|e| e.to_string())
                .and_then(|j| std::fs::write(&path, j).map_err(|e| e.to_string()));
            self.status = match res {
                Ok(()) => format!("Exported to {}", path.display()),
                Err(e) => format!("Export failed: {e}"),
            };
        }
    }

    // ------------------------------------------------------------ sidebar

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        ui.label(RichText::new("Recipe Book").font(serif(26.0)).color(t().text));
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            let w = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
            if ui.add_sized([w, 30.0], egui::Button::new(RichText::new("Import").color(t().accent_dark))).clicked() {
                self.import = Some(ImportDialog { text: String::new(), error: None });
            }
            if ui.add_sized([w, 30.0], egui::Button::new("New")).clicked() {
                self.add_recipe(Recipe::blank());
                self.editing = true;
            }
        });
        ui.add_space(2.0);
        ui.add(egui::TextEdit::singleline(&mut self.filter).hint_text("Search").desired_width(f32::INFINITY).margin(Margin::symmetric(8, 6)));
        ui.add_space(6.0);

        // Tag filter (only tags that exist on some recipe), shown as plain links.
        let mut all_tags: Vec<String> = Vec::new();
        for r in &self.recipes {
            for tag in &r.tags {
                if !all_tags.iter().any(|x| x.eq_ignore_ascii_case(tag)) {
                    all_tags.push(tag.clone());
                }
            }
        }
        all_tags.sort_by_key(|s| s.to_lowercase());
        if self.active_tag.as_ref().is_some_and(|a| !all_tags.iter().any(|x| x.eq_ignore_ascii_case(a))) {
            self.active_tag = None;
        }
        if !all_tags.is_empty() {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(10.0, 2.0);
                for tag in &all_tags {
                    let on = self.active_tag.as_ref().is_some_and(|a| a.eq_ignore_ascii_case(tag));
                    let color = if on { t().accent_dark } else { t().muted };
                    let resp = link(ui, tag, color);
                    if on {
                        ui.painter().hline(resp.rect.x_range(), resp.rect.bottom() - 2.0, Stroke::new(1.5, t().accent));
                    }
                    if resp.clicked() {
                        self.active_tag = if on { None } else { Some(tag.clone()) };
                    }
                }
            });
            ui.add_space(6.0);
        }

        let ctx = ui.ctx().clone();
        egui::Panel::bottom("theme_picker")
            .frame(egui::Frame::new().inner_margin(Margin::symmetric(0, 10)))
            .show(ui, |ui| {
                ui.label(RichText::new(format!("Theme: {}", t().name)).size(12.5).color(t().muted));
                ui.add_space(2.0);
                let current = THEME.load(Ordering::Relaxed);
                let mut pick = None;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    for (i, th) in THEMES.iter().enumerate() {
                        if i == 5 {
                            ui.add_space(6.0);
                        }
                        let (rect, resp) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::click());
                        let p = ui.painter();
                        p.rect_filled(rect, 3, th.bg);
                        p.rect_stroke(rect, 3, Stroke::new(1.0, th.muted), egui::StrokeKind::Inside);
                        p.rect_filled(egui::Rect::from_min_size(rect.min + Vec2::new(9.0, 9.0), Vec2::splat(9.0)), egui::CornerRadius { nw: 0, ne: 0, sw: 0, se: 3 }, th.accent);
                        if i == current {
                            p.rect_stroke(rect.expand(2.5), 4, Stroke::new(1.5, t().text), egui::StrokeKind::Outside);
                        }
                        if resp.on_hover_text(th.name).clicked() {
                            pick = Some(i);
                        }
                    }
                });
                if let Some(i) = pick {
                    set_theme(&ctx, i);
                }
            });

        let needle = self.filter.to_lowercase();
        let mut click = None;
        let active_tag = self.active_tag.clone();
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            for (i, r) in self.recipes.iter().enumerate() {
                if !needle.is_empty() && !r.name.to_lowercase().contains(&needle) {
                    continue;
                }
                if let Some(tag) = &active_tag {
                    if !r.tags.iter().any(|x| x.eq_ignore_ascii_case(tag)) {
                        continue;
                    }
                }
                let selected = i == self.selected;
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 50.0), Sense::click());
                let fill = if selected {
                    t().hover
                } else if resp.hovered() {
                    t().hover.gamma_multiply(0.6)
                } else {
                    Color32::TRANSPARENT
                };
                ui.painter().rect_filled(rect, 4, fill);
                let name = if r.name.trim().is_empty() { "(untitled)" } else { r.name.as_str() };
                let mut job = egui::text::LayoutJob::simple(name.to_string(), serif(16.5), t().text, rect.width() - 20.0);
                job.wrap.max_rows = 1;
                job.wrap.break_anywhere = true;
                let g = ui.painter().layout_job(job);
                ui.painter().galley(rect.left_top() + Vec2::new(10.0, 6.0), g, t().text);
                let sub = if r.tags.is_empty() {
                    format!("serves {}", r.servings)
                } else {
                    r.tags.join(" · ")
                };
                ui.painter().text(rect.left_top() + Vec2::new(10.0, 29.0), egui::Align2::LEFT_TOP, sub, FontId::proportional(12.5), t().muted);
                if resp.clicked() {
                    click = Some(i);
                }
            }
        });
        if let Some(i) = click {
            self.select(i);
        }
    }

    // ------------------------------------------------------------ main

    fn main_panel(&mut self, ui: &mut egui::Ui) {
        if self.recipes.is_empty() {
            ui.add_space(100.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("No recipes yet").font(serif(26.0)));
                ui.add_space(4.0);
                ui.label(RichText::new("Import one from the sidebar, or start a new one.").color(t().muted));
            });
            return;
        }
        self.selected = self.selected.min(self.recipes.len() - 1);

        let avail = ui.available_width();
        let content_w = (avail - 48.0).clamp(300.0, CONTENT_WIDTH);
        let side = ((avail - content_w) / 2.0).max(0.0);

        // Top bar: page tabs on the left, plain links on the right.
        ui.horizontal(|ui| {
            ui.add_space(side);
            ui.allocate_ui_with_layout(Vec2::new(content_w, 32.0), Layout::left_to_right(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 22.0;
                if tab(ui, "Recipe", !self.editing).clicked() {
                    self.editing = false;
                }
                if tab(ui, "Edit", self.editing).clicked() {
                    self.editing = true;
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 18.0;
                    if self.confirm_delete {
                        if link(ui, "Yes, delete", t().danger).clicked() {
                            self.recipes.remove(self.selected);
                            self.confirm_delete = false;
                            self.dirty = true;
                            if !self.recipes.is_empty() {
                                self.select(self.selected.min(self.recipes.len() - 1));
                            }
                            return;
                        }
                        if link(ui, "Cancel", t().muted).clicked() {
                            self.confirm_delete = false;
                        }
                        ui.label(RichText::new("Delete this recipe?").size(14.5).color(t().muted));
                    } else {
                        if link(ui, "Delete", t().muted).clicked() {
                            self.confirm_delete = true;
                        }
                        if link(ui, "Export", t().muted).clicked() {
                            self.export_current();
                        }
                    }
                });
            });
        });
        ui.add_space(8.0);
        if self.recipes.is_empty() {
            return;
        }

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.add_space(side);
                ui.vertical(|ui| {
                    ui.set_width(content_w);
                    ui.add_space(8.0);
                    if self.editing {
                        self.edit_view(ui);
                    } else {
                        self.read_view(ui, content_w);
                    }
                    ui.add_space(40.0);
                });
            });
        });
    }

    fn read_view(&mut self, ui: &mut egui::Ui, content_w: f32) {
        let base = self.recipes[self.selected].servings.max(1);
        let factor = self.target as f64 / base as f64;
        let mut apply = false;

        // ---- title block
        {
            let r = &self.recipes[self.selected];
            ui.label(RichText::new(&r.name).font(serif(40.0)).color(t().text));
            if !r.description.trim().is_empty() {
                ui.add_space(2.0);
                ui.label(RichText::new(&r.description).font(serif(17.0)).color(t().muted));
            }
            if !r.tags.is_empty() {
                ui.add_space(2.0);
                ui.label(RichText::new(r.tags.join("  ·  ")).size(13.5).color(t().accent_dark));
            }
        }
        ui.add_space(10.0);

        // ---- servings line
        ui.horizontal(|ui| {
            ui.label(RichText::new("Serves").font(serif(17.0)));
            if ui.add_sized([26.0, 26.0], egui::Button::new("−")).clicked() && self.target > 1 {
                self.target -= 1;
            }
            ui.add(egui::DragValue::new(&mut self.target).range(1..=1000).min_decimals(0));
            if ui.add_sized([26.0, 26.0], egui::Button::new("+")).clicked() {
                self.target += 1;
            }
            if self.target != base {
                ui.add_space(8.0);
                ui.label(RichText::new(format!("recipe makes {base}, ×{}", recipe::format_number(factor))).size(13.5).color(t().muted));
                if link(ui, "Reset", t().accent_dark).clicked() {
                    self.target = base;
                }
                if link(ui, "Save as new base", t().accent_dark).on_hover_text("Rewrite the stored amounts for this serving size").clicked() {
                    apply = true;
                }
            }
        });
        ui.add_space(14.0);

        let wide = content_w >= 680.0;
        let left_w = if wide { 280.0 } else { content_w };
        let gap = 44.0;
        let right_w = if wide { content_w - left_w - gap } else { content_w };

        let r = &self.recipes[self.selected];
        let done_ing = &mut self.done_ingredients;
        let done_steps = &mut self.done_steps;

        let ingredients = |ui: &mut egui::Ui, done: &mut HashSet<usize>| {
            heading(ui, "Ingredients");
            let mut toggle = None;
            egui::Grid::new("ingredients").num_columns(2).spacing([14.0, 8.0]).show(ui, |ui| {
                for (i, ing) in r.ingredients.iter().enumerate() {
                    let is_done = done.contains(&i);
                    let amt = scaled_amount(&ing.amount, factor);
                    let unit = display_unit(&ing.unit, &amt);
                    let qty = [amt, unit].iter().filter(|s| !s.is_empty()).cloned().collect::<Vec<_>>().join(" ");
                    let (mut q, mut n) = (RichText::new(qty).size(15.5), RichText::new(&ing.name).size(15.5));
                    if is_done {
                        q = q.color(t().muted).strikethrough();
                        n = n.color(t().muted).strikethrough();
                    } else {
                        q = q.color(t().accent_dark);
                    }
                    let a = ui.add(egui::Label::new(q).sense(Sense::click()));
                    let b = ui.add(egui::Label::new(n).sense(Sense::click()).wrap());
                    if a.clicked() || b.clicked() {
                        toggle = Some(i);
                    }
                    if a.hovered() || b.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    ui.end_row();
                }
            });
            if let Some(i) = toggle {
                if !done.remove(&i) {
                    done.insert(i);
                }
            }
            if r.ingredients.is_empty() {
                ui.label(RichText::new("No ingredients yet.").color(t().muted));
            }
        };

        let method = |ui: &mut egui::Ui, done: &mut HashSet<usize>| {
            heading(ui, "Method");
            let mut toggle = None;
            for (i, step) in r.steps.iter().enumerate() {
                let is_done = done.contains(&i);
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 14.0;
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(26.0, 28.0), Sense::hover());
                    let col = if is_done { t().muted } else { t().accent };
                    ui.painter().text(rect.right_top(), egui::Align2::RIGHT_TOP, (i + 1).to_string(), serif(26.0), col);
                    let mut st = RichText::new(step).size(16.0);
                    if is_done {
                        st = st.color(t().muted).strikethrough();
                    }
                    let resp = ui.add(egui::Label::new(st).sense(Sense::click()).wrap());
                    if resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if resp.clicked() {
                        toggle = Some(i);
                    }
                });
                ui.add_space(10.0);
            }
            if let Some(i) = toggle {
                if !done.remove(&i) {
                    done.insert(i);
                }
            }
            if r.steps.is_empty() {
                ui.label(RichText::new("No steps yet.").color(t().muted));
            }
        };

        if wide {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.vertical(|ui| {
                    ui.set_width(left_w);
                    ingredients(ui, done_ing);
                });
                ui.add_space(gap);
                ui.vertical(|ui| {
                    ui.set_width(right_w);
                    method(ui, done_steps);
                });
            });
        } else {
            ingredients(ui, done_ing);
            ui.add_space(14.0);
            method(ui, done_steps);
        }

        if !r.notes.trim().is_empty() {
            ui.add_space(14.0);
            heading(ui, "Notes");
            let resp = egui::Frame::new().inner_margin(Margin { left: 14, right: 0, top: 0, bottom: 0 }).show(ui, |ui| {
                ui.label(RichText::new(&r.notes).font(serif(16.0)).color(t().muted));
            });
            let rc = resp.response.rect;
            ui.painter().vline(rc.left() + 2.0, rc.y_range(), Stroke::new(2.0, t().border));
        }
        ui.add_space(8.0);
        ui.label(RichText::new("Click an ingredient or step to cross it off.").size(12.5).color(t().muted));

        if apply {
            let target = self.target;
            self.recipes[self.selected].rescale(target);
            self.dirty = true;
            self.status = format!("Amounts rewritten for {target} servings.");
        }
    }

    fn edit_view(&mut self, ui: &mut egui::Ui) {
        let target = &mut self.target;
        let new_tag = &mut self.new_tag;
        let r = &mut self.recipes[self.selected];
        let mut changed = false;

        small_label(ui, "Name");
        changed |= ui.add(egui::TextEdit::singleline(&mut r.name).font(serif(22.0)).desired_width(f32::INFINITY).margin(Margin::symmetric(8, 7))).changed();
        small_label(ui, "Description");
        changed |= ui.add(egui::TextEdit::multiline(&mut r.description).desired_rows(2).desired_width(f32::INFINITY).margin(Margin::symmetric(8, 7))).changed();
        small_label(ui, "Servings this recipe makes");
        if ui.add(egui::DragValue::new(&mut r.servings).range(1..=1000)).changed() {
            *target = r.servings;
            changed = true;
        }
        ui.add_space(14.0);

        heading(ui, "Tags");
        // Presets first, then any custom tags this recipe already has.
        let mut shown: Vec<String> = recipe::PRESET_TAGS.iter().map(|s| s.to_string()).collect();
        for tag in &r.tags {
            if !shown.iter().any(|x| x.eq_ignore_ascii_case(tag)) {
                shown.push(tag.clone());
            }
        }
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
            for tag in &shown {
                let pos = r.tags.iter().position(|x| x.eq_ignore_ascii_case(tag));
                if chip(ui, tag, pos.is_some()).clicked() {
                    match pos {
                        Some(p) => {
                            r.tags.remove(p);
                        }
                        None => r.tags.push(tag.clone()),
                    }
                    changed = true;
                }
            }
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let resp = ui.add(egui::TextEdit::singleline(new_tag).hint_text("Custom tag").desired_width(180.0).margin(Margin::symmetric(8, 5)));
            let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if (ui.add(small_button("Add")).clicked() || enter) && !new_tag.trim().is_empty() {
                recipe::add_tag(&mut r.tags, new_tag);
                new_tag.clear();
                changed = true;
            }
        });
        ui.add_space(14.0);

        heading(ui, "Ingredients");
        ui.label(RichText::new("Amount accepts 2, 0.5, 1/2 or 1 1/2. Leave it empty for \"salt to taste\".").size(13.0).color(t().muted));
        ui.add_space(4.0);
        let mut remove = None;
        let mut swap = None;
        let count = r.ingredients.len();
        for (i, ing) in r.ingredients.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                changed |= ui.add(egui::TextEdit::singleline(&mut ing.amount).desired_width(60.0).hint_text("Amt").margin(Margin::symmetric(8, 5))).changed();
                changed |= ui.add(egui::TextEdit::singleline(&mut ing.unit).desired_width(76.0).hint_text("Unit").margin(Margin::symmetric(8, 5))).changed();
                let name_w = (ui.available_width() - 3.0 * 30.0 - 14.0).max(80.0);
                changed |= ui.add(egui::TextEdit::singleline(&mut ing.name).desired_width(name_w).hint_text("Ingredient").margin(Margin::symmetric(8, 5))).changed();
                if ui.add_enabled(i > 0, egui::Button::new("↑").min_size(Vec2::splat(26.0))).clicked() { swap = Some((i, i - 1)); }
                if ui.add_enabled(i + 1 < count, egui::Button::new("↓").min_size(Vec2::splat(26.0))).clicked() { swap = Some((i, i + 1)); }
                if ui.add(egui::Button::new(RichText::new("×").color(t().danger).size(16.0)).min_size(Vec2::splat(26.0))).clicked() { remove = Some(i); }
            });
        }
        if let Some((a, b)) = swap { r.ingredients.swap(a, b); changed = true; }
        if let Some(i) = remove { r.ingredients.remove(i); changed = true; }
        if ui.add(small_button("Add ingredient")).clicked() {
            r.ingredients.push(Ingredient::default());
            changed = true;
        }
        ui.add_space(14.0);

        heading(ui, "Method");
        let mut remove = None;
        let mut swap = None;
        let count = r.steps.len();
        for (i, step) in r.steps.iter_mut().enumerate() {
            ui.horizontal_top(|ui| {
                ui.label(RichText::new(format!("{}", i + 1)).font(serif(20.0)).color(t().accent));
                let w = (ui.available_width() - 26.0 - 16.0).max(120.0);
                changed |= ui.add(egui::TextEdit::multiline(step).desired_rows(2).desired_width(w).margin(Margin::symmetric(8, 6))).changed();
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;
                    if ui.add_enabled(i > 0, egui::Button::new("↑").min_size(Vec2::splat(26.0))).clicked() { swap = Some((i, i - 1)); }
                    if ui.add_enabled(i + 1 < count, egui::Button::new("↓").min_size(Vec2::splat(26.0))).clicked() { swap = Some((i, i + 1)); }
                    if ui.add(egui::Button::new(RichText::new("×").color(t().danger).size(16.0)).min_size(Vec2::splat(26.0))).clicked() { remove = Some(i); }
                });
            });
            ui.add_space(2.0);
        }
        if let Some((a, b)) = swap { r.steps.swap(a, b); changed = true; }
        if let Some(i) = remove { r.steps.remove(i); changed = true; }
        if ui.add(small_button("Add step")).clicked() {
            r.steps.push(String::new());
            changed = true;
        }
        ui.add_space(14.0);

        heading(ui, "Notes");
        changed |= ui.add(egui::TextEdit::multiline(&mut r.notes).desired_rows(3).desired_width(f32::INFINITY).margin(Margin::symmetric(8, 6))).changed();

        if changed {
            self.dirty = true;
            self.done_ingredients.clear();
            self.done_steps.clear();
        }
    }

    fn import_window(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.import.as_mut() else { return };
        let mut open = true;
        let mut do_paste = false;
        let mut do_file = false;
        egui::Window::new(RichText::new("Import a recipe").font(serif(20.0)))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .default_width(560.0)
            .frame(egui::Frame::window(&ctx.global_style()).inner_margin(Margin::same(20)))
            .show(ctx, |ui| {
                ui.label(RichText::new("Paste your recipe. The first line is the title. Put Ingredients and Instructions on their own lines, and add \"Serves 4\" or \"Tags: Dinner, Quick\" lines if you like.").color(t().muted));
                ui.add_space(8.0);
                // Fixed-height box: long pastes scroll inside it instead of stretching the window.
                egui::ScrollArea::vertical()
                    .id_salt("import_scroll")
                    .min_scrolled_height(300.0)
                    .max_height(300.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut dialog.text)
                                .desired_rows(14)
                                .desired_width(f32::INFINITY)
                                .margin(Margin::symmetric(10, 8))
                                .hint_text("Pasta Night\nServes 4\nTags: Dinner, Quick\nIngredients\n2 cups flour\n...\nInstructions\n1. Mix.\n2. Bake."),
                        );
                    });
                if let Some(e) = &dialog.error {
                    ui.add_space(4.0);
                    ui.colored_label(t().danger, e);
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.add_sized([150.0, 32.0], primary_button("Import")).clicked() { do_paste = true; }
                    if ui.add_sized([150.0, 32.0], egui::Button::new("Open file…")).on_hover_text(".txt, .md or .json").clicked() { do_file = true; }
                });
            });

        if do_paste {
            let text = dialog.text.clone();
            // Pasted JSON works too.
            let result = if text.trim_start().starts_with(['{', '[']) {
                recipe::parse_json(&text)
            } else {
                recipe::parse_text(&text).map(|r| vec![r])
            };
            match result {
                Ok(list) => {
                    for r in list {
                        self.add_recipe(r);
                    }
                    self.import = None;
                    self.editing = true; // let the user check what was parsed
                    self.status = "Imported. Check the details and fix anything that looks off.".into();
                    return;
                }
                Err(e) => dialog.error = Some(e),
            }
        }
        if do_file {
            self.import_file();
        }
        if !open {
            self.import = None;
        }
    }
}

impl eframe::App for RecipeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("list")
            .default_size(260.0)
            .size_range(220.0..=360.0)
            .frame(egui::Frame::new().fill(t().sidebar).inner_margin(Margin::symmetric(14, 8)).stroke(Stroke::new(1.0, t().border)))
            .show(ui, |ui| self.sidebar(ui));
        egui::Panel::bottom("status")
            .frame(egui::Frame::new().fill(t().bg).inner_margin(Margin::symmetric(16, 5)))
            .show(ui, |ui| {
                let msg = if self.status.is_empty() { "Changes save automatically." } else { self.status.as_str() };
                ui.label(RichText::new(msg).size(12.0).color(t().muted));
            });
        egui::CentralPanel::default_margins().show(ui, |ui| self.main_panel(ui));
        let ctx = ui.ctx().clone();
        self.import_window(&ctx);

        if self.dirty {
            self.save();
        }
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1100.0, 760.0]).with_min_inner_size([760.0, 480.0]).with_title("Recipe Book"),
        ..Default::default()
    };
    eframe::run_native(
        "Recipe Book",
        options,
        Box::new(|cc| {
            load_theme();
            setup_fonts(&cc.egui_ctx);
            apply_theme(&cc.egui_ctx);
            Ok(Box::new(RecipeApp::new()))
        }),
    )
}
