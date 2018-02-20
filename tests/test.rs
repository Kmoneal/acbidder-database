extern crate acbidder_database;
extern crate diesel;
extern crate web3;

use acbidder_database::establish_connection;
use acbidder_database::create_listing;
use acbidder_database::delete_listing;
use acbidder_database::is_whitelisted;
use acbidder_database::models::*;
use acbidder_database::schema::listings::dsl::*;
use diesel::query_dsl::limit_dsl::LimitDsl;
use diesel::RunQueryDsl;
use acbidder_database::adchain_registry;

///---------------------------------------------------------------------
///NOTE: Tests must be run with a clean table and using -- --test-threads=1
///---------------------------------------------------------------------

//first.com
//check that the ad_server is being added to listing table
#[test]
fn test_1_add_ad_server_to_listing() {
    let connection = establish_connection();
    let creation = create_listing(&connection, "first.com");
    assert!(creation == 1, "Insertion failed");

    let results = listings
        .limit(5)
        .load::<AdServer>(&connection)
        .expect("Error loading adserver");
    for ad_server in results {
        assert_eq!(ad_server.domain, format!("first.com"));
        println!("{}", ad_server.domain);
    }

    let deletion = delete_listing(&connection, format!("first.com"));
    assert!(deletion == 1, "Deletion failed");
}

//second.com
//check that the ad_server with the same name cannot be added to listing
#[test]
fn test_2_add_invalid_ad_server_to_listing() {
    let connection = establish_connection();
    let creation = create_listing(&connection, "second.com");
    assert!(creation == 1, "Insertion failed");
    let creation = create_listing(&connection, "second.com");
    assert!(creation == 0, "Repeat insertion succeeded");

    let results = listings
        .limit(5)
        .load::<AdServer>(&connection)
        .expect("Error loading adserver");
    for ad_server in results {
        assert_eq!(ad_server.domain, format!("second.com"));
        println!("{}", ad_server.domain);
    }

    let deletion = delete_listing(&connection, format!("second.com"));
    assert!(deletion == 1, "Deletion failed");
}

//third.com
//check that listing is removed properly
#[test]
fn test_3_add_and_remove_ad_server_from_listing() {
    let connection = establish_connection();
    let creation = create_listing(&connection, "third.com");
    assert!(creation == 1, "Insertion failed");

    let deletion = delete_listing(&connection, format!("third.com"));
    assert!(deletion == 1, "Deletion failed");
}

//fourth.com
//placeholder
#[test]
fn test_4() {}

//fifth.com
//check that special characters cannot be used as a domain name
#[test]
fn test_5_add_invalid_ad_server_from_listing_using_special_characters() {
    let connection = establish_connection();
    let creation = create_listing(&connection, "*fifth.com");
    assert!(
        creation == 0,
        "Insertion with special character * succeeded"
    );
    let creation = create_listing(&connection, "the_fifth.com");
    assert!(
        creation == 0,
        "Insertion with special character _ succeeded"
    );
    let creation = create_listing(&connection, "%fifth.com");
    assert!(
        creation == 0,
        "Insertion with special character % succeeded"
    );

    let deletion = delete_listing(&connection, format!("%fifth.com"));
    assert!(
        deletion == 0,
        "Deletion succeeded when there are no matches"
    );
}

//sixth.com
//check that a non-existing domain cannot be deleted and returns proper response
#[test]
fn test_6_invalid_remove_ad_server_from_listing() {
    let connection = establish_connection();
    let deletion = delete_listing(&connection, format!("definitelyNotReal.com"));
    assert!(
        deletion == 0,
        "Deletion succeeded when there are no matches"
    );
}

//seventh.com
//check if the whitelist function works to find a specified domain
#[test]
fn test_7_add_ad_server_to_listing_and_valid_whitelist() {
    let connection = establish_connection();
    let creation = create_listing(&connection, "seventh.com");
    assert!(creation == 1, "Insertion failed");

    let is_whitelisted = is_whitelisted(&connection, format!("seventh.com"));
    assert!(
        is_whitelisted,
        "is_whitelisted returned false when entry exists"
    );

    let deletion = delete_listing(&connection, format!("seventh.com"));
    assert!(deletion == 1, "Deletion failed");
}

//eighth.com
//check the whitelist function for a false positive
#[test]
fn test_8_add_ad_server_to_listing_and_invalid_whitelist() {
    let connection = establish_connection();
    let creation = create_listing(&connection, "eighth.com");
    assert!(creation == 1, "Insertion failed");

    let is_whitelisted = is_whitelisted(&connection, format!("Noteighth.com"));
    assert!(
        !is_whitelisted,
        "is_whitelisted returned true when entry does not exits"
    );

    let deletion = delete_listing(&connection, format!("eighth.com"));
    assert!(deletion == 1, "Deletion failed");
}

//nineth.com
//check that multiple listings can be loaded
#[test]
fn test_9_add_ad_server_to_listing_and_show_ad_servers() {
    let connection = establish_connection();
    let creation = create_listing(&connection, "anineth.com");
    assert!(creation == 1, "Insertion failed");
    let creation = create_listing(&connection, "bnineth.com");
    assert!(creation == 1, "Insertion failed");
    let creation = create_listing(&connection, "cnineth.com");
    assert!(creation == 1, "Insertion failed");
    let creation = create_listing(&connection, "dnineth.com");
    assert!(creation == 1, "Insertion failed");

    let results = listings
        .limit(5)
        .load::<AdServer>(&connection)
        .expect("Error loading adserver");

    assert!(results.len() == 4);
    let mut iteration = 0;
    for ad_server in results {
        if iteration == 0 {
            assert_eq!(ad_server.domain, format!("anineth.com"));
        }
        if iteration == 1 {
            assert_eq!(ad_server.domain, format!("bnineth.com"));
        }
        if iteration == 2 {
            assert_eq!(ad_server.domain, format!("cnineth.com"));
        }
        if iteration == 3 {
            assert_eq!(ad_server.domain, format!("dnineth.com"));
        }
        iteration += 1;
    }

    let deletion = delete_listing(&connection, format!("%nineth.com"));
    assert!(deletion == 4, "Deletion failed");
}

//tenth.com
//check that special character deletion works
#[test]
fn test_10_add_ad_server_to_listing_and_special_character_deletion() {
    let connection = establish_connection();
    let creation = create_listing(&connection, "tenth.com");
    assert!(creation == 1, "Insertion failed");

    let deletion = delete_listing(&connection, format!("%.com"));
    assert!(deletion == 1, "Deletion failed");

    let creation = create_listing(&connection, "tenth.com");
    assert!(creation == 1, "Insertion failed");

    let deletion = delete_listing(&connection, format!("te_th.com"));
    assert!(deletion == 1, "Deletion failed");
}

//eleventh.com
//check that special characters cannot be used on is_whitelist function
#[test]
fn test_11_add_ad_server_to_listing_and_special_character_whitelisted() {
    let connection = establish_connection();
    let creation = create_listing(&connection, "eleventh.com");
    assert!(creation == 1, "Insertion failed");

    let _is_whitelisted = is_whitelisted(&connection, format!("_leventh.com"));
    assert!(
        !_is_whitelisted,
        "is_whitelisted returned true when entry does not exits"
    );

    let _is_whitelisted = is_whitelisted(&connection, format!("%venth.com"));
    assert!(
        !_is_whitelisted,
        "is_whitelisted returned true when entry does not exits"
    );

    let deletion = delete_listing(&connection, format!("eleventh.com"));
    assert!(deletion == 1, "Deletion failed");
}

//These tests were an attempt to test web3 but have the same issues that warrant this crate (eventloop being dropped causing thread to be hungup)
// #[test]
// fn z_load_database() {
// 	const RPC_ENDPOINT: &str = "http://localhost:8545";
// 	let connection = establish_connection();
//     let (_eloop, http) = web3::transports::Http::new(RPC_ENDPOINT)
//         .unwrap();
//     let web3 = web3::Web3::new(http);
//     let adchain_registry = adchain_registry::RegistryInstance::new(&web3);
//     let _is_whitelisted = adchain_registry.is_in_registry("fox.com");
//     if _is_whitelisted {
//     	let creation = create_listing(&connection, "fox.com");
//     	assert!(creation == 1, "Insertion failed");
//     }
// }

// #[test]
// fn y_load_database() {
//     const RPC_ENDPOINT: &str = "http://localhost:8545";
// 	let connection = establish_connection();
//     let (_eloop, http) = web3::transports::Http::new(RPC_ENDPOINT)
//         .unwrap();
//     let web3 = web3::Web3::new(http);
//     let adchain_registry = adchain_registry::RegistryInstance::new(&web3);
//     let _is_whitelisted = adchain_registry.is_in_registry("msn.com");
//     if _is_whitelisted {
//     	let creation = create_listing(&connection, "msn.com");
//     	assert!(creation == 1, "Insertion failed");
//     }
// }
