use database::UserData;
use rocket::{get, http::Status, serde::json::Json};
use serde::{Serialize};
use rocket::http::Header;
use rocket::{Request, Response, response::status::Custom};
use rocket::fairing::{Fairing, Info, Kind};


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


#[derive(Serialize, Debug)]
pub struct PostUserResponse {
    pub status: String,
    pub message: String,
    pub user_id: u64
}

#[get("/users/<id>")]
async fn get_user_handler(id: i32) -> Result<Json<UserResponse>, Status> {
    let optional_user:Option<UserData> = database::get_user(id).await;

    if optional_user.is_some() {
        let user: UserData = optional_user.unwrap();

        Ok(Json(UserResponse {
            status: "success".to_string(),
            message: user
        }))


    } else {
        Err(Status::NotFound)
    }
}

#[post("/users", data="<body>")]
async fn post_user_handler(mut body: String) -> Result<Json<PostUserResponse>, Custom<Json<GenericResponse>>> {
    println!("{:?}", body);
    let user:u64 = database::create_user(body, 0).await.expect("Error creating user");

    Ok(Json(PostUserResponse{
        status: "success".to_string(),
        message: "User created".to_string(),
        user_id: user
    }))
}

#[catch(404)] 
fn not_found() -> Json<GenericResponse> {
    Json(GenericResponse {
        status: "error".to_string(),
        message: "Resource was not found".to_string(),
    })
}

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[launch]
fn rocket() -> _ {
    database::initialize();

    rocket::build().mount("/", routes![get_user_handler, post_user_handler]).register("/", catchers![not_found]).attach(CORS)
}


pub mod database {
    use mysql::{*, prelude::Queryable};
    use once_cell::sync::OnceCell;
    use serde::{Serialize,Deserialize};

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

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    pub struct UserData {
        pub id: i32,
        pub name: String,
        pub power: i32,
    }

    pub async fn get_user(id: i32) -> Option<UserData> {
        let mut _conn: PooledConn = DATABASE_CLIENT.database_conn().await;

        let _query: String = format!("SELECT * FROM users WHERE id = {}", id);
        
        _conn.query_map(_query, |(id, name, power)| {
            UserData { id, name, power }
        }).unwrap().pop()
    }

    pub async fn create_user(name: String,power:i32) -> std::result::Result<u64, mysql::error::Error>{
        let mut _conn: PooledConn = DATABASE_CLIENT.database_conn().await;

        let _query: String = format!("INSERT INTO users (name, power) VALUES (:name, :power)");
        
        _conn.exec_drop(_query, 
            params! {
            "name" => name, 
            "power" => power,
            },
        ).and_then(|_| 
            Ok(_conn.last_insert_id()))

    }


}