use crate::models::recipes::Recipe;
use sqlx::PgPool;
use crate::services::openai;
use anyhow::{Context, Result};
use crate::api::PrefParam;

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
    let recipe_pref = sqlx::query_as!(
        RecipePreference,
        "
        WITH original (id) AS (
            SELECT COALESCE(
                (SELECT id FROM recipes WHERE id = $1 AND isoriginal = true),
                (SELECT original_fk FROM variations WHERE variation_fk = $1)
            )
        )

        SELECT variations.variation_fk AS recipe_id, array_agg(preferences.name) AS preferences
        FROM original, variations, recipe_preferences, preferences
        WHERE variations.original_fk = original.id AND variations.variation_fk = recipe_preferences.recipe_fk AND recipe_preferences.preference_fk = preferences.id
        GROUP BY variations.variation_fk
        UNION
        SELECT original.id AS recipe_id, array_agg(preferences.name) AS preferences
        FROM original, recipe_preferences, preferences
        WHERE original.id = recipe_preferences.recipe_fk AND recipe_preferences.preference_fk = preferences.id
        GROUP BY original.id
        ",
        recipe_id
    ).fetch_all(db).await?;


    // check if any preferences are superset of query preferences
    let i = checkRecipe(preferences, recipe_pref).await;
    return if i != -1 {
        // if yes, return recipe
        let recipe = sqlx::query_as!(
            Recipe,
            "
            SELECT * FROM recipes
            WHERE id = $1
            ",
            i
        )
            .fetch_one(db)
            .await?;
        Ok(recipe)
    } else {
        // if not create new recipe
        Ok(create_modified_recipe(db, recipe_id, preferences).await?)
    }
}

async fn checkRecipe(pref_param: &[String], recipe_pref: Vec<RecipePreference>) -> i32 {
    for rp in recipe_pref{
        if checkMatch(pref_param, &rp.preferences).await {
            return rp.recipe_id.unwrap();
        }
    }

    return -1;
}

async fn checkMatch(pref_param: &[String], pref: &[String]) -> bool {
    if pref.len() != pref_param.len() {
        return false;
    }

    for p in pref {
        if !pref_param.contains(&p) {
            return false;
        }
    }

    return true;
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
