/// Glaze recipes, and what a batch of one needs.
#[plexus::activation]
pub mod glaze {
    pub enum Colorant {
        Clear,
        Iron,
        Copper,
        Cobalt,
    }

    pub struct Recipe {
        pub name: String,
        pub colorant: Colorant,
        /// The cone it matures at.
        pub cone: u8,
    }

    /// What to weigh out.
    pub struct Batch {
        pub recipe: String,
        pub silica_grams: u32,
        pub flux_grams: u32,
        pub colorant_grams: u32,
    }

    pub enum GlazeError {
        /// No recipe has this name.
        Unknown(String),
        /// A batch needs at least 100 grams.
        TooSmall(u32),
    }

    #[derive(Default)]
    pub struct Glaze;

    fn book() -> Vec<Recipe> {
        let r = |name: &str, colorant, cone| Recipe { name: name.into(), colorant, cone };
        vec![r("celadon", Colorant::Iron, 10), r("oxblood", Colorant::Copper, 10), r("midnight", Colorant::Cobalt, 6), r("glass", Colorant::Clear, 6)]
    }

    impl Glaze {
        /// Every recipe in the book.
        pub fn recipes(&self) -> Vec<Recipe> {
            book()
        }

        /// The recipes coloured with one thing.
        pub fn coloured(&self, colorant: Colorant) -> Vec<Recipe> {
            book().into_iter().filter(|r| r.colorant as u8 == colorant as u8).collect()
        }

        /// What a batch of a recipe needs.
        pub fn weigh(&self, recipe: String, grams: u32) -> Result<Batch, GlazeError> {
            if grams < 100 {
                return Err(GlazeError::TooSmall(grams));
            }
            let found = book().into_iter().find(|r| r.name == recipe).ok_or(GlazeError::Unknown(recipe))?;
            let colorant_grams = if matches!(found.colorant, Colorant::Clear) { 0 } else { grams * 3 / 100 };
            let rest = grams - colorant_grams;
            Ok(Batch { recipe: found.name, silica_grams: rest * 6 / 10, flux_grams: rest - rest * 6 / 10, colorant_grams })
        }
    }
}
