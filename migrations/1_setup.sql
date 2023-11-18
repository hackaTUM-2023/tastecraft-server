CREATE TABLE recipes(
    id serial primary key,
    title text not null,
    description text,
    method text,
    preptime int,
    difficulty int,
    isoriginal boolean
);

CREATE TABLE variations(
    originalid int not null,
    variationid int not null,

    PRIMARY KEY (originalid, variationid),
    CONSTRAINT fk_originalid FOREIGN KEY (originalid) REFERENCES recipes(id) on delete cascade,
    CONSTRAINT fk_variationid FOREIGN KEY (variationid) REFERENCES recipes(id) on delete cascade
);

CREATE TABLE preferences(
    id serial primary key,
    name text not null
);