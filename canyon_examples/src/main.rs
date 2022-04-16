use canyon_sql::*;
// use chrono::NaiveDate;
pub mod leagues;
pub mod tournaments;

use leagues::*;
// use tournaments::*;

/// The entry point of a Canyon managed program.
/// 
/// Go read the oficial docs for more info about the `#[canyon]` annotation (not ready doc yet)
/// 
/// TODO Docs explaining the virtues of `#[canyon]`, the `full managed state`
/// and the `just Crud operations` option
///  
#[canyon]  // TODO Add a log level argument
fn main() {
    /*  
        The insert example.
        On the first run, you may desire to uncomment the method call below,
        to be able to populate some data into the schema.
        Remember that all operation with CanyonCrud must be awaited,
        due to it's inherent async nature
    */
    // wire_data_on_schema().await;


    let all_leagues: Vec<Leagues> = Leagues::find_all().await;
    // println!("Leagues elements: {:?}", &all_leagues);

    let all_leagues_as_querybuilder = Leagues::find_all_query()
        .where_clause(LeaguesFields::id(1), Comp::Eq)
        .query()
        .await;
    // println!("Leagues elements QUERYBUILDER: {:?}", &all_leagues_as_querybuilder);

}

/// Example of usage of the `.insert()` Crud operation. Also, allows you
/// to wire some data on the database to be able to retrieve and play with data 
/// 
/// Notice how the `fn` must be `async`, due to Canyon's usage of **tokio**
/// as it's runtime
/// 
/// One big important note on Canyon insert. Canyon automatically manages
/// the ID field (commonly the primary key of any table) for you.
/// This means that if you keep calling this method, Canyon will keep inserting
/// records on the database, not with the id on the instance, only with the 
/// autogenerated one. 
/// 
/// This may change on a nearly time. 'cause it's direct implications on the
/// data integrity, but for now keep an eye on this.
/// 
/// An example of multiples inserts ignoring the provided `id` could end on a
/// situation like this:
/// 
/// ```
/// ... Leagues { id: 43, ext_id: 1, slug: "LEC", name: "League Europe Champions", region: "EU West", image_url: "https://lec.eu" }, 
/// Leagues { id: 44, ext_id: 2, slug: "LCK", name: "League Champions Korea", region: "South Korea", image_url: "https://korean_lck.kr" }, 
/// Leagues { id: 45, ext_id: 1, slug: "LEC", name: "League Europe Champions", region: "EU West", image_url: "https://lec.eu" }, 
/// Leagues { id: 46, ext_id: 2, slug: "LCK", name: "League Champions Korea", region: "South Korea", image_url: "https://korean_lck.kr" } ...
/// ``` 
async fn wire_data_on_schema() {
    // Data for the examples
    let lec: Leagues = Leagues {
        id: 1,
        ext_id: 1,
        slug: "LEC".to_string(),
        name: "League Europe Champions".to_string(),
        region: "EU West".to_string(),
        image_url: "https://lec.eu".to_string(),
    };

    let lck: Leagues = Leagues {
        id: 2,
        ext_id: 2,
        slug: "LCK".to_string(),
        name: "League Champions Korea".to_string(),
        region: "South Korea".to_string(),
        image_url: "https://korean_lck.kr".to_string(),
    };

    // Now, the insert operations in Canyon is designed as a method over
    // the object, so the data of the instance is automatically parsed
    // into it's correct types and formats and inserted into the table
    lec.insert().await;
    lck.insert().await;

    /*  At some point on the console, if the operation it's successful, 
        you must see something similar to this, depending on the logging
        level choosed on Canyon
        
        INSERT STMT: INSERT INTO leagues (ext_id, slug, name, region, image_url) VALUES ($1,$2,$3,$4,$5)
        FIELDS: id, ext_id, slug, name, region, image_url

        INSERT STMT: INSERT INTO leagues (ext_id, slug, name, region, image_url) VALUES ($1,$2,$3,$4,$5)
        FIELDS: id, ext_id, slug, name, region, image_url
    */
}

/// Example of usage of a search by a given Foreign Key
/// TODO Example not ready yet
async fn search_data_by_fk_example() {
    // Data for the examples
    // let lec: Leagues = Leagues {
    //     id: 1,
    //     ext_id: 1,
    //     slug: "LEC".to_string(),
    //     name: "League Europe Champions".to_string(),
    //     region: "EU West".to_string(),
    //     image_url: "https://lec.eu".to_string(),
    // };

    // let tournament_test = Tournaments {
    //         id: 1,
    //         ext_id: 1, 
    //         slug: "slug".to_string(),
    //         // start_date: NaiveDate::from_ymd(2015, 3, 14), 
    //         // end_date: NaiveDate::from_ymd(2015, 3, 14),
    //         league: 1
    // };

    // let tests_foreign = Tournaments::search_by_fk(&league_test).await;
    // println!("TestForeign elements FK: {:?}", &tests_foreign);
}