use crate::errors::ServiceError;
use crate::models::{NewUser, User};
use crate::schema::users::dsl::*;
use crate::utils::{hash_password, verify};
use diesel::prelude::*;
use log::{debug, info};

pub fn show_users(conn: &mut PgConnection) -> Vec<User> {
    let results = users
        //.select((users::username, users::password))
        .load::<User>(conn)
        .expect("Error loading users");

    debug!("Getting {} users", results.len());

    return results;
}

pub fn get_user(conn: &mut PgConnection, _username: &str, _password: &str) -> Result<User, String> {
    debug!(
        "Selecting User with username: {:?} password:{:?}",
        _username, _password
    );

    let mut results = users
        .filter(username.eq(_username))
        .limit(1)
        .load::<User>(conn)
        .expect("Error loading users");

    debug!("Query get_user_id {} match:", results.len());
    for user in results.pop() {
        return Ok(user);
    }
    return Err("UserNotFound".to_string());
}

pub fn check_login(
    conn: &mut PgConnection,
    _username: &str,
    _password: &str,
) -> Result<bool, ServiceError> {
    debug!(
        "Selecting User with username: {:?} password:{:?}",
        _username, _password
    );

    let results = users
        .filter(username.eq(_username))
        .load::<User>(conn)
        .expect("Error loading users");

    debug!("Query {} match:", results.len());

    let pwd_hash = hash_password(&_password)?;
    for user in &results {
        if verify(&pwd_hash, &user.password)? {
            info!(
                "Logging in: User {:?} Pwd:{:?} Id:{:?}",
                user.username, user.password, user.id
            );
            return Ok(true);
        }
    }
    return Ok(false);
}

pub fn insert_user(conn: &mut PgConnection, name: &str, pwd: &str) -> Result<User, ServiceError> {
    debug!("Creating new User Name: {:?} pwd: {:?} ", name, pwd);

    let new_user = NewUser {
        username: &name,
        password: &hash_password(&pwd)?,
    };

    Ok(diesel::insert_into(users)
        .values(&new_user)
        .get_result(conn)
        .expect("Error inserting new user"))
}
