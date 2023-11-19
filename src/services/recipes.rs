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
                "
        SELECT * FROM recipes
        WHERE title LIKE $1
        ",
                format!("%{search_text}%")
            )
                .fetch_all(db)
                .await?
        }
        None => {
            sqlx::query_as!(
                Recipe,
                "
        SELECT * FROM recipes
        "
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
    // 1. get original of that recipe
    // 2. for original and all variations, get preferences
    let preference_ids: HashSet<i32> = sqlx::query!(
        r#"SELECT id FROM preferences WHERE name = ANY($1)"#,
        &preferences
    )
    .map(|row| row.id as i32)
    .fetch_all(db)
    .await?
    .into_iter()
    .collect::<HashSet<_>>();

    // Get all variations for original recipe
    let variations_for_recipe: Vec<i32> = sqlx::query!(
        r#"SELECT variation_fk FROM variations WHERE original_fk = $1"#,
        recipe_id
    ).map(|row| row.variation_fk as i32)
    .fetch_all(db).await?;

    for v in variations_for_recipe {
        // Get all preferences for the recipe
        let recipe_pref: HashSet<i32> = sqlx::query!(
            r#"SELECT preference_fk FROM recipe_preferences WHERE recipe_fk = $1"#,
            v
        ).map(|row| row.preference_fk as i32)
        .fetch_all(db).await?
        .into_iter()
        .collect::<HashSet<_>>();

        // If the recipe has the same preferences as the user, return it
        if recipe_pref == preference_ids {
            return Ok(sqlx::query_as!(
                Recipe,
                "
                SELECT * FROM recipes
                WHERE id = $1
                ",
                v
            )
                .fetch_one(db)
                .await?);
        }
    }


    Ok(create_modified_recipe(db, recipe_id, preferences).await?)
}

async fn create_modified_recipe(
    db: &PgPool,
    recipe_id: i32,
    preferences: &[String],
) -> Result<Recipe> {

    // load recipe to id
    let recipe = sqlx::query_as!(
        Recipe,
        "
        SELECT * FROM recipes
        WHERE id = $1
        ",
        recipe_id
    )
        .fetch_one(db)
        .await?;

    // generate new recipe based on preferences -> AI call
    let new_recipe = openai::send_request(&recipe, &preferences).await?;

    let mut tx = db.begin().await?;

    // create new recipe entry in database
    let new_recipe = sqlx::query_as!(
        Recipe,
        "
        INSERT INTO recipes (title, description, instructions, preptime, difficulty, isoriginal)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        ",
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
            "
            INSERT INTO recipe_preferences (recipe_fk, preference_fk)
            SELECT $1, id FROM preferences WHERE name = ANY($2)
            ",
            new_recipe.id,
            preferences
        )
        .execute(&mut *tx)
        .await?;

    // create recipe mapping to original mapping
    if recipe.isoriginal {
        // if recipe is original, original_fk = recipe.id
        sqlx::query!(
            "
            INSERT INTO variations (original_fk, variation_fk)
            VALUES ($1, $2)
            ",
            recipe.id,
            new_recipe.id
        )
            .execute(&mut *tx)
            .await?;
    } else {
        // if recipe is not original, original_fk = original_fk in mapping of recipe
        sqlx::query!(
            "
            INSERT INTO variations (original_fk, variation_fk)
            SELECT original_fk, $2 FROM variations WHERE variation_fk = $1
            ",
            recipe.id,
            new_recipe.id
        )
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(new_recipe)
}
