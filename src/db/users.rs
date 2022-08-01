use crate::models::{NewUser, User};
use crate::schema::users::dsl::*;
use diesel::prelude::*;
use log::{debug, info};

pub fn show_users(conn: &mut PgConnection) -> Vec<User> {
    let results = users
        //.select((users::username, users::password))
        .load::<User>(conn)
        .expect("Error loading videos");

    debug!("Getting {} users", results.len());

    return results;
}

pub fn get_user_id(
    conn: &mut PgConnection,
    _username: &str,
    _password: &str,
) -> Result<i32, String> {
    debug!(
        "Selecting User with username: {:?} password:{:?}",
        _username, _password
    );

    let results = users
        .filter(username.eq(_username))
        .limit(1)
        .load::<User>(conn)
        .expect("Error loading users");

    debug!("Query get_user_id {} match:", results.len());
    for user in &results {
        return Ok(user.id);
    }
    return Err("UserNotFound".to_string());
}

pub fn check_login(conn: &mut PgConnection, _username: &str, _password: &str) -> bool {
    debug!(
        "Selecting User with username: {:?} password:{:?}",
        _username, _password
    );

    let results = users
        .filter(username.eq(_username))
        .load::<User>(conn)
        .expect("Error loading users");

    debug!("Query {} match:", results.len());

    for user in &results {
        if &user.password == &_password {
            info!(
                "Logging in: User {:?} Pwd:{:?} Id:{:?}",
                user.username, user.password, user.id
            );
            return true;
        }
    }
    return false;
}

pub fn insert_user(conn: &mut PgConnection, name: &str, pwd: &str) -> User {
    debug!("Creating new User Name: {:?} pwd: {:?} ", name, pwd);

    let new_user = NewUser {
        username: &name,
        password: &pwd,
    };

    diesel::insert_into(users)
        .values(&new_user)
        .get_result(conn)
        .expect("Error saving new video")
}
