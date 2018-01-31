#[macro_use]
extern crate diesel;
extern crate dotenv;

extern crate web3;
extern crate tiny_keccak;

use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use dotenv::dotenv;
use std::env;

use self::models::{AdServer, NewAdServer};

pub mod schema;
pub mod models;
pub mod adchain_registry;


pub fn establish_connection() -> MysqlConnection {
	dotenv().ok();

	let database_url = env::var("DATABASE_URL")
	    .expect("DATABASE_URL must be set");
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
	match diesel::insert_into(listings::table).values(&new_ad_server).execute(conn) {
		Ok(val) => return val,
		Err(_) => return 0,
	}
}

//delets a listing (ad_server) with the name in domain_name and returns the number of rows deleted
pub fn delete_listing(conn: &MysqlConnection, domain_name: String) ->  usize {
	use schema::listings::dsl::*;
    
    match diesel::delete(listings.filter(domain.like(domain_name)))
        .execute(conn) {
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

    match listings.filter(domain.like(domain_name)).limit(1).load::<AdServer>(conn) {
    	Ok(val) => {
    		if val.len() == 0 {
    			return false
    		}
    		else if val.len() > 1 {
    			panic!("Too many matches to domain, something went wrong!");
    		}
    		else {
    			return true;
    		}
    	},
    	Err(e) => println!("Could not check if whitelisted: {}", e),
    }
    return false;
}

pub fn maintain_database() {
    use std::{thread, time};
    let duration = time::Duration::from_sec(3600);

	let connection = establish_connection();

	const RPC_ENDPOINT: &str = "http://localhost:8545";
    let (_eloop, http) = web3::transports::Http::new(RPC_ENDPOINT)
        .unwrap();
    let web3 = web3::Web3::new(http);
    let adchain_registry = adchain_registry::RegistryInstance::new(&web3);

    while 1 {
    	let _is_whitelisted = adchain_registry.is_in_registry("fox.com");
        if _is_whitelisted {
    	    let creation = create_listing(&connection, "fox.com");
    	    assert!(creation == 1, "Insertion failed");
        }
    }
}