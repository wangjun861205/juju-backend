table! {
    answers (id) {
        id -> Int4,
        user_id -> Int4,
        option_id -> Int4,
    }
}

table! {
    date_ranges (id) {
        id -> Int4,
        range_ -> Daterange,
        vote_id -> Int4,
        user_id -> Int4,
    }
}

table! {
    dates (id) {
        id -> Int4,
        date_ -> Date,
        user_id -> Int4,
        vote_id -> Int4,
    }
}

table! {
    invite_codes (id) {
        id -> Int4,
        code -> Varchar,
    }
}

table! {
    options (id) {
        id -> Int4,
        option -> Varchar,
        question_id -> Int4,
    }
}

table! {
    organizations (id) {
        id -> Int4,
        name -> Varchar,
        version -> Int8,
    }
}

table! {
    question_update_marks (id) {
        id -> Int4,
        question_id -> Int4,
        user_id -> Int4,
        has_updated -> Bool,
    }
}

table! {
    questions (id) {
        id -> Int4,
        description -> Varchar,
        vote_id -> Int4,
        type_ -> crate::models::QuestionTypeMapping,
        version -> Int8,
    }
}

table! {
    users (id) {
        id -> Int4,
        nickname -> Varchar,
        phone -> Varchar,
        email -> Varchar,
        password -> Varchar,
        salt -> Varchar,
    }
}

table! {
    users_organizations (id) {
        id -> Int4,
        user_id -> Int4,
        organization_id -> Int4,
    }
}

table! {
    vote_update_marks (id) {
        id -> Int4,
        vote_id -> Int4,
        user_id -> Int4,
        has_updated -> Bool,
    }
}

table! {
    votes (id) {
        id -> Int4,
        name -> Varchar,
        deadline -> Nullable<Date>,
        organization_id -> Int4,
        version -> Int8,
    }
}

joinable!(answers -> options (option_id));
joinable!(answers -> users (user_id));
joinable!(date_ranges -> users (user_id));
joinable!(date_ranges -> votes (vote_id));
joinable!(dates -> users (user_id));
joinable!(dates -> votes (vote_id));
joinable!(options -> questions (question_id));
joinable!(question_update_marks -> questions (question_id));
joinable!(question_update_marks -> users (user_id));
joinable!(questions -> votes (vote_id));
joinable!(users_organizations -> organizations (organization_id));
joinable!(users_organizations -> users (user_id));
joinable!(vote_update_marks -> users (user_id));
joinable!(vote_update_marks -> votes (vote_id));
joinable!(votes -> organizations (organization_id));

allow_tables_to_appear_in_same_query!(
    answers,
    date_ranges,
    dates,
    invite_codes,
    options,
    organizations,
    question_update_marks,
    questions,
    users,
    users_organizations,
    vote_update_marks,
    votes,
);
