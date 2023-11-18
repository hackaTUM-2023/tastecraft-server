use crate::models::recipes::Recipe;
use sqlx::PgPool;
use crate::services::openai;

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

pub async fn create_motified_recipe(
    db: &PgPool,
    recipe_id: i32,
    preferences: Vec<&str>,
) -> Result<Recipe, sqlx::Error> {
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

    // TODO generate new recipe based on preferences -> AI call
    let new_recipe = openai::send_request(&recipe, &preferences).await;
    if new_recipe.is_err() {
        return Err(sqlx::Error::RowNotFound);
    }

    let new_recipe = new_recipe.unwrap();
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
    .fetch_one(db)
    .await?;

    // create preference mapping
    for preference in preferences {
        sqlx::query!(
            "
            INSERT INTO recipe_preferences (recipe_fk, preference_fk)
            VALUES ($1,
                (SELECT id FROM preferences WHERE name = $2)
            )
            ",
            new_recipe.id,
            preference
        )
        .execute(db)
        .await?;
    }

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
        .execute(db)
        .await?;
    } else {
        // if recipe is not original, original_fk = original_fk in mapping of recipe
        sqlx::query!(
            "
            INSERT INTO variations (original_fk, variation_fk)
            VALUES (
                (SELECT original_fk FROM variations WHERE variation_fk = $1),
                $2
            )
            ",
            recipe.id,
            new_recipe.id
        )
        .execute(db)
        .await?;
    }

    Ok(new_recipe)
}
