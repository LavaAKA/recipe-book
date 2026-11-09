#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod recipe;

use eframe::egui::{self, Align, Color32, CornerRadius, FontId, Layout, Margin, RichText, Sense, Stroke, Vec2};
use recipe::{display_unit, scaled_amount, Ingredient, Recipe};
use std::collections::HashSet;
use std::path::PathBuf;

// ------------------------------------------------------------------ theme

use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Copy)]
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
    /// Accent used for text on cards / soft backgrounds (darker in light themes, lighter in dark ones).
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

const CONTENT_WIDTH: f32 = 740.0;

fn apply_theme(ctx: &egui::Context) {
    let mut v = if t().dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    v.panel_fill = t().bg;
    v.window_fill = t().card;
    v.extreme_bg_color = t().card; // text edit background
    v.faint_bg_color = t().sidebar;
    v.window_corner_radius = CornerRadius::same(14);
    v.window_stroke = Stroke::new(1.0, t().border);
    v.selection.bg_fill = t().accent_soft;
    v.selection.stroke = Stroke::new(1.0, t().accent);
    v.hyperlink_color = t().accent;

    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, t().border);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, t().text);
    for w in [&mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        w.corner_radius = CornerRadius::same(8);
    }
    v.widgets.inactive.bg_fill = t().card;
    v.widgets.inactive.weak_bg_fill = t().card;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, t().border);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, t().text);
    v.widgets.hovered.bg_fill = t().accent_soft;
    v.widgets.hovered.weak_bg_fill = t().accent_soft;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, t().accent);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, t().accent_dark);
    v.widgets.active.bg_fill = t().accent_soft;
    v.widgets.active.weak_bg_fill = t().accent_soft;
    v.widgets.active.bg_stroke = Stroke::new(1.0, t().accent_dark);
    v.widgets.active.fg_stroke = Stroke::new(1.0, t().accent_dark);
    ctx.set_visuals(v);

    ctx.global_style_mut(|s| {
        s.spacing.item_spacing = Vec2::new(10.0, 8.0);
        s.spacing.button_padding = Vec2::new(14.0, 7.0);
        s.spacing.interact_size.y = 32.0;
        s.text_styles.insert(egui::TextStyle::Body, FontId::proportional(16.0));
        s.text_styles.insert(egui::TextStyle::Button, FontId::proportional(15.0));
        s.text_styles.insert(egui::TextStyle::Small, FontId::proportional(13.0));
        s.text_styles.insert(egui::TextStyle::Heading, FontId::proportional(30.0));
    });
}

fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .fill(t().card)
        .stroke(Stroke::new(1.0, t().border))
        .corner_radius(14)
        .inner_margin(Margin::same(20))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
}

fn section_label(ui: &mut egui::Ui, text: &str) {
    ui.add_space(6.0);
    ui.label(RichText::new(text.to_uppercase()).size(12.5).strong().color(t().muted).extra_letter_spacing(1.2));
    ui.add_space(2.0);
}

fn primary_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.to_string()).color(t().on_accent).strong())
        .fill(t().accent)
        .stroke(Stroke::NONE)
        .corner_radius(10)
}

fn small_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.to_string()).size(14.0)).corner_radius(8)
}

/// A round numbered badge. Filled green with a tick when done.
fn badge(ui: &mut egui::Ui, text: &str, done: bool) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(30.0), Sense::hover());
    let p = ui.painter();
    if done {
        p.circle_filled(rect.center(), 15.0, t().done);
        p.text(rect.center(), egui::Align2::CENTER_CENTER, "✓", FontId::proportional(15.0), t().on_accent);
    } else {
        p.circle_filled(rect.center(), 15.0, t().accent_soft);
        p.text(rect.center(), egui::Align2::CENTER_CENTER, text, FontId::proportional(14.5), t().accent_dark);
    }
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
        Ok(text) => recipe::parse_json(&text).unwrap_or_default(),
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
        ui.label(RichText::new("Recipe Book").size(22.0).strong().color(t().text));
        ui.label(RichText::new(format!("{} recipe{}", self.recipes.len(), if self.recipes.len() == 1 { "" } else { "s" })).size(13.0).color(t().muted));
        ui.add_space(8.0);

        let w = ui.available_width();
        if ui.add_sized([w, 38.0], primary_button("+  Import recipe")).clicked() {
            self.import = Some(ImportDialog { text: String::new(), error: None });
        }
        if ui.add_sized([w, 34.0], egui::Button::new("New blank recipe").corner_radius(10)).clicked() {
            self.add_recipe(Recipe::blank());
            self.editing = true;
        }
        ui.add_space(4.0);
        ui.add(egui::TextEdit::singleline(&mut self.filter).hint_text("Search recipes").desired_width(f32::INFINITY).margin(Margin::symmetric(10, 7)));
        ui.add_space(6.0);

        let ctx = ui.ctx().clone();
        egui::Panel::bottom("theme_picker")
            .frame(egui::Frame::new().inner_margin(Margin::symmetric(0, 8)))
            .show(ui, |ui| {
                ui.label(RichText::new(format!("THEME · {}", t().name.to_uppercase())).size(12.0).strong().color(t().muted).extra_letter_spacing(1.0));
                ui.add_space(2.0);
                let current = THEME.load(Ordering::Relaxed);
                let mut pick = None;
                for row in THEMES.chunks(5).enumerate() {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        for (j, th) in row.1.iter().enumerate() {
                            let i = row.0 * 5 + j;
                            let (rect, resp) = ui.allocate_exact_size(Vec2::splat(34.0), Sense::click());
                            let p = ui.painter();
                            p.circle_filled(rect.center(), 15.0, th.bg);
                            p.circle_stroke(rect.center(), 15.0, Stroke::new(1.0, th.border));
                            p.circle_filled(rect.center(), 8.0, th.accent);
                            if i == current {
                                p.circle_stroke(rect.center(), 16.5, Stroke::new(2.5, t().accent_dark));
                            }
                            if resp.on_hover_text(format!("{} ({})", th.name, if th.dark { "dark" } else { "light" })).clicked() {
                                pick = Some(i);
                            }
                        }
                    });
                }
                if let Some(i) = pick {
                    set_theme(&ctx, i);
                }
            });

        let needle = self.filter.to_lowercase();
        let mut click = None;
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            for (i, r) in self.recipes.iter().enumerate() {
                if !needle.is_empty() && !r.name.to_lowercase().contains(&needle) {
                    continue;
                }
                let selected = i == self.selected;
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 52.0), Sense::click());
                let fill = if selected {
                    t().accent_soft
                } else if resp.hovered() {
                    t().hover
                } else {
                    Color32::TRANSPARENT
                };
                ui.painter().rect_filled(rect, 10, fill);
                if selected {
                    let bar = egui::Rect::from_min_size(rect.left_top() + Vec2::new(0.0, 12.0), Vec2::new(3.0, rect.height() - 24.0));
                    ui.painter().rect_filled(bar, 2, t().accent);
                }
                let name = if r.name.trim().is_empty() { "(untitled)" } else { r.name.as_str() };
                let mut job = egui::text::LayoutJob::simple(name.to_string(), FontId::proportional(15.5), if selected { t().accent_dark } else { t().text }, rect.width() - 28.0);
                job.wrap.max_rows = 1;
                job.wrap.break_anywhere = true;
                let g = ui.painter().layout_job(job);
                ui.painter().galley(rect.left_top() + Vec2::new(14.0, 8.0), g, t().text);
                let s = |n: usize, w: &str| format!("{n} {w}{}", if n == 1 { "" } else { "s" });
                let sub = format!("{} · {}", s(r.servings as usize, "serving"), s(r.ingredients.len(), "ingredient"));
                ui.painter().text(rect.left_top() + Vec2::new(14.0, 30.0), egui::Align2::LEFT_TOP, sub, FontId::proportional(12.5), t().muted);
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
            ui.add_space(120.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("No recipes yet").size(26.0).strong());
                ui.add_space(6.0);
                ui.label(RichText::new("Import one from the sidebar, or start a blank recipe.").color(t().muted));
            });
            return;
        }
        self.selected = self.selected.min(self.recipes.len() - 1);

        let avail = ui.available_width();
        let content_w = (avail - 48.0).clamp(300.0, CONTENT_WIDTH);
        let side = ((avail - content_w) / 2.0).max(0.0);

        // Toolbar
        ui.horizontal(|ui| {
            ui.add_space(side);
            ui.allocate_ui_with_layout(Vec2::new(content_w, 36.0), Layout::left_to_right(Align::Center), |ui| {
                let view = ui.add(egui::Button::new(RichText::new("Recipe").strong()).selected(!self.editing).corner_radius(8));
                let edit = ui.add(egui::Button::new(RichText::new("Edit").strong()).selected(self.editing).corner_radius(8));
                if view.clicked() {
                    self.editing = false;
                }
                if edit.clicked() {
                    self.editing = true;
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if self.confirm_delete {
                        if ui.add(egui::Button::new(RichText::new("Yes, delete").color(t().on_accent)).fill(t().danger).corner_radius(8)).clicked() {
                            self.recipes.remove(self.selected);
                            self.confirm_delete = false;
                            self.dirty = true;
                            if !self.recipes.is_empty() {
                                self.select(self.selected.min(self.recipes.len() - 1));
                            }
                            return;
                        }
                        if ui.add(small_button("Cancel")).clicked() {
                            self.confirm_delete = false;
                        }
                        ui.label(RichText::new("Delete this recipe?").color(t().muted));
                    } else {
                        if ui.add(egui::Button::new(RichText::new("Delete").color(t().danger).size(14.0)).corner_radius(8)).clicked() {
                            self.confirm_delete = true;
                        }
                        if ui.add(small_button("Export")).clicked() {
                            self.export_current();
                        }
                    }
                });
            });
        });
        ui.add_space(6.0);
        if self.recipes.is_empty() {
            return;
        }

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.add_space(side);
                ui.vertical(|ui| {
                    ui.set_width(content_w);
                    ui.add_space(6.0);
                    if self.editing {
                        self.edit_view(ui);
                    } else {
                        self.read_view(ui);
                    }
                    ui.add_space(40.0);
                });
            });
        });
    }

    fn read_view(&mut self, ui: &mut egui::Ui) {
        let base = self.recipes[self.selected].servings.max(1);
        let factor = self.target as f64 / base as f64;
        let mut apply = false;

        // ---- header
        {
            let r = &self.recipes[self.selected];
            ui.label(RichText::new(&r.name).size(34.0).strong().color(t().text));
            if !r.description.trim().is_empty() {
                ui.add_space(2.0);
                ui.label(RichText::new(&r.description).size(16.5).color(t().muted));
            }
        }
        ui.add_space(14.0);

        // ---- servings card
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Servings").size(16.0).strong());
                ui.add_space(6.0);
                if ui.add_sized([34.0, 34.0], egui::Button::new(RichText::new("−").size(18.0)).corner_radius(17)).clicked() && self.target > 1 {
                    self.target -= 1;
                }
                ui.add(egui::DragValue::new(&mut self.target).range(1..=1000).min_decimals(0));
                if ui.add_sized([34.0, 34.0], egui::Button::new(RichText::new("+").size(18.0)).corner_radius(17)).clicked() {
                    self.target += 1;
                }
                if self.target != base {
                    ui.add_space(6.0);
                    ui.label(RichText::new(format!("recipe makes {base} · ×{}", recipe::format_number(factor))).size(14.0).color(t().muted));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.add(small_button("Save as new base")).on_hover_text("Rewrite the stored amounts for this serving size").clicked() {
                            apply = true;
                        }
                        if ui.add(small_button("Reset")).clicked() {
                            self.target = base;
                        }
                    });
                }
            });
        });
        ui.add_space(12.0);

        // ---- ingredients
        let r = &self.recipes[self.selected];
        card(ui, |ui| {
            ui.label(RichText::new("Ingredients").size(21.0).strong());
            ui.add_space(8.0);
            let mut toggle = None;
            egui::Grid::new("ingredients").num_columns(2).spacing([18.0, 10.0]).show(ui, |ui| {
                for (i, ing) in r.ingredients.iter().enumerate() {
                    let done = self.done_ingredients.contains(&i);
                    let amt = scaled_amount(&ing.amount, factor);
                    let unit = display_unit(&ing.unit, &amt);
                    let qty = [amt, unit].iter().filter(|s| !s.is_empty()).cloned().collect::<Vec<_>>().join(" ");
                    let mut q = RichText::new(qty).strong().size(16.0).color(t().accent_dark);
                    let mut n = RichText::new(&ing.name).size(16.0);
                    if done {
                        q = q.color(t().muted).strikethrough();
                        n = n.color(t().muted).strikethrough();
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
                if !self.done_ingredients.remove(&i) {
                    self.done_ingredients.insert(i);
                }
            }
            if r.ingredients.is_empty() {
                ui.label(RichText::new("No ingredients yet. Switch to Edit to add some.").color(t().muted));
            }
        });
        ui.add_space(12.0);

        // ---- instructions
        card(ui, |ui| {
            ui.label(RichText::new("Instructions").size(21.0).strong());
            ui.add_space(8.0);
            let mut toggle = None;
            for (i, step) in r.steps.iter().enumerate() {
                let done = self.done_steps.contains(&i);
                ui.horizontal_top(|ui| {
                    badge(ui, &(i + 1).to_string(), done);
                    let mut st = RichText::new(step).size(16.5);
                    if done {
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
                ui.add_space(8.0);
            }
            if let Some(i) = toggle {
                if !self.done_steps.remove(&i) {
                    self.done_steps.insert(i);
                }
            }
            if r.steps.is_empty() {
                ui.label(RichText::new("No steps yet. Switch to Edit to add some.").color(t().muted));
            }
            ui.label(RichText::new("Click a step or ingredient to tick it off.").size(12.5).color(t().muted));
        });

        if !r.notes.trim().is_empty() {
            ui.add_space(12.0);
            egui::Frame::new().fill(t().accent_soft).corner_radius(14).inner_margin(Margin::same(20)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new("Notes").size(18.0).strong().color(t().accent_dark));
                ui.add_space(4.0);
                ui.label(RichText::new(&r.notes).size(15.5).color(t().text));
            });
        }

        if apply {
            let t = self.target;
            self.recipes[self.selected].rescale(t);
            self.dirty = true;
            self.status = format!("Amounts rewritten for {t} servings.");
        }
    }

    fn edit_view(&mut self, ui: &mut egui::Ui) {
        let target = &mut self.target;
        let r = &mut self.recipes[self.selected];
        let mut changed = false;

        card(ui, |ui| {
            section_label(ui, "Name");
            changed |= ui.add(egui::TextEdit::singleline(&mut r.name).font(FontId::proportional(20.0)).desired_width(f32::INFINITY).margin(Margin::symmetric(10, 8))).changed();
            section_label(ui, "Description");
            changed |= ui.add(egui::TextEdit::multiline(&mut r.description).desired_rows(2).desired_width(f32::INFINITY).margin(Margin::symmetric(10, 8))).changed();
            section_label(ui, "Servings this recipe makes");
            if ui.add(egui::DragValue::new(&mut r.servings).range(1..=1000)).changed() {
                *target = r.servings;
                changed = true;
            }
        });
        ui.add_space(12.0);

        card(ui, |ui| {
            ui.label(RichText::new("Ingredients").size(21.0).strong());
            ui.label(RichText::new("Amount accepts 2, 0.5, 1/2 or 1 1/2. Leave it empty for \"salt to taste\".").size(13.0).color(t().muted));
            ui.add_space(6.0);
            let mut remove = None;
            let mut swap = None;
            let count = r.ingredients.len();
            for (i, ing) in r.ingredients.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    changed |= ui.add(egui::TextEdit::singleline(&mut ing.amount).desired_width(60.0).hint_text("Amt").margin(Margin::symmetric(8, 6))).changed();
                    changed |= ui.add(egui::TextEdit::singleline(&mut ing.unit).desired_width(76.0).hint_text("Unit").margin(Margin::symmetric(8, 6))).changed();
                    let name_w = (ui.available_width() - 3.0 * 34.0 - 12.0).max(80.0);
                    changed |= ui.add(egui::TextEdit::singleline(&mut ing.name).desired_width(name_w).hint_text("Ingredient").margin(Margin::symmetric(8, 6))).changed();
                    if ui.add_enabled(i > 0, egui::Button::new("↑").min_size(Vec2::splat(28.0))).clicked() { swap = Some((i, i - 1)); }
                    if ui.add_enabled(i + 1 < count, egui::Button::new("↓").min_size(Vec2::splat(28.0))).clicked() { swap = Some((i, i + 1)); }
                    if ui.add(egui::Button::new(RichText::new("×").color(t().danger).size(17.0)).min_size(Vec2::splat(28.0))).clicked() { remove = Some(i); }
                });
            }
            if let Some((a, b)) = swap { r.ingredients.swap(a, b); changed = true; }
            if let Some(i) = remove { r.ingredients.remove(i); changed = true; }
            ui.add_space(4.0);
            if ui.add(small_button("+  Add ingredient")).clicked() {
                r.ingredients.push(Ingredient::default());
                changed = true;
            }
        });
        ui.add_space(12.0);

        card(ui, |ui| {
            ui.label(RichText::new("Instructions").size(21.0).strong());
            ui.add_space(6.0);
            let mut remove = None;
            let mut swap = None;
            let count = r.steps.len();
            for (i, step) in r.steps.iter_mut().enumerate() {
                ui.horizontal_top(|ui| {
                    badge(ui, &(i + 1).to_string(), false);
                    let w = (ui.available_width() - 28.0 - 18.0).max(120.0);
                    changed |= ui.add(egui::TextEdit::multiline(step).desired_rows(2).desired_width(w).margin(Margin::symmetric(10, 8))).changed();
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 4.0;
                        if ui.add_enabled(i > 0, egui::Button::new("↑").min_size(Vec2::splat(28.0))).clicked() { swap = Some((i, i - 1)); }
                        if ui.add_enabled(i + 1 < count, egui::Button::new("↓").min_size(Vec2::splat(28.0))).clicked() { swap = Some((i, i + 1)); }
                        if ui.add(egui::Button::new(RichText::new("×").color(t().danger).size(17.0)).min_size(Vec2::splat(28.0))).clicked() { remove = Some(i); }
                    });
                });
                ui.add_space(4.0);
            }
            if let Some((a, b)) = swap { r.steps.swap(a, b); changed = true; }
            if let Some(i) = remove { r.steps.remove(i); changed = true; }
            if ui.add(small_button("+  Add step")).clicked() {
                r.steps.push(String::new());
                changed = true;
            }
        });
        ui.add_space(12.0);

        card(ui, |ui| {
            ui.label(RichText::new("Notes").size(21.0).strong());
            ui.add_space(4.0);
            changed |= ui.add(egui::TextEdit::multiline(&mut r.notes).desired_rows(3).desired_width(f32::INFINITY).margin(Margin::symmetric(10, 8))).changed();
        });

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
        egui::Window::new(RichText::new("Import a recipe").size(20.0).strong())
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .default_width(560.0)
            .frame(egui::Frame::window(&ctx.global_style()).inner_margin(Margin::same(22)))
            .show(ctx, |ui| {
                ui.label(RichText::new("Paste your recipe. The first line is the title. Put Ingredients and Instructions on their own lines, and add \"Serves 4\" to set the servings.").color(t().muted));
                ui.add_space(8.0);
                ui.add(
                    egui::TextEdit::multiline(&mut dialog.text)
                        .desired_rows(14)
                        .desired_width(f32::INFINITY)
                        .margin(Margin::symmetric(12, 10))
                        .hint_text("Pasta Night\nServes 4\nIngredients\n2 cups flour\n...\nInstructions\n1. Mix.\n2. Bake."),
                );
                if let Some(e) = &dialog.error {
                    ui.add_space(4.0);
                    ui.colored_label(t().danger, e);
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.add_sized([170.0, 36.0], primary_button("Import pasted text")).clicked() { do_paste = true; }
                    if ui.add_sized([170.0, 36.0], egui::Button::new("Open file…").corner_radius(10)).on_hover_text(".txt, .md or .json").clicked() { do_file = true; }
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
            .default_size(270.0)
            .size_range(220.0..=360.0)
            .frame(egui::Frame::new().fill(t().sidebar).inner_margin(Margin::symmetric(14, 8)).stroke(Stroke::new(1.0, t().border)))
            .show(ui, |ui| self.sidebar(ui));
        egui::Panel::bottom("status")
            .frame(egui::Frame::new().fill(t().bg).inner_margin(Margin::symmetric(16, 6)))
            .show(ui, |ui| {
                let msg = if self.status.is_empty() { "Changes save automatically." } else { self.status.as_str() };
                ui.label(RichText::new(msg).size(12.5).color(t().muted));
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
            apply_theme(&cc.egui_ctx);
            Ok(Box::new(RecipeApp::new()))
        }),
    )
}
