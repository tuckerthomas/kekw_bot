table! {
    periods (id) {
        id -> Integer,
        start_day -> BigInt,
        end_day -> Nullable<BigInt>,
    }
}

table! {
    rolls (id) {
        id -> Integer,
        selection_1 -> Integer,
        selection_2 -> Integer,
        period_id -> Integer,
    }
}

table! {
    submissions (id) {
        id -> Integer,
        dis_user_id -> Text,
        title -> Text,
        link -> Text,
        period_id -> Integer,
    }
}

joinable!(rolls -> periods (period_id));
joinable!(submissions -> periods (period_id));

allow_tables_to_appear_in_same_query!(
    periods,
    rolls,
    submissions,
);
