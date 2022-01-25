table! {
    answers (id) {
        id -> Int4,
        user_id -> Int4,
        option_id -> Int4,
    }
}

table! {
    dates (id) {
        id -> Int4,
        d -> Date,
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
    }
}

table! {
    questions (id) {
        id -> Int4,
        description -> Varchar,
        vote_id -> Int4,
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
    votes (id) {
        id -> Int4,
        name -> Varchar,
        deadline -> Nullable<Date>,
        status -> crate::models::VoteStatusMapping,
        organization_id -> Int4,
    }
}

joinable!(answers -> options (option_id));
joinable!(answers -> users (user_id));
joinable!(dates -> users (user_id));
joinable!(dates -> votes (vote_id));
joinable!(options -> questions (question_id));
joinable!(questions -> votes (vote_id));
joinable!(users_organizations -> organizations (organization_id));
joinable!(users_organizations -> users (user_id));
joinable!(votes -> organizations (organization_id));

allow_tables_to_appear_in_same_query!(answers, dates, invite_codes, options, organizations, questions, users, users_organizations, votes,);
