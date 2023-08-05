use axum_login::{UserStore, AuthUser};
use diesel::RunQueryDsl;
use diesel::query_dsl::methods::FindDsl;
use diesel::r2d2::Pool;
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use axum::async_trait;
use diesel::pg::Pg;
use core::marker::PhantomData;
use diesel::Table;
use eyre::Result;
use diesel::associations::HasTable;
use diesel::Selectable;
use diesel::query_dsl::LoadQuery;
use diesel::helper_types::Find;
use diesel::query_dsl::methods::LimitDsl;
use diesel::helper_types::Limit;

#[derive(Clone)]
pub struct DieselStore<DB: Sync + Clone, User, Role = ()>
{
    db: DB,
    _user_type: PhantomData<User>,
    _role_type: PhantomData<Role>,
}

impl<DB: Sync + Clone, User, Role> DieselStore<DB, User, Role> {
    pub fn new(db: DB) -> Self {
        Self {
            db,
            _user_type: Default::default(),
            _role_type: Default::default(),
        }
    }
}

pub type PostgresStore<User, Role = ()> = DieselStore<Pool<ConnectionManager<PgConnection>>, User, Role>;

#[async_trait]
impl<UserId: Sync + Copy, User, Role> UserStore<UserId, Role> for PostgresStore<User, Role>
where
Find<<User as HasTable>::Table, UserId>: LimitDsl + RunQueryDsl<PgConnection>,
for<'query> Limit<Find<<User as HasTable>::Table, UserId>>: LoadQuery<'query, PgConnection, User>,
     Role: PartialOrd + PartialEq + Clone + Send + Sync + 'static,
<User as HasTable>::Table: Table + FindDsl<UserId>,
    User: HasTable + AuthUser<UserId, Role> + Selectable<Pg>,
{
    type User = User;

    async fn load_user(&self, user_id: &UserId) -> Result<Option<Self::User>> {
        use diesel::RunQueryDsl;
        let connection = &mut self.db.get().unwrap();

        let user : User = User::table()
            .find(*user_id)
            .first(connection)
            .expect("Uh, oh, cannot load users");

        Ok(Some(user))
    }
}
