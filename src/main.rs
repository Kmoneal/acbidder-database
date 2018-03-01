extern crate acbidder_database;
use acbidder_database::maintain_database;
//use acbidder_database::establish_connection;
//use acbidder_database::current_auto_increment_value;

fn main() {
	
    //let connection = establish_connection();
    //current_auto_increment_value(&connection);

    maintain_database();
}
