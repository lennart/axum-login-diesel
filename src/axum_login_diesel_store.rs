use axum_login::{UserStore, AuthUser};
use diesel::Identifiable;
use diesel::query_dsl::methods::FindDsl;
use diesel::r2d2::Pool;
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use axum::async_trait;
use diesel::pg::Pg;
use core::marker::PhantomData;
use diesel::Table;
use diesel::query_builder::QueryId;
use eyre::Result;
use diesel::associations::HasTable;
use diesel::Selectable;
use diesel::SelectableHelper;
use diesel::query_builder::AsQuery;
use diesel::ExpressionMethods;
use diesel::sql_types::SqlType;
use diesel::expression::AsExpression;
use diesel::query_dsl::LoadQuery;
use diesel::helper_types::Find;
use diesel::query_dsl::methods::LimitDsl;
use diesel::helper_types::Limit;

#[derive(Clone)]
pub struct DieselStore<DB: Sync + Clone, DBTable, PK: Clone + Sync + Send, User, Role = ()>
{
    db: DB,
    //    query: String,
    _pk_type: PhantomData<PK>,
    _table_type: PhantomData<DBTable>,
    _user_type: PhantomData<User>,
    _role_type: PhantomData<Role>,
}

impl<DB: Sync + Clone, DBTable: Clone + Send + Sync, PK: Clone + Sync + Send, User, Role> DieselStore<DB, DBTable, PK, User, Role> {
    pub fn new(db: DB) -> Self {
        Self {
            db,
            _pk_type: Default::default(),
            _table_type: Default::default(),
            _user_type: Default::default(),
            _role_type: Default::default(),
        }
    }
}

pub type PostgresStore<DBTable, PK, User, Role = ()> = DieselStore<Pool<ConnectionManager<PgConnection>>, DBTable, PK, User, Role>;

#[async_trait]
impl<'a, UserId: Sync + Copy, T: Clone + Send + Sync + 'static, PK: Clone + Sync + Send + 'static, User, Role> UserStore<UserId, Role> for PostgresStore<T, PK, User, Role>
where
    // R: HasTable<Table = T>,
    // T: Table,
    // T::PrimaryKey: ExpressionMethods,
    // diesel::dsl::SqlTypeOf<T::PrimaryKey>: SqlType,
    // PK: AsExpression<diesel::dsl::SqlTypeOf<T::PrimaryKey>>,
    // T: FilterDsl<EqAny<<T as Table>::PrimaryKey, Vec<PK>>, Output = Q>,
    // Q: for<'a> LoadQuery<'a, PgConnection, R>,


//    R: HasTable<Table = T>,
T: Table + FindDsl<UserId>,
Find<T, UserId>: LimitDsl + Table,
for<'query> Limit<Find<T, UserId>>: LoadQuery<'query, PgConnection, User>,
 //    T: Table + Clone + Send + Sync + 'static + diesel::query_dsl::methods::FindDsl<UserId>,
 // <T as FindDsl<UserId>>::Output: diesel::Identifiable + Table,
 //    T::PrimaryKey: ExpressionMethods,
 //    diesel::dsl::SqlTypeOf<T::PrimaryKey>: SqlType,
 //    PK: AsExpression<diesel::dsl::SqlTypeOf<T::PrimaryKey>>,
//    T: FilterDsl<EqAny<<T as Table>::PrimaryKey, Vec<PK>>, Output = Q>,
//    Filter<T, EqAny<T::PrimaryKey, PK>>: for<'a> LoadQuery<'a, PgConnection, R>,


//     T: ,
     Role: PartialOrd + PartialEq + Clone + Send + Sync + 'static,
    User: HasTable<Table = T> + AuthUser<UserId, Role> + Selectable<Pg>,
// <User as Selectable<Pg>>::SelectExpression: QueryId,
//     super::schema::users::BoxedQuery<'a, Pg>:
//     diesel::query_dsl::methods::SelectDsl<diesel::dsl::AsSelect<User, Pg>>,
// for<'query> diesel::dsl::Select<
//     super::schema::users::BoxedQuery<'a, Pg>,
//     diesel::dsl::AsSelect<User, Pg>,
//     >: diesel::query_dsl::LoadQuery<'query, PgConnection, User>,
{
    type User = User;

    async fn load_user(&self, user_id: &UserId) -> Result<Option<Self::User>> {
        use super::schema::users::dsl::*;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;
        let connection = &mut self.db.get().unwrap();

        let user : User = User::table()
            .find(*user_id)
          //  .into_boxed()
          //  .select(User::as_select())
            .first(connection)
            .expect("Uh, oh, cannot load users");

        
        //         let user : User = connection
        //             .interact(|conn| {
        //                 users::table.find(1)
        //                 // FilterDsl::filter(users::table,
        //                 //                   users::name.eq(
        //                 //                       "foo"
        //                 //                      // user_id.as_expression()
                                     
        //                 //                   ))
        // //                    .into_boxed()
        // //                    .limit(1)
        //   //                  .select(User::as_select())
        //                     .get_result(conn)
        //                 // FindDsl::find(users::dsl::users,
        //                 //     1
        //                 // //    user_id.as_expression().assume_not_null()
        //                 // ).get_result::<User>(conn)
        //             })
        //              .await??;

        Ok(Some(user))
    }
}

