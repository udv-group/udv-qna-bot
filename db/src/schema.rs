table! {
    categories (id) {
        id -> Integer,
        name -> Text,
    }
}

table! {
    questions (id) {
        id -> Integer,
        category -> Nullable<Integer>,
        question -> Text,
        answer -> Text,
    }
}

joinable!(questions -> categories (category));

allow_tables_to_appear_in_same_query!(
    categories,
    questions,
);
