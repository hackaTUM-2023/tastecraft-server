use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Recipe {
    pub id: Option<i32>,
    pub title: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub preptime: Option<i32>,
    pub difficulty: Option<i32>,
    pub isoriginal: bool,
    //pub ingredients: Vec<Ingredient>,
}