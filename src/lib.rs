#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate rustc_hex;

extern crate tiny_keccak;
extern crate web3;

use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use dotenv::dotenv;
use std::env;
use std::str::FromStr;
use web3::types::*;
use web3::futures::Future;
use web3::api::EthFilter;
use web3::api::Namespace;
use std::time::Duration;
use web3::futures::Stream;
use std::collections::HashMap;
use std::sync::Mutex;

use rustc_hex::ToHex;


use self::models::{AdServer, NewAdServer};

pub mod schema;
pub mod models;
pub mod adchain_registry;

pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connection to {}", database_url))
}

pub fn create_listing<'a>(conn: &MysqlConnection, domain_name: &'a str) -> usize {
    use schema::listings;

    //ensures that no special characters are used and valid domain name characters are used
    for character in domain_name.chars() {
        if !character.is_ascii_alphanumeric() && character != '.' && character != '-' {
            return 0;
        }
    }

    let new_ad_server = NewAdServer {
        domain: domain_name,
    };
    match diesel::insert_into(listings::table)
        .values(&new_ad_server)
        .execute(conn)
    {
        Ok(val) => return val,
        Err(_) => return 0,
    }
}

//delets a listing (ad_server) with the name in domain_name and returns the number of rows deleted
pub fn delete_listing(conn: &MysqlConnection, domain_name: String) -> usize {
    use schema::listings::dsl::*;

    match diesel::delete(listings.filter(domain.like(domain_name))).execute(conn) {
        Ok(val) => return val,
        Err(_) => return 0,
    }
}

pub fn is_whitelisted(conn: &MysqlConnection, domain_name: String) -> bool {
    use schema::listings::dsl::*;

    for character in domain_name.chars() {
        if !character.is_ascii_alphanumeric() && character != '.' && character != '-' {
            return false;
        }
    }

    match listings
        .filter(domain.like(domain_name))
        .limit(1)
        .load::<AdServer>(conn)
    {
        Ok(val) => {
            if val.len() == 0 {
                return false;
            } else if val.len() > 1 {
                panic!("Too many matches to domain, something went wrong!");
            } else {
                return true;
            }
        }
        Err(e) => println!("Could not check if whitelisted: {}", e),
    }
    return false;
}

pub fn maintain_database() {
    //constants to compare values against
    const RPC_ENDPOINT: &str = "http://localhost:8545";
    const REGISTRY_ADDR: &str = "0x8009a230dc908e71befafba36e09efef2513640d";//THIS CHANGES BASED ON NETWORK
    const APPLICATION_HASH: &str = "0x5cde15b9901ca13a7e2eb4fb919870d1bde9e8d93d9aa5e26945b42190067bdc";
    const NEW_LISTING_WHITELISTED_HASH: &str = "0xa7dee6157e26f0945c6e2fa27b51c0811370eb1863f1e5285e8dea4291fdd3de";
    const APPLICATION_REMOVED_HASH: &str = "2e5ec035f6eac8ff1cf7cdf36cfeca7c85413f9f67652dc2c13d20f337204a26";
    const LISTING_REMOVED_HASH: &str = "d1ffb796b7108387b2f02adf47b4b81a1690cf2a190422c87a4f670780103e63";

    //create web3 transport and communication
    let (_eloop, http) = web3::transports::Http::new(RPC_ENDPOINT).unwrap();
    let web3 = web3::Web3::new(http);
    
    //create filters for the past and streaming events
    let eth_filter_stream_events = EthFilter::new(web3.transport());
    let eth_filter_past_events = EthFilter::new(web3.transport());

    //create filter object to put perameters on the filtered events
    let filter = FilterBuilder::default()
        .from_block(BlockNumber::Number(0))
        .to_block(BlockNumber::Latest)
        .address(vec![H160::from_str(REGISTRY_ADDR).unwrap()]).build();
    let filter_past_events = filter.clone();
    
    //create the filters to be used
    let filter = eth_filter_stream_events.create_logs_filter(filter).wait().unwrap();
    let filter_past_events = eth_filter_past_events.create_logs_filter(filter_past_events).wait().unwrap();

    //application and listing HashMap to keep track of domain names
    let mut applications = Mutex::new(HashMap::new());

    //start to stream events but not yet act on them
    let filter_stream = filter.stream(Duration::from_secs(0))
        .for_each(|log| {

            let mut domain_name_hash = String::new();
            let mut data_vector: Vec<u8> = vec![];
            data_vector = log.data.0;

            println!("\nTEST DOMAIN NAME HASH {}\n", data_vector.to_hex());

            domain_name_hash = (&data_vector[0..32]).to_hex();

            println!("Domain Name Hash: {}", domain_name_hash);

            // if appplication was made, store applicant info in applications HashMap
            if log.topics[0] == H256::from_str(APPLICATION_HASH).unwrap() {
                println!("Application Event");
                let mut domain_name = String::new();
                for iterator in 0..data_vector.len() {
                    if data_vector[(data_vector.len() - 1 - iterator) as usize] > 44 {
                        domain_name.push(data_vector[(data_vector.len() - 1 - iterator) as usize] as char);
                    }
                    else {
                        break;
                    }
                }
                unsafe {
                    domain_name.as_mut_vec().reverse();
                }
                println!("Application Domain Name: {}", domain_name);
                (*applications.lock().unwrap()).insert(domain_name_hash, domain_name);
            }
            else if log.topics[0] == H256::from_str(NEW_LISTING_WHITELISTED_HASH).unwrap() {
                println!("New Listing Whitelisted Event");
                println!("Listing Domain Name: {:?}", (*applications.lock().unwrap()).get(&domain_name_hash));
                //////INSERT INTO DATABASE
                //applications.remove(&domain_name_hash);
            }
            else if log.topics[0] == H256::from_str(LISTING_REMOVED_HASH).unwrap() {
                println!("Listing Removed Event");
                println!("Listing Domain Name: {:?}", (*applications.lock().unwrap()).get(&domain_name_hash));
                //////DELETE FROM DATABASE
                (*applications.lock().unwrap()).remove(&domain_name_hash);
            }
            else if log.topics[0] == H256::from_str(APPLICATION_REMOVED_HASH).unwrap() {
                println!("Application Removed Event");
                println!("Listing Domain Name: {:?}", (*applications.lock().unwrap()).get(&domain_name_hash));
                (*applications.lock().unwrap()).remove(&domain_name_hash);
            }
            Ok(())
        })
        .map_err(|err| {
            println!("Error with log stream");
        });

    //get all the past events
    let past_logs = match filter_past_events.logs().wait() {
        Ok(val) => val,
        Err(e) => panic!(format!("Could not retrieve past logs on the Registry: {:?}", e)),
    };

    //past_logs iterator
    for log in past_logs {
        let mut domain_name_hash = String::new();
        let mut data_vector: Vec<u8> = vec![];
        data_vector = log.data.0;

        domain_name_hash = (&data_vector[0..32]).to_hex();

        println!("Domain Name Hash: {}", domain_name_hash);
        if log.topics[0] == H256::from_str(APPLICATION_HASH).unwrap() {
            println!("Application Event");
            let mut domain_name = String::new();
            for iterator in 0..data_vector.len() {
                if data_vector[(data_vector.len() - 1 - iterator) as usize] > 44 {
                    domain_name.push(data_vector[(data_vector.len() - 1 - iterator) as usize] as char);
                }
                else {
                    break;
                }
            }
            unsafe {
                domain_name.as_mut_vec().reverse();
            }
            println!("Application Domain Name: {}", domain_name);
            (*applications.lock().unwrap()).insert(domain_name_hash, domain_name);
        }
        else if log.topics[0] == H256::from_str(NEW_LISTING_WHITELISTED_HASH).unwrap() {
            println!("New Listing Whitelisted Event");
            println!("Listing Domain Name: {:?}", (*applications.lock().unwrap()).get(&domain_name_hash));
            //////INSERT INTO DATABASE
            //applications.remove(&domain_name_hash);
        }
        else if log.topics[0] == H256::from_str(LISTING_REMOVED_HASH).unwrap() {
            println!("Listing Removed Event");
            println!("Listing Domain Name: {:?}", (*applications.lock().unwrap()).get(&domain_name_hash));
            //////DELETE FROM DATABASE
            (*applications.lock().unwrap()).remove(&domain_name_hash);
        }
        else if log.topics[0] == H256::from_str(APPLICATION_REMOVED_HASH).unwrap() {
            println!("Application Removed Event");
            println!("Listing Domain Name: {:?}", (*applications.lock().unwrap()).get(&domain_name_hash));
            (*applications.lock().unwrap()).remove(&domain_name_hash);
        }
    }

    //start to iterate through new events
    filter_stream.wait();
}
