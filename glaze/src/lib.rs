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

    use plexus::store::{Result, Row, Store, Value};

    /// The book, kept in a store of its own; a colorant is its number.
    pub struct Glaze {
        store: Store,
    }

    impl Default for Glaze {
        fn default() -> Glaze {
            Glaze {
                store: Store::open(
                    "glaze",
                    "create table if not exists recipe (name text primary key, colorant integer not null, cone integer not null);
                     insert or ignore into recipe values ('celadon', 1, 10), ('oxblood', 2, 10), ('midnight', 3, 6), ('glass', 0, 6);",
                ),
            }
        }
    }

    fn recipe(row: &Row) -> Result<Recipe> {
        let colorant = match row.get::<u8>(1)? {
            0 => Colorant::Clear,
            1 => Colorant::Iron,
            2 => Colorant::Copper,
            _ => Colorant::Cobalt,
        };
        Ok(Recipe { name: row.get(0)?, colorant, cone: row.get(2)? })
    }

    impl Glaze {
        fn book(&self, filter: &str, params: &[Value]) -> Vec<Recipe> {
            self.store.query(&format!("select name, colorant, cone from recipe {filter} order by rowid"), params, recipe).expect("the glaze store reads")
        }

        /// Every recipe in the book.
        pub fn recipes(&self) -> Vec<Recipe> {
            self.book("", &[])
        }

        /// The recipes coloured with one thing.
        pub fn coloured(&self, colorant: Colorant) -> Vec<Recipe> {
            self.book("where colorant = ?1", &[(colorant as u8).into()])
        }

        /// What a batch of a recipe needs.
        pub fn weigh(&self, recipe: String, grams: u32) -> Result<Batch, GlazeError> {
            if grams < 100 {
                return Err(GlazeError::TooSmall(grams));
            }
            let found = self.book("where name = ?1", &[recipe.as_str().into()]).pop().ok_or(GlazeError::Unknown(recipe))?;
            let colorant_grams = if matches!(found.colorant, Colorant::Clear) { 0 } else { grams * 3 / 100 };
            let rest = grams - colorant_grams;
            Ok(Batch { recipe: found.name, silica_grams: rest * 6 / 10, flux_grams: rest - rest * 6 / 10, colorant_grams })
        }
    }
}
