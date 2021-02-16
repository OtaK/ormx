use crate::User;
use sqlx::MySqlPool;

pub(crate) async fn query_users(
    db: &MySqlPool,
    filter: Option<&str>,
    limit: Option<usize>,
) -> anyhow::Result<Vec<User>> {
    let result = ormx::conditional_query_as!(
        User,
        r#"SELECT id AS user_id, uuid, first_name, last_name, email, disabled, role AS "role: _", last_login"#
        "FROM users"
        Some(f) = filter => {
            "WHERE first_name LIKE" ?(f)
            "OR last_name LIKE" ?(f)
        }
        "ORDER BY first_name DESC"
        Some(l) = limit => {
            "LIMIT" ?(l as i64)
        }
    )
    .fetch_all(db)
    .await?;

    Ok(result)
}
