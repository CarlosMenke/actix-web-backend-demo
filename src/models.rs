use crate::schema::users;

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

#[derive(Queryable, Debug, AsChangeset)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
}
