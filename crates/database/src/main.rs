pub mod entities;

use database::DatabaseManager;
use entities::{prelude::*, *};
use sea_orm::entity::prelude::*;
use sea_orm::{Set, NotSet};
use sea_orm::*;
use sea_orm::{ConnectionTrait, Database, DbBackend, DbErr, Statement};


//命令
//sqlite://D:/project/star-river-backend/crates/database/db/db.sqlite
// sea-orm-cli migrate refresh -d ./crates/migration
// sea-orm-cli generate entity -u sqlite:///D:/project/star-river-backend/crates/database/db/db.sqlite -o ./crates/database/src/entity
//sea-orm-cli generate entity -o ./crates/entity
//sea-orm-cli generate entity -u sqlite://D:/project/star-river-backend/crates/database/db/db.sqlite




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database = DatabaseManager::new().await;
    println!("database: {:?}", database.path);

    database.migrate().await;

    let db = database.conn;

    let happy_bakery = bakery::ActiveModel {
        name: ActiveValue::Set("Happy Bakery".to_owned()),
        profit_margin: ActiveValue::Set(0.0),
        ..Default::default()
    };

    let res = Bakery::insert(happy_bakery).exec(&db).await?;
    println!("res: {:?}", res);
    let sad_bakery = bakery::ActiveModel {
        id: ActiveValue::Set(res.last_insert_id),
        name: ActiveValue::Set("Sad Bakery".to_owned()),
        profit_margin: ActiveValue::NotSet,
    };
    sad_bakery.update(&db).await?;



    


    Ok(())
}
