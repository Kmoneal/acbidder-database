extern crate acbidder_database;
extern crate diesel;
extern crate web3;

use acbidder_database::establish_connection;

use acbidder_database::create_listing;
use acbidder_database::is_whitelisted;
use acbidder_database::delete_listing;

use acbidder_database::create_request;
use acbidder_database::get_latest_request_id;
use acbidder_database::delete_request;

use acbidder_database::create_response;
use acbidder_database::get_latest_response_id;
use acbidder_database::delete_response;

use acbidder_database::models::*;
use acbidder_database::schema::listings::dsl::*;

use diesel::query_dsl::limit_dsl::LimitDsl;
use diesel::RunQueryDsl;

use acbidder_database::current_auto_increment_value_responses;
use acbidder_database::current_auto_increment_value_requests;


///---------------------------------------------------------------------
///NOTE: Tests must be run with a clean table and using -- --test-threads=1
///---------------------------------------------------------------------

//first.com
//check that the ad_server is being added to listing table
#[test]
fn test_1_add_ad_server_to_listing() {
    let connection = establish_connection();
    let creation = create_listing(&connection, "first.com").unwrap();
    assert!(creation == 1, "Insertion failed");

    let results = listings
        .limit(5)
        .load::<AdServer>(&connection)
        .expect("Error loading adserver");
    for ad_server in results {
        assert_eq!(ad_server.domain, format!("first.com"));
        println!("{}", ad_server.domain);
    }

    let deletion = delete_listing(&connection, format!("first.com")).unwrap();
    assert!(deletion == 1, "Deletion failed");
}

//second.com
//check that the ad_server with the same name cannot be added to listing
#[test]
fn test_2_add_invalid_ad_server_to_listing() {
    let connection = establish_connection();
    let creation = create_listing(&connection, "second.com").unwrap();
    assert!(creation == 1, "Insertion failed");
    match create_listing(&connection, "second.com"){
        Ok(_) => panic!("Insert succeeded when it was supposed to fail."),
        Err(e) => if e != "Insert for Listing failed." {
            panic!("Repeat insertion succeeded.");
        },
    };

    let results = listings
        .limit(5)
        .load::<AdServer>(&connection)
        .expect("Error loading adserver");
    for ad_server in results {
        assert_eq!(ad_server.domain, format!("second.com"));
        println!("{}", ad_server.domain);
    }

    let deletion = delete_listing(&connection, format!("second.com")).unwrap();
    assert!(deletion == 1, "Deletion failed");
}

//third.com
//check that listing is removed properly
#[test]
fn test_3_add_and_remove_ad_server_from_listing() {
    let connection = establish_connection();
    let creation = create_listing(&connection, "third.com").unwrap();
    assert!(creation == 1, "Insertion failed");

    let deletion = delete_listing(&connection, format!("third.com")).unwrap();
    assert!(deletion == 1, "Deletion failed");
}

//fourth.com
//should always succeed
#[test]
fn test_4() {

}

//fifth.com
//check that special characters cannot be used as a domain name
#[test]
fn test_5_add_invalid_ad_server_from_listing_using_special_characters() {
    let connection = establish_connection();
    match create_listing(&connection, "*fifth.com"){
        Ok(_) => panic!("Create listing succeeded when it was supposed to fail."),
        Err(e) => if e != "Improper domain name *fifth.com" {
            panic!("Insertion with special character * succeeded.");
        },
    };
    match create_listing(&connection, "the_fifth.com"){
        Ok(_) => panic!("Create listing succeeded when it was supposed to fail."),
        Err(e) => if e != "Improper domain name the_fifth.com" {
            panic!("Insertion with special character _ succeeded.");
        },
    };
    match create_listing(&connection, "%fifth.com"){
        Ok(_) => panic!("Create listing succeeded when it was supposed to fail."),
        Err(e) => if e != "Improper domain name %fifth.com" {
            panic!("Insertion with special character % succeeded.");
        },
    };

    let deletion = delete_listing(&connection, format!("%fifth.com")).unwrap();
    assert!(deletion == 0, "Deletion succeeded when there are no matches.");
}

//sixth.com
//check that a non-existing domain cannot be deleted and returns proper response
#[test]
fn test_6_invalid_remove_ad_server_from_listing() {
    let connection = establish_connection();
    let deletion = delete_listing(&connection, format!("definitelyNotReal.com")).unwrap();
    assert!(deletion == 0, "Deletion succeeded when there are no matches.");
}

//seventh.com
//check if the whitelist function works to find a specified domain
#[test]
fn test_7_add_ad_server_to_listing_and_valid_whitelist() {
    let connection = establish_connection();
    let creation = create_listing(&connection, "seventh.com").unwrap();
    assert!(creation == 1, "Insertion failed");

    let is_whitelisted = is_whitelisted(&connection, format!("seventh.com")).unwrap();
    assert!(
        is_whitelisted,
        "is_whitelisted returned false when entry exists"
    );

    let deletion = delete_listing(&connection, format!("seventh.com")).unwrap();
    assert!(deletion == 1, "Deletion failed");
}

//eighth.com
//check the whitelist function for a false positive
#[test]
fn test_8_add_ad_server_to_listing_and_invalid_whitelist() {
    let connection = establish_connection();
    let creation = create_listing(&connection, "eighth.com").unwrap();
    assert!(creation == 1, "Insertion failed");

    let is_whitelisted = is_whitelisted(&connection, format!("Noteighth.com")).unwrap();
    assert!(
        !is_whitelisted,
        "is_whitelisted returned true when entry does not exits"
    );

    let deletion = delete_listing(&connection, format!("eighth.com")).unwrap();
    assert!(deletion == 1, "Deletion failed");
}

//nineth.com
//check that multiple listings can be loaded
#[test]
fn test_9_add_ad_server_to_listing_and_show_ad_servers() {
    let connection = establish_connection();
    let creation = create_listing(&connection, "anineth.com").unwrap();
    assert!(creation == 1, "Insertion failed");
    let creation = create_listing(&connection, "bnineth.com").unwrap();
    assert!(creation == 1, "Insertion failed");
    let creation = create_listing(&connection, "cnineth.com").unwrap();
    assert!(creation == 1, "Insertion failed");
    let creation = create_listing(&connection, "dnineth.com").unwrap();
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

    let deletion = delete_listing(&connection, format!("%nineth.com")).unwrap();
    assert!(deletion == 4, "Deletion failed");
}

//tenth.com
//check that special character deletion works
#[test]
fn test_10_add_ad_server_to_listing_and_special_character_deletion() {
    let connection = establish_connection();
    let creation = create_listing(&connection, "tenth.com").unwrap();
    assert!(creation == 1, "Insertion failed");

    let deletion = delete_listing(&connection, format!("%.com")).unwrap();
    assert!(deletion == 1, "Deletion failed. There may be persisting entries from before.");

    let creation = create_listing(&connection, "tenth.com").unwrap();
    assert!(creation == 1, "Insertion failed");

    let deletion = delete_listing(&connection, format!("te_th.com")).unwrap();
    assert!(deletion == 1, "Deletion failed");
}

//eleventh.com
//check that special characters cannot be used on is_whitelist function
#[test]
fn test_11_add_ad_server_to_listing_and_special_character_whitelisted() {
    let connection = establish_connection();
    let creation = create_listing(&connection, "eleventh.com").unwrap();
    assert!(creation == 1, "Insertion failed");

    match is_whitelisted(&connection, format!("_leventh.com")){
        Ok(_) => panic!("is_whitelisted succeeded when it was supposed to fail"),
        Err(e) => if e != "Improper domain name _leventh.com" {
            panic!("is_whitelisted returned true when entry does not exits");
        },
    };

    match is_whitelisted(&connection, format!("%venth.com")){
        Ok(_) => panic!("is_whitelisted succeeded when it was supposed to fail"),
        Err(e) => if e != "Improper domain name %venth.com" {
            panic!("is_whitelisted returned true when entry does not exits");
        },
    };

    let deletion = delete_listing(&connection, format!("eleventh.com")).unwrap();
    assert!(deletion == 1, "Deletion failed");
}

//twelfth.com
//
#[test]
fn test_12_add_and_remove_request () {
    let connection = establish_connection();
    let auto_increment_value = current_auto_increment_value_requests(&connection).unwrap();
    let creation = create_request(&connection, "twelfth.com", 5).unwrap();
    assert!(creation == auto_increment_value as i32, "Insertion failed");
    let deletion = delete_request(&connection, format!("twelfth.com")).unwrap();
    assert!(deletion == 1, "Deletion failed");
}

//thirteenth.com
//
#[test]
fn test_13_add_and_remove_request_check_id () {
    let connection = establish_connection();
    let auto_increment_value = current_auto_increment_value_requests(&connection).unwrap();
    let creation = create_request(&connection, "thirteenth.com", 2).unwrap();
    assert!(creation == auto_increment_value as i32, "Insertion failed");

    let identification_value = get_latest_request_id(&connection).unwrap();
    assert!(creation == identification_value, "ID value does not match the latest insertion into requests table");

    let deletion = delete_request(&connection, format!("thirteenth.com")).unwrap();
    assert!(deletion == 1, "Deletion failed");
}

//fourteenth.com
//
#[test]
fn test_14_add_and_remove_response () {
    let connection = establish_connection();
    let auto_increment_value = current_auto_increment_value_responses(&connection).unwrap();
    let creation = create_response(&connection, "fourteenth.com").unwrap();
    assert!(creation == auto_increment_value as i32, "Insertion failed");
    let deletion = delete_response(&connection, format!("fourteenth.com")).unwrap();
    assert!(deletion == 1, "Deletion failed");
}

//fifteenth.com
//
#[test]
fn test_15_add_and_remove_response_check_id () {
    let connection = establish_connection();
    let auto_increment_value = current_auto_increment_value_responses(&connection).unwrap();
    let creation = create_response(&connection, "fifteenth.com").unwrap();
    assert!(creation == auto_increment_value as i32, "Insertion failed");

    let identification_value = get_latest_response_id(&connection).unwrap();
    assert!(creation == identification_value, "ID value does not match the latest insertion into requests table");

    let deletion = delete_response(&connection, format!("fifteenth.com")).unwrap();
    assert!(deletion == 1, "Deletion failed");
}