use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Recipe {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub preptime: Option<i32>,
    pub difficulty: Option<i32>,
    pub isoriginal: bool,
    //pub ingredients: Vec<Ingredient>,
}