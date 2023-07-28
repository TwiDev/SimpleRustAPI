use database::UserData;
use rocket::{get, http::Status, serde::json::Json};
use serde::{Serialize};


#[macro_use]
extern crate rocket;

#[derive(Serialize, Debug)]

pub struct GenericResponse {
    pub status: String,
    pub message: String,
}


#[derive(Serialize, Debug)]
pub struct UserResponse {
    pub status: String,
    pub message: UserData
}

#[get("/users/<id>")]
async fn get_user_handler(id: i32) -> Result<Json<UserResponse>, Status> {
    if id == 1 {
        let user:UserData = database::get_user(id).await;

        Ok(Json(UserResponse {
            status: "success".to_string(),
            message: user
        }))


    } else {
        Err(Status::NotFound)
    }
}

#[catch(404)] 
fn not_found() -> Json<GenericResponse> {
    Json(GenericResponse {
        status: "error".to_string(),
        message: "Resource was not found".to_string(),
    })
}

#[launch]
fn rocket() -> _ {
    database::initialize();

    rocket::build().mount("/", routes![get_user_handler]).register("/", catchers![not_found])
}


pub mod database {
    use mysql::{*, prelude::Queryable};
    use once_cell::sync::OnceCell;
    use serde::{Serialize};

    const URL: &str = "mysql://root:root@localhost:3306/core";
    static DB_POOL: OnceCell<Pool> = OnceCell::new();
    static DATABASE_CLIENT: DatabaseClient = DatabaseClient {};

    
    #[derive(Clone, Copy)]
    pub struct DatabaseClient;
    
    impl DatabaseClient {
        pub async fn database_pool(self) -> &'static Pool {
            DB_POOL.get().unwrap()
        }

        pub async fn database_conn(self) -> PooledConn {
            let pool: &Pool  = self.database_pool().await;
            pool.get_conn().unwrap()
        }
    }
    
   
    pub fn initialize() {
        DB_POOL.set(Pool::new(URL).unwrap()).unwrap();
    }

    #[derive(Serialize, Debug, PartialEq, Eq)]
    pub struct UserData {
        pub id: i32,
        pub name: String,
        pub power: i32,
    }

    pub async fn get_user(id: i32) -> UserData{
        let mut _conn: PooledConn = DATABASE_CLIENT.database_conn().await;

        let _query: String = format!("SELECT * FROM users WHERE id = {}", id);
        
        _conn.query_map(_query, |(id, name, power)| {
            UserData { id, name, power }
        }).unwrap().pop().unwrap()
    }


}