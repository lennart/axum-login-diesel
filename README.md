# axum-login integration for diesel

integrate [diesel](https://github.com/diesel-rs/diesel) with
[axum-login](https://github.com/maxcountryman/axum-login)

## references and code snippets

various parts from gitter and stack overflow, potentially integrated in this repo.

generics with diesel, https://stackoverflow.com/a/67660978

not quite, but almost:

(https://matrix.to/#/!lNGJpfiFVovXFJYmwx:matrix.org/$167545602134687zjLNx:matrix.org?via=mozilla.org&via=gitter.im&via=matrix.org)

```rust
pub trait AuthLoginService: WithDatabaseConnection {
    type UserType: WithUser<User> + Selectable<Mysql> + Default;

    fn query<'a>(&self, username: &'a str) -> UsersBoxedQuery<'a> {
        users::table.filter(users::username.eq(username)).into_boxed()
    }

    fn auth_login<'a>(&self, username: &'a str, password: &str) -> ServiceResult<Self::UserType>
    where
        <Self::UserType as Selectable<Mysql>>::SelectExpression: QueryId,
        users::BoxedQuery<'a, Mysql>:
            diesel::query_dsl::methods::SelectDsl<diesel::dsl::AsSelect<Self::UserType, Mysql>>,
        for<'query> diesel::dsl::Select<users::BoxedQuery<'a, Mysql>, diesel::dsl::AsSelect<Self::UserType, Mysql>>:
            diesel::query_dsl::LoadQuery<'query, MysqlConnection, Self::UserType>,
    {
        let query = self.query(username);

        let mut conn = self.database()?;

        let user = query
            .limit(1)
            .select(Self::UserType::as_select())
            .get_result(&mut conn)
            .optional()
            .context("database query failed")?;

        let mut result = Ok(());

        let user = match user {
            Some(user) => user,
            None => {
                result = Err(ServiceError::AuthenticationFailed)
                    .with_context(|| format!("the user {} does not exist", username));
                Self::UserType::default()
            },
        };

        let updated_hash = match compare_password(password, &user.user().password, true) {
            Ok(update_hash) => {
                result?;
                update_hash
            },
            Err(err) => {
                result?;
                return Err(ServiceError::AuthenticationFailed).context(err);
            },
        };

        if let Some(updated_hash) = updated_hash {
            let rows_affected = diesel::update(users::table)
                .set(users::password.eq(updated_hash))
                .filter(users::username.eq(username))
                .execute(&mut conn)?;
            if rows_affected != 1 {
                warn!(
                    "db password hash update unexpectedly affected {} rows for user {}",
                    rows_affected, username
                );
            } else {
                info!("db password hash updated for user {}", username);
            }
        }

        drop(conn);

        self.check(username, &user)?;
        self.validate(username, &user)?;

        Ok(user)
    }

    fn check<'a>(&self, username: &'a str, user: &Self::UserType) -> ServiceResult<()>;

    fn validate<'a>(&self, _username: &'a str, _user: &Self::UserType) -> ServiceResult<()>;
}
 
```

thread: https://matrix.to/#/!lNGJpfiFVovXFJYmwx:matrix.org/$16650481081013TYYcO:gitter.im?via=mozilla.org&via=gitter.im&via=matrix.org

```rust

fn get_by_ids<R, T, PK>(conn: &mut PgConnection, ids: Vec<PK>) -> Result<Vec<R>, Error>
where
    R: HasTable<Table = T>,
    T: Table,
    T::PrimaryKey: ExpressionMethods,
    diesel::dsl::SqlTypeOf<T::PrimaryKey>: SqlType,
    PK: AsExpression<diesel::dsl::SqlTypeOf<T::PrimaryKey>>,
    T: FilterDsl<EqAny<<T as Table>::PrimaryKey, Vec<PK>>, Output = Q>,
    Filter<T, EqAny<T::PrimaryKey, PK>>: for<'a> LoadQuery<'a, PgConnection, R>,
{
    let predicate = R::table().primary_key().eq_any(ids);
    let table: T = R::table();
    let query: Filter<T, EqAny<T::PrimaryKey, Vec<PK>>> = R::table().filter(predicate);
    query.get_results(conn)
}

// fixing trait bounds issues
fn get_by_ids<R, T, PK, Q>(conn: &mut PgConnection, ids: Vec<PK>) -> Result<Vec<R>, Error>
where
    R: HasTable<Table = T>,
    T: Table,
    T::PrimaryKey: ExpressionMethods,
    diesel::dsl::SqlTypeOf<T::PrimaryKey>: SqlType,
    PK: AsExpression<diesel::dsl::SqlTypeOf<T::PrimaryKey>>,
    T: FilterDsl<EqAny<<T as Table>::PrimaryKey, Vec<PK>>, Output = Q>,
    Q: for<'a> LoadQuery<'a, PgConnection, R>,
{
    let predicate = R::table().primary_key().eq_any(ids);
    let table: T = R::table();
    let query: Filter<T, EqAny<T::PrimaryKey, Vec<PK>>> = R::table().filter(predicate);
    query.get_results(conn)
}
```
