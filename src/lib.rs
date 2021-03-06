#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate rustc_hex;
extern crate tiny_keccak;
extern crate web3;

use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use diesel::sql_query;

use dotenv::dotenv;

use std::env;
use std::str::FromStr;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::Mutex;

use web3::types::*;
use web3::futures::Future;
use web3::api::EthFilter;
use web3::api::Namespace;
use web3::futures::Stream;

use rustc_hex::ToHex;

use self::models::*;

pub mod schema;
pub mod models;

pub fn current_auto_increment_value_responses(conn: &MysqlConnection) -> Result<i64, String> {
    let response = sql_query("SELECT AUTO_INCREMENT FROM information_schema.TABLES WHERE TABLE_SCHEMA = \"acbidder_database\" AND TABLE_NAME = \"responses\"")
        .get_results::<AutoIncrement>(conn);
    let response = match response {
        Ok(val) => val,
        Err(_) => return Err(format!("AUTO_INCREMENT could not be retrieved from responses.")),
    };
    Ok(response[0].AUTO_INCREMENT)
}

pub fn current_auto_increment_value_requests(conn: &MysqlConnection) -> Result<i64, String> {
    let response = sql_query("SELECT AUTO_INCREMENT FROM information_schema.TABLES WHERE TABLE_SCHEMA = \"acbidder_database\" AND TABLE_NAME = \"requests\"")
        .get_results::<AutoIncrement>(conn);
    let response = match response {
        Ok(val) => val,
        Err(_) => return Err(format!("AUTO_INCREMENT could not be retrieved from responses.")),
    };
    Ok(response[0].AUTO_INCREMENT)
}

//TODO: modify to pass up error vs panicking
pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connection to {}", database_url))
}

pub fn improper_domain_name(name: & str) -> bool {
    for character in name.chars() {
        if !character.is_ascii_alphanumeric() && character != '.' && character != '-' {
            return true;
        }
    }
    false
}

//returns number of listings created
pub fn create_listing<'a>(conn: &MysqlConnection, domain_name: &'a str) -> Result<usize, String> {
    use schema::listings;

    //ensures that no special characters are used and valid domain name characters are used
    if improper_domain_name(domain_name) {
        return Err(format!("Improper domain name {}", domain_name));
    }

    let new_ad_server = NewAdServer {
        domain: domain_name,
    };
    match diesel::insert_into(listings::table)
        .values(&new_ad_server)
        .execute(conn)
    {
        Ok(val) => return Ok(val),
        Err(_) => return Err(format!("Insert for Listing failed.")),
    }
}

pub fn is_whitelisted(conn: &MysqlConnection, domain_name: String) -> Result<bool, String> {
    use schema::listings::dsl::*;

    if improper_domain_name(&domain_name) {
        return Err(format!("Improper domain name {}", domain_name));
    }

    match listings
        .filter(domain.like(domain_name))
        .limit(1)
        .load::<AdServer>(conn)
    {
        Ok(val) => {
            if val.len() == 0 {
                return Ok(false);
            } else if val.len() > 1 {
                return Err(format!("Something impossible happened."));
            } else {
                return Ok(true);
            }
        }
        Err(e) => return Err(format!("Could not check if whitelisted: {}", e)),
    }
}

//delets a listing (ad_server) with the name in domain_name and returns the number of rows deleted
pub fn delete_listing(conn: &MysqlConnection, domain_name: String) -> Result<usize, String> {
    use schema::listings::dsl::*;

    match diesel::delete(listings.filter(domain.like(domain_name))).execute(conn) {
        Ok(val) => return Ok(val),
        Err(_) => return Err(format!("Delete for Listing failed.")),
    }
}

//returns the id number
pub fn create_request<'a>(conn: &MysqlConnection, publisher_name: &'a str, user_quality: i32) -> Result<i32, String> {
    use schema::requests;
    use schema::requests::dsl::*;

    if improper_domain_name(publisher_name) {
        return Err(format!("Improper domain name {}", publisher_name));
    }

    let new_request = NewRequest {
        publisher: publisher_name,
        userquality: user_quality,
    };

    match diesel::insert_into(requests::table)
        .values(&new_request)
        .execute(conn)
    {
            Ok(val) => val,
            Err(_) => return Err(format!("Insert for Request has failed.")),
    };
    let request_inserted = match requests
        .filter(publisher.like(publisher_name))
        .limit(1)
        .load::<Request>(conn)
        {
            Ok(val) => val,
            Err(_) => return Err(format!("Could not retrieve id of newly added Request.")),
        };

    Ok(request_inserted[0].id)
}

//return the id of the last request made or return 0
pub fn get_latest_request_id(conn:&MysqlConnection) -> Result<i32, String> {
    use schema::requests::dsl::*;

    let latest_request = match requests
        .order(id.desc())
        .limit(1)
        .load::<Request>(conn)
        {
            Ok(val) => val,
            Err(_) => return Err(format!("Could not retrieve id for Request.")),
        };
    Ok(latest_request[0].id)
}

//delete a request with publisher_name and returns number of rows deleted
pub fn delete_request(conn: &MysqlConnection, publisher_name: String) -> Result<usize, String> {
    use schema::requests::dsl::*;

    if improper_domain_name(&publisher_name) {
        return Err(format!("Improper domain name {}", publisher_name));
    }

    match diesel::delete(requests.filter(publisher.like(publisher_name))).execute(conn) {
        Ok(val) => return Ok(val),
        Err(_) => return Err(format!("Delete for Request has failed.")),
    }
}


//returns the id number
pub fn create_response<'a>(conn: &MysqlConnection, publisher_name: &'a str) -> Result<i32, String> {
    use schema::responses;
    use schema::responses::dsl::*;

    if improper_domain_name(publisher_name) {
        return Err(format!("Improper domain name {}", publisher_name));
    }

    let new_response = NewResponse {
        publisher: publisher_name,
    };

    match diesel::insert_into(responses::table)
        .values(&new_response)
        .execute(conn)
    {
            Ok(val) => val,
            Err(_) => return Err(format!("Insert for Response has failed.")),
    };
    let response_inserted = match responses
        .filter(publisher.like(publisher_name))
        .limit(1)
        .load::<Response>(conn)
        {
            Ok(val) => val,
            Err(_) => return Err(format!("Could not retreive id for newly added Response.")),
        };
    Ok(response_inserted[0].id)
}

//return the id of the last response made or return 0
pub fn get_latest_response_id(conn: &MysqlConnection) -> Result<i32, String> {
    use schema::responses::dsl::*;

    let latest_response = match responses
        .order(id.desc())
        .limit(1)
        .load::<Response>(conn)
        {
            Ok(val) => val,
            Err(_) => return Err(format!("Could not retrieve id for Response.")),
        };
    Ok(latest_response[0].id)
}

//delete a response with publisher_name and returns number of rows deleted
pub fn delete_response(conn: &MysqlConnection, publisher_name: String) -> Result<usize, String> {
    use schema::responses::dsl::*;

    if improper_domain_name(&publisher_name) {
        return Err(format!("Improper domain name {}", publisher_name));
    }

    match diesel::delete(responses.filter(publisher.like(publisher_name))).execute(conn) {
        Ok(val) => return Ok(val),
        Err(_) => return Err(format!("Delete for Response has failed.")),
    }
}

//TODO: error handling
//listen to events and maintain database based on the events
pub fn maintain_database() {
    //constants to compare values against
    const RPC_ENDPOINT: &str = "http://localhost:8545";
    const REGISTRY_ADDR: &str = "8009a230dc908e71befafba36e09efef2513640d";//THIS CHANGES BASED ON NETWORK

    //create web3 transport and communication
    let (_eloop, http) = web3::transports::Http::new(RPC_ENDPOINT).expect("Web3 failed to create transport.");
    let web3 = web3::Web3::new(http);
    
    //create filters for the past and streaming events
    let eth_filter_stream_events = EthFilter::new(web3.transport());
    let eth_filter_past_events = EthFilter::new(web3.transport());

    //create filter object to put perameters on the filtered events
    let filter = FilterBuilder::default()
        .from_block(BlockNumber::Number(0))
        .to_block(BlockNumber::Latest)
        .address(vec![H160::from_str(REGISTRY_ADDR).expect("Registry address could not be turned to H160.")]).build();
    let filter_past_events = filter.clone();
    
    //create the filters to be used
    let filter_stream_events = eth_filter_stream_events.create_logs_filter(filter).wait().expect("Filter could not be created.");
    let filter_past_events = eth_filter_past_events.create_logs_filter(filter_past_events).wait().expect("Filter could not be created.");

    //application and listing HashMap to keep track of domain names
    let applications = Mutex::new(HashMap::new());

    //start to stream events but not yet act on them
    let filter_stream = filter_stream_events.stream(Duration::from_secs(0))
        .for_each(|log| {
            let data_vector = log.data.0;
            let domain_name_hash = (&data_vector[0..32]).to_hex();
            let topics = log.topics[0];
            log_handler(domain_name_hash, &applications, topics, data_vector);
            Ok(())
        })
        .map_err(|_| {
            println!("Error with log stream");
        });
    //get all the past events
    let past_logs = match filter_past_events.logs().wait() {
        Ok(val) => val,
        Err(e) => panic!(format!("Could not retrieve past logs on the Registry: {:?}", e)),
    };

    //past_logs iterator
    for log in past_logs {
        let data_vector = log.data.0;
        let domain_name_hash = (&data_vector[0..32]).to_hex();
        let topics = log.topics[0];
        log_handler(domain_name_hash, &applications, topics, data_vector);
    }
    //start to iterate through new events
    filter_stream.wait().expect("Could not start stream of new logs.");
}

//uses the log data to determine what action to take (what event the log references)
fn log_handler(domain_name_hash: String, applications: & Mutex<HashMap<String, String>>, topics: H256, data_vector: Vec<u8>) {
    const APPLICATION_HASH: &str = "5cde15b9901ca13a7e2eb4fb919870d1bde9e8d93d9aa5e26945b42190067bdc";
    const NEW_LISTING_WHITELISTED_HASH: &str = "a7dee6157e26f0945c6e2fa27b51c0811370eb1863f1e5285e8dea4291fdd3de";
    const APPLICATION_REMOVED_HASH: &str = "2e5ec035f6eac8ff1cf7cdf36cfeca7c85413f9f67652dc2c13d20f337204a26";
    const LISTING_REMOVED_HASH: &str = "d1ffb796b7108387b2f02adf47b4b81a1690cf2a190422c87a4f670780103e63";

    //println!("Domain Name Hash: {}", domain_name_hash);
        if topics == H256::from_str(APPLICATION_HASH).expect("Const String could not be converted to H256.") {
            //println!("Application Event");
            let mut domain_name = String::new();
            for iterator in 0..data_vector.len() {
                if data_vector[(data_vector.len() - 1 - iterator) as usize] > 122 {
                    break;
                }
                else if data_vector[(data_vector.len() - 1 - iterator) as usize] > 44 {
                    domain_name.push(data_vector[(data_vector.len() - 1 - iterator) as usize] as char);
                }
                else {
                    break;
                }
            }
            unsafe {
                domain_name.as_mut_vec().reverse();
            }
            //println!("Application Domain Name: {}", domain_name);
            (*applications.lock().expect("Lock could not be unwrapped.")).insert(domain_name_hash, domain_name);
        }
        else if topics == H256::from_str(NEW_LISTING_WHITELISTED_HASH).expect("Const String could not be converted to H256.") {
            //println!("New Listing Whitelisted Event");
            //println!("Listing Domain Name: {:?}", (*applications.lock().expect("Lock could not be unwrapped.")).get(&domain_name_hash));
            let connection = establish_connection();
            let _creation = create_listing(&connection, (applications.lock().expect("Lock could not be unwrapped.").get(&domain_name_hash))
                .expect("Error retrieving information from HashMap."));
            //println!("Should be 1 entry: {}", _creation);
        }
        else if topics == H256::from_str(LISTING_REMOVED_HASH).expect("Const String could not be converted to H256.") {
            //println!("Listing Removed Event");
            //println!("Listing Domain Name: {:?}", (*applications.lock().expect("Lock could not be unwrapped.")).get(&domain_name_hash));
            let connection = establish_connection();
            let _deletion = delete_listing(&connection, (applications.lock().expect("Lock could not be unwrapped.").get(&domain_name_hash))
                .expect("Error retrieving information from HashMap.").to_string());
            //println!("Should be 1 entry: {}", _deletion);
            (*applications.lock().expect("Lock could not be unwrapped.")).remove(&domain_name_hash);
        }
        else if topics == H256::from_str(APPLICATION_REMOVED_HASH).expect("Const String could not be converted to H256.") {
            //println!("Application Removed Event");
            //println!("Listing Domain Name: {:?}", (*applications.lock().expect("Lock could not be unwrapped.")).get(&domain_name_hash));
            (*applications.lock().expect("Lock could not be unwrapped.")).remove(&domain_name_hash);
        }
}