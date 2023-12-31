use std::collections::HashSet;

use crate::models::recipes::Recipe;
use sqlx::PgPool;
use crate::services::openai;
use anyhow::Result;

pub async fn get_original_recipes(
    db: &PgPool,
    search_text: Option<&str>,
) -> Result<Vec<Recipe>, sqlx::Error> {
    let recipes = match search_text {
        Some(search_text) => {
            sqlx::query_as!(
                Recipe,
                r#"SELECT * FROM recipes WHERE title LIKE $1"#,
                format!("%{search_text}%")
            )
            .fetch_all(db)
            .await?
        }
        None => {
            sqlx::query_as!(
                Recipe,
                r#"SELECT * FROM recipes"#
            )
            .fetch_all(db)
            .await?
        }
    };

    Ok(recipes)
}

struct RecipePreference {
    recipe_id: Option<i32>,
    preferences: Vec<String>,
}

pub async fn get_recipe_by_id(
    db: &PgPool,
    recipe_id: i32,
    preferences: &[String],
) -> Result<Recipe> {
    // fetch original id
    let recipe_id = sqlx::query!(
        r#"SELECT id FROM recipes WHERE id = $1 AND isoriginal=true
        UNION
        SELECT original_fk as id FROM variations WHERE variation_fk = $1"#,
        recipe_id
    )
        .map(|row| row.id)
        .fetch_one(db)
        .await?;
 
    // get all preferences there are
    let preference_ids: HashSet<i32> = sqlx::query!(
        r#"SELECT id FROM preferences WHERE name = ANY($1)"#,
        &preferences
    )
    .map(|row| row.id)
    .fetch_all(db)
    .await?
    .into_iter()
    .collect::<HashSet<_>>();

    // Get all variations for original recipe and original
    let mut variations_for_recipe: Vec<i32> = sqlx::query!(
        r#"SELECT variation_fk FROM variations WHERE original_fk = $1"#,
        recipe_id
    ).map(|row| row.variation_fk)
    .fetch_all(db).await?;

    variations_for_recipe.push(recipe_id.unwrap());
    for v in variations_for_recipe {
        // Get all preferences for the recipe
        let recipe_pref: HashSet<i32> = sqlx::query!(
            r#"SELECT preference_fk FROM recipe_preferences WHERE recipe_fk = $1"#,
            v
        ).map(|row| row.preference_fk)
        .fetch_all(db).await?
        .into_iter()
        .collect::<HashSet<_>>();

        // If the recipe has the same preferences as the user, return it
        if recipe_pref == preference_ids {
            return Ok(sqlx::query_as!(
                Recipe,
                r#"SELECT * FROM recipes WHERE id = $1"#,
                v
            )
                .fetch_one(db)
                .await?);
        }
    }


    create_modified_recipe(db, recipe_id.unwrap(), preferences).await
}

async fn create_modified_recipe(
    db: &PgPool,
    recipe_id: i32,
    preferences: &[String],
) -> Result<Recipe> {

    // load recipe to id
    let recipe = sqlx::query_as!(
        Recipe,
        r#"SELECT * FROM recipes WHERE id = $1"#,
        recipe_id
    )
        .fetch_one(db)
        .await?;

    // generate new recipe based on preferences -> AI call
    let new_recipe = openai::send_request(&recipe, preferences).await?;

    let mut tx = db.begin().await?;

    // create new recipe entry in database
    let new_recipe = sqlx::query_as!(
        Recipe,
        r#"INSERT INTO recipes (title, description, instructions, preptime, difficulty, isoriginal)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
        new_recipe.title,
        new_recipe.description,
        new_recipe.instructions,
        new_recipe.preptime,
        new_recipe.difficulty,
        false
    )
        .fetch_one(&mut *tx)
        .await?;

    // create preference mapping
    println!("preferences: {:?}", preferences);
    sqlx::query!(
            r#"INSERT INTO recipe_preferences (recipe_fk, preference_fk)
            SELECT $1, id FROM preferences WHERE name = ANY($2)"#,
            new_recipe.id,
            preferences
        )
        .execute(&mut *tx)
        .await?;

    // create recipe mapping to original mapping
    if recipe.isoriginal {
        // if recipe is original, original_fk = recipe.id
        sqlx::query!(
            r#"INSERT INTO variations (original_fk, variation_fk)
            VALUES ($1, $2)"#,
            recipe.id,
            new_recipe.id
        )
            .execute(&mut *tx)
            .await?;
    } else {
        // if recipe is not original, original_fk = original_fk in mapping of recipe
        sqlx::query!(
            r#"INSERT INTO variations (original_fk, variation_fk)
            SELECT original_fk, $2 FROM variations WHERE variation_fk = $1"#,
            recipe.id,
            new_recipe.id
        )
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(new_recipe)
}
