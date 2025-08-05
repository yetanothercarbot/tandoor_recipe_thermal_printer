use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TandoorRecipe {
    id: u32,
    name: String,
    description: String,
    steps: Vec<TandoorStep>
}

#[derive(Serialize, Deserialize)]
struct TandoorStep {
    instruction: String,
    ingredients: Vec<TandoorIngredient>
}

#[derive(Serialize, Deserialize)]
struct TandoorIngredient {
    id: u32,
    amount: Option<f32>,
    unit: Option<TandoorUnit>,
    food: TandoorFood,
    note: String
}
#[derive(Serialize, Deserialize)]
struct TandoorFood {

}

#[derive(Serialize, Deserialize)]
struct TandoorUnit {

}