use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TandoorRecipe {
    pub id: u32,
    pub name: String,
    pub working_time: Option<u32>,
    pub waiting_time: Option<u32>,
    pub servings: Option<u32>,
    pub servings_text: Option<String>,
    pub steps: Vec<TandoorStep>
}

#[derive(Serialize, Deserialize)]
pub struct TandoorStep {
    pub instruction: String,
    pub ingredients: Vec<TandoorIngredient>,
    pub name: String,
    pub time: u32,
    pub step_recipe_data: Option<TandoorRecipe>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TandoorIngredient {
    id: u32,
    pub amount: f64,
    pub unit: Option<TandoorUnit>,
    pub food: Option<TandoorFood>,
    note: String
}
#[derive(Clone, Serialize, Deserialize)]
pub struct TandoorFood {
    pub id: u32,
    pub name: String,
    pub plural_name: Option<String>
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct TandoorUnit {
    pub id: u32,
    pub name: String,
    pub plural_name: Option<String>
}

impl TandoorRecipe {
    pub(crate) fn get_all_ingredients(&self) -> Vec<TandoorIngredient> {
        let mut ingredients: Vec<TandoorIngredient> = Vec::new();

        for step in &self.steps {
            for ingredient in &step.ingredients {
                ingredients.push(ingredient.clone());
            }
        }

        ingredients.sort_by_key(|x| x.food.as_ref().unwrap().id);
        ingredients
    }

    pub(crate) fn get_servings(&self) -> String {
        let mut output = String::new();
        if self.servings.is_some() {
            output.push_str(&format!("{} ", self.servings.unwrap()));
            if self.servings_text.as_ref().is_none() || self.servings_text.as_ref().unwrap().is_empty() {
                output.push_str("servings");
            } else {
                output.push_str(&format!("{}", self.servings_text.as_ref().unwrap()));
            }
        }
        // format!("{} {}", self.servings, if self.servings_text.is_none() || self.servings_text.as_ref().unwrap().is_empty() {"servings"} else {self.servings_text.as_ref().unwrap()})
        output
    }

    pub(crate) fn get_duration(&self) -> String {
        let mut output = String::new();

        if self.working_time.is_some() && self.working_time.unwrap() > 0 {
            output.push_str(&format!("{} min working time\n", self.working_time.unwrap()));
        }

        if self.waiting_time.is_some() && self.waiting_time.unwrap() > 0 {
            output.push_str(&format!("{} min waiting time\n", self.waiting_time.unwrap()));
        }

        output
    }
}

impl TandoorStep {
    pub(crate) fn instruction(&self) -> String {
        let mut output = String::new();
        if self.step_recipe_data.is_some() {
            let subrecipe_name = &self.step_recipe_data.as_ref().unwrap().name;
            let subrecipe_id = &self.step_recipe_data.as_ref().unwrap().id;
            output.push_str(&format!("Create recipe \"{}\". ", subrecipe_name));
            println!("Note: Recipe depends on other recipe ({subrecipe_name} - ID: {subrecipe_id})");
        }
        output.push_str(&self.instruction);

        output
    }
}

impl TandoorIngredient {

    pub(crate) fn include(&self) -> bool {
        self.food.is_some()
    }
    pub(crate) fn pretty_print(&self) -> String {
        let mut output = String::from("- ");

        // Amount
        if self.amount > 0.0 {
            let leftover = (self.amount * 100f64).round() as u64 % 100;

            let rounded: String;
            if self.amount >= 1f64 {
                rounded = format!("{} ", self.amount.round())
            } else {
                rounded = String::new();
            }

            match leftover {
                13 => output.push_str(&format!("{}1/8 ", rounded)),
                25 => output.push_str(&format!("{}1/4 ", rounded)),
                33 => output.push_str(&format!("{}1/3 ", rounded)),
                50 => output.push_str(&format!("{}1/2 ", rounded)),
                67 => output.push_str(&format!("{}2/3 ", rounded)),
                75 => output.push_str(&format!("{}3/4 ", rounded)),
                _ => output.push_str(&format!("{} ", self.amount)),
            }
        }

        // Unit
        if self.unit.is_some() && self.amount > 0.0 {
            let unit = self.unit.as_ref().unwrap();
            if unit.plural_name.is_some() && (self.amount - 1.0) > f64::EPSILON {
                output.push_str(&format!("{} ", unit.plural_name.as_ref().unwrap()));
            } else {
                output.push_str(&format!("{} ", unit.name));
            }
        }

        // Name
        if self.food.is_some() {
            let food = self.food.as_ref().unwrap();
            if food.plural_name.is_some() && (self.amount - 1.0) > f64::EPSILON {
                output.push_str(&format!("{}", food.plural_name.as_ref().unwrap()));
            } else {
                output.push_str(&format!("{}", food.name));
            }
        }


        // Note
        if !self.note.is_empty() {
            output.push_str(&format!(" ({})", self.note));
        }

        output
    }
}