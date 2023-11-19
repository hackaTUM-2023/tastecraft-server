CREATE TABLE recipes(
    id serial primary key,
    title text not null,
    description text,
    instructions text,
    preptime int,
    difficulty int,
    isoriginal boolean not null default false
);

CREATE TABLE variations(
    original_fk int not null,
    variation_fk int not null,

    PRIMARY KEY (original_fk, variation_fk),
    CONSTRAINT fk_originalid FOREIGN KEY (original_fk) REFERENCES recipes(id) on delete cascade,
    CONSTRAINT fk_variationid FOREIGN KEY (variation_fk) REFERENCES recipes(id) on delete cascade
);

CREATE TABLE preferences(
    id serial primary key,
    name text not null unique
);

CREATE TABLE recipe_preferences(
    recipe_fk int not null,
    preference_fk int not null,

    PRIMARY KEY (recipe_fk, preference_fk),
    CONSTRAINT fk_recipeid FOREIGN KEY (recipe_fk) REFERENCES recipes(id) on delete cascade,
    CONSTRAINT fk_preferenceid FOREIGN KEY (preference_fk) REFERENCES preferences(id) on delete cascade
);