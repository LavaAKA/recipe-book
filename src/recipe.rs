//! Recipe data model, quantity scaling/formatting, and the plain-text importer.

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Ingredient {
    /// Free text so the user can type "1 1/2", "0.5", "½" ... Empty = no amount ("salt to taste").
    #[serde(default, deserialize_with = "string_or_number")]
    pub amount: String,
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_servings")]
    pub servings: u32,
    #[serde(default)]
    pub ingredients: Vec<Ingredient>,
    #[serde(default)]
    pub steps: Vec<String>,
    #[serde(default)]
    pub notes: String,
}

fn default_servings() -> u32 {
    4
}

fn string_or_number<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum V {
        S(String),
        N(f64),
    }
    Ok(match V::deserialize(d)? {
        V::S(s) => s,
        V::N(n) => format_number(n),
    })
}

impl Recipe {
    pub fn blank() -> Self {
        Recipe {
            name: "New recipe".into(),
            description: String::new(),
            servings: 4,
            ingredients: vec![Ingredient::default()],
            steps: vec![String::new()],
            notes: String::new(),
        }
    }

    /// Multiply every amount in place and set the new base serving count.
    pub fn rescale(&mut self, new_servings: u32) {
        let factor = new_servings as f64 / self.servings.max(1) as f64;
        for ing in &mut self.ingredients {
            if let Some(v) = parse_amount(&ing.amount) {
                ing.amount = format_number(v * factor);
            }
        }
        self.servings = new_servings;
    }
}

// ---------------------------------------------------------------- amounts

const UNICODE_FRACTIONS: &[(char, &str)] = &[
    ('½', "1/2"),
    ('⅓', "1/3"),
    ('⅔', "2/3"),
    ('¼', "1/4"),
    ('¾', "3/4"),
    ('⅕', "1/5"),
    ('⅛', "1/8"),
    ('⅜', "3/8"),
    ('⅝', "5/8"),
    ('⅞', "7/8"),
];

/// Parse "2", "0.5", "1/2", "1 1/2", "1½" ... Returns None for anything else (ranges, words).
pub fn parse_amount(s: &str) -> Option<f64> {
    let mut text = s.trim().to_string();
    if text.is_empty() {
        return None;
    }
    for (c, rep) in UNICODE_FRACTIONS {
        text = text.replace(*c, &format!(" {rep} "));
    }
    let mut total = 0.0;
    let mut any = false;
    for tok in text.split_whitespace() {
        total += parse_token(tok)?;
        any = true;
    }
    any.then_some(total)
}

fn parse_token(tok: &str) -> Option<f64> {
    if let Some((a, b)) = tok.split_once('/') {
        let a: f64 = a.parse().ok()?;
        let b: f64 = b.parse().ok()?;
        if b == 0.0 {
            return None;
        }
        return Some(a / b);
    }
    let v: f64 = tok.parse().ok()?;
    (v.is_finite() && v >= 0.0).then_some(v)
}

/// Pretty-print a quantity: 1.5 -> "1 1/2", 0.333 -> "1/3", 2.0 -> "2", 2.37 -> "2.37".
pub fn format_number(x: f64) -> String {
    if !x.is_finite() || x <= 0.0 {
        return "0".into();
    }
    const FRACS: &[(f64, &str)] = &[
        (0.125, "1/8"),
        (0.25, "1/4"),
        (1.0 / 3.0, "1/3"),
        (0.375, "3/8"),
        (0.5, "1/2"),
        (0.625, "5/8"),
        (2.0 / 3.0, "2/3"),
        (0.75, "3/4"),
        (0.875, "7/8"),
    ];
    let mut whole = x.floor();
    let frac = x - whole;
    if frac < 0.02 {
        return fmt_int(whole);
    }
    if frac > 0.98 {
        return fmt_int(whole + 1.0);
    }
    for (v, s) in FRACS {
        if (frac - v).abs() < 0.02 {
            return if whole == 0.0 { (*s).to_string() } else { format!("{} {}", fmt_int(whole), s) };
        }
    }
    // No clean fraction: fall back to up to 2 decimals.
    let rounded = (x * 100.0).round() / 100.0;
    whole = rounded;
    let mut s = format!("{whole:.2}");
    while s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    s
}

fn fmt_int(x: f64) -> String {
    format!("{}", x as u64)
}

/// Scaled amount text for display. Non-numeric amounts are shown unchanged.
pub fn scaled_amount(amount: &str, factor: f64) -> String {
    match parse_amount(amount) {
        Some(v) => format_number(v * factor),
        None => amount.trim().to_string(),
    }
}

/// "cup" -> "cups" when the amount is more than one, for common countable units.
pub fn display_unit(unit: &str, shown_amount: &str) -> String {
    const COUNTABLE: &[&str] = &[
        "cup", "clove", "can", "ounce", "pound", "slice", "piece", "stick", "package",
    ];
    let u = unit.trim();
    let base = u.strip_suffix('s').unwrap_or(u);
    if COUNTABLE.contains(&base.to_lowercase().as_str()) {
        let plural = parse_amount(shown_amount).map(|v| v > 1.001).unwrap_or(false);
        return if plural { format!("{base}s") } else { base.to_string() };
    }
    u.to_string()
}

const UNITS: &[&str] = &[
    "tsp", "teaspoon", "teaspoons", "tbsp", "tablespoon", "tablespoons", "cup", "cups", "oz", "ounce",
    "ounces", "lb", "lbs", "pound", "pounds", "g", "gram", "grams", "kg", "ml", "l", "liter", "liters",
    "clove", "cloves", "can", "cans", "pinch", "dash", "slice", "slices", "piece", "pieces", "stick",
    "sticks", "package", "packages", "pkg",
];

// ---------------------------------------------------------------- import

fn parse_ingredient_line(line: &str) -> Ingredient {
    let line = line.trim();
    // Peel off the leading amount: every leading token that parses as a number/fraction.
    let mut amount_tokens: Vec<&str> = Vec::new();
    let mut rest = line;
    loop {
        let trimmed = rest.trim_start();
        let (tok, after) = match trimmed.find(char::is_whitespace) {
            Some(i) => (&trimmed[..i], &trimmed[i..]),
            None => (trimmed, ""),
        };
        // Handle glued unit like "2tbsp" or "1½": only accept pure numeric tokens here.
        if !tok.is_empty() && parse_amount(tok).is_some() {
            amount_tokens.push(tok);
            rest = after;
        } else {
            break;
        }
    }
    if amount_tokens.is_empty() {
        return Ingredient { amount: String::new(), unit: String::new(), name: line.to_string() };
    }
    let rest = rest.trim();
    let (unit, name) = match rest.split_once(char::is_whitespace) {
        Some((first, tail)) if UNITS.contains(&first.trim_end_matches('.').to_lowercase().as_str()) => {
            (first.trim_end_matches('.').to_string(), tail.trim().to_string())
        }
        None if UNITS.contains(&rest.to_lowercase().as_str()) => (rest.to_string(), String::new()),
        _ => (String::new(), rest.to_string()),
    };
    Ingredient {
        amount: amount_tokens.join(" "),
        unit,
        name: name.trim_start_matches("of ").to_string(),
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Section {
    Intro,
    Ingredients,
    Steps,
    Notes,
}

fn heading_of(line: &str) -> Option<Section> {
    let h = line
        .trim()
        .trim_start_matches('#')
        .trim()
        .trim_end_matches(':')
        .trim()
        .to_lowercase();
    match h.as_str() {
        "ingredients" | "ingredient list" => Some(Section::Ingredients),
        "instructions" | "directions" | "steps" | "method" | "preparation" | "procedure" => {
            Some(Section::Steps)
        }
        "notes" | "tips" | "note" => Some(Section::Notes),
        _ => None,
    }
}

fn strip_step_prefix(line: &str) -> String {
    let l = line.trim();
    let lower = l.to_lowercase();
    let l = if lower.starts_with("step") {
        l[4..].trim_start().trim_start_matches(|c: char| c.is_ascii_digit()).trim_start_matches([':', '.', ')', '-']).trim()
    } else {
        let digits = l.chars().take_while(|c| c.is_ascii_digit()).count();
        if digits > 0 && l[digits..].starts_with(['.', ')']) {
            l[digits + 1..].trim()
        } else {
            l
        }
    };
    l.to_string()
}

fn is_bare_step_label(line: &str) -> bool {
    let l = line.trim().trim_end_matches(':').to_lowercase();
    l.strip_prefix("step")
        .map(|r| !r.trim().is_empty() && r.trim().chars().all(|c| c.is_ascii_digit()))
        .unwrap_or(false)
}

fn servings_from(line: &str) -> Option<u32> {
    let lower = line.trim().to_lowercase();
    for key in ["servings", "serves", "yield", "makes"] {
        if let Some(r) = lower.strip_prefix(key) {
            let digits: String = r
                .trim_start_matches([':', ' '])
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if let Ok(n) = digits.parse::<u32>() {
                if n > 0 {
                    return Some(n);
                }
            }
        }
    }
    None
}

/// Turn pasted recipe text into a Recipe. Understands "Ingredients" / "Instructions" headings,
/// bullets, numbered steps, "Step 1" labels and "Serves 4" lines.
pub fn parse_text(text: &str) -> Result<Recipe, String> {
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
    let title = lines
        .next()
        .ok_or_else(|| "Nothing to import: the text is empty.".to_string())?
        .trim_start_matches('#')
        .trim()
        .to_string();

    let mut r = Recipe {
        name: title,
        description: String::new(),
        servings: 4,
        ingredients: Vec::new(),
        steps: Vec::new(),
        notes: String::new(),
    };
    let mut section = Section::Intro;
    let mut saw_heading = false;

    for line in lines {
        if let Some(s) = heading_of(line) {
            section = s;
            saw_heading = true;
            continue;
        }
        if let Some(n) = servings_from(line) {
            r.servings = n;
            continue;
        }
        match section {
            Section::Intro => {
                if !r.description.is_empty() {
                    r.description.push(' ');
                }
                r.description.push_str(line);
            }
            Section::Ingredients => {
                let l = line.trim_start_matches(['-', '*', '•', '–']).trim();
                if !l.is_empty() {
                    r.ingredients.push(parse_ingredient_line(l));
                }
            }
            Section::Steps => {
                if is_bare_step_label(line) {
                    continue;
                }
                let l = strip_step_prefix(line);
                if !l.is_empty() {
                    r.steps.push(l);
                }
            }
            Section::Notes => {
                if !r.notes.is_empty() {
                    r.notes.push('\n');
                }
                r.notes.push_str(line);
            }
        }
    }

    if !saw_heading {
        // No headings: treat the remaining text as the method so nothing is lost.
        r.steps = r.description.split(". ").map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        r.description.clear();
    }
    if r.ingredients.is_empty() && r.steps.is_empty() {
        return Err("Couldn't find any ingredients or steps. Add \"Ingredients\" and \"Instructions\" headings.".into());
    }
    Ok(r)
}

/// Parse JSON: either one recipe object or an array of them.
pub fn parse_json(text: &str) -> Result<Vec<Recipe>, String> {
    if let Ok(list) = serde_json::from_str::<Vec<Recipe>>(text) {
        return Ok(list);
    }
    serde_json::from_str::<Recipe>(text)
        .map(|r| vec![r])
        .map_err(|e| format!("Not valid recipe JSON: {e}"))
}

pub fn sample() -> Recipe {
    parse_text(
        "Cajun Chicken Fettuccine Alfredo\n\
         Juicy Cajun-spiced chicken sliced over fettuccine in a rich, garlicky parmesan cream sauce.\n\
         Serves 4\n\
         Ingredients\n\
         2 boneless, skinless chicken breasts, halved lengthwise\n\
         1 tbsp Cajun seasoning\n\
         1/2 tsp garlic powder\n\
         1 tsp ground black pepper, divided\n\
         3 tbsp salted butter\n\
         2 tbsp minced garlic\n\
         1 tbsp all-purpose flour\n\
         2 cups heavy cream\n\
         1/2 tsp salt\n\
         1 cup grated parmesan cheese\n\
         12 oz fettuccine\n\
         Instructions\n\
         1. Cut the chicken in half lengthwise into thin cutlets. Season with the Cajun seasoning, garlic powder and 1/2 tsp of the pepper.\n\
         2. Heat a large skillet over medium heat. Cook the chicken 4 minutes per side, then set aside.\n\
         3. Lower the heat. Stir the butter, garlic and flour in the skillet until it begins to thicken.\n\
         4. Whisk in the cream, salt, remaining 1/2 tsp pepper and parmesan for 6-8 minutes until smooth, then reduce to a simmer.\n\
         5. Boil the fettuccine according to the package directions. Drain and toss with the sauce.\n\
         6. Slice the chicken and serve it over the pasta.\n\
         Notes\n\
         Start the pasta water while the chicken sears. Garnish with parsley and extra parmesan.",
    )
    .expect("sample recipe parses")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amounts() {
        assert_eq!(parse_amount("1 1/2"), Some(1.5));
        assert_eq!(parse_amount("1½"), Some(1.5));
        assert_eq!(parse_amount("0.5"), Some(0.5));
        assert_eq!(parse_amount("a pinch"), None);
        assert_eq!(parse_amount("1-2"), None);
        assert_eq!(format_number(1.5), "1 1/2");
        assert_eq!(format_number(0.3333), "1/3");
        assert_eq!(format_number(2.0), "2");
        assert_eq!(format_number(2.37), "2 3/8");
        assert_eq!(format_number(2.2), "2.2");
        assert_eq!(format_number(0.999), "1");
    }

    #[test]
    fn scaling() {
        assert_eq!(scaled_amount("1/2", 2.0), "1");
        assert_eq!(scaled_amount("12", 1.5), "18");
        assert_eq!(scaled_amount("1", 0.5), "1/2");
        assert_eq!(scaled_amount("to taste", 3.0), "to taste");
        assert_eq!(display_unit("cup", "2"), "cups");
        assert_eq!(display_unit("cups", "1"), "cup");
        assert_eq!(display_unit("tbsp", "3"), "tbsp");
    }

    #[test]
    fn ingredient_lines() {
        let i = parse_ingredient_line("1 1/2 cups heavy cream");
        assert_eq!((i.amount.as_str(), i.unit.as_str(), i.name.as_str()), ("1 1/2", "cups", "heavy cream"));
        let i = parse_ingredient_line("2 boneless chicken breasts");
        assert_eq!((i.amount.as_str(), i.unit.as_str(), i.name.as_str()), ("2", "", "boneless chicken breasts"));
        let i = parse_ingredient_line("Salt to taste");
        assert_eq!((i.amount.as_str(), i.name.as_str()), ("", "Salt to taste"));
    }

    #[test]
    fn sample_parses() {
        let r = sample();
        assert_eq!(r.name, "Cajun Chicken Fettuccine Alfredo");
        assert_eq!(r.servings, 4);
        assert_eq!(r.ingredients.len(), 11);
        assert_eq!(r.steps.len(), 6);
        assert!(r.steps[0].starts_with("Cut the chicken"));
        assert!(r.notes.contains("pasta water"));
    }

    #[test]
    fn step_labels_and_rescale() {
        let r = parse_text("Toast\nIngredients\n2 slices bread\nSteps\nStep 1\nToast it.\nStep 2\nEat it.").unwrap();
        assert_eq!(r.steps, vec!["Toast it.", "Eat it."]);
        let mut r = r;
        r.rescale(8); // default base is 4 servings, so x2
        assert_eq!(r.ingredients[0].amount, "4");
        assert_eq!(r.servings, 8);
    }

    #[test]
    fn json_roundtrip() {
        let r = parse_json(r#"{"name":"X","ingredients":[{"amount":2,"unit":"cup","name":"milk"}]}"#).unwrap();
        assert_eq!(r[0].ingredients[0].amount, "2");
        assert_eq!(r[0].servings, 4);
    }
}
