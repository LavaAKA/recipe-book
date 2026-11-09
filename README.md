# Recipe Book

A small desktop app (Rust + egui) for keeping recipes.

- Import recipes by pasting text or opening a `.txt`, `.md` or `.json` file
- Change the number of servings and every amount rescales
- Edit ingredients, steps and notes
- 10 colour themes (5 light, 5 dark)

## Run

```
cargo run --release
```

Recipes are stored in `%APPDATA%\RecipeBook\recipes.json`.
