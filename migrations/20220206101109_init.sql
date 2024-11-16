CREATE TABLE questions(
    id INTEGER PRIMARY KEY NOT NULL,
    category INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    question TEXT NOT NULL,
    answer TEXT NOT NULL
);
CREATE TABLE categories(
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);
CREATE TABLE users(
    id INTEGER PRIMARY KEY NOT NULL,
    username TEXT NULL,
    first_name TEXT NOT NULL,
    last_name TEXT NULL
)