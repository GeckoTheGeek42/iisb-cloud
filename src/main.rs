#![feature(plugin, test, str_char)]
#![plugin(maud_macros)]
// #![plugin(postgres_macros)]

#[macro_use]
extern crate log;

extern crate test;

extern crate postgres;

extern crate iron;
//extern crate router;
//extern crate plugin;
extern crate maud;

extern crate rustc_serialize;

extern crate crypto;

// extern crate mio;

extern crate scoped_threadpool;
extern crate threadpool;

extern crate chrono;

pub mod server;
pub mod db;
pub mod crypt;
pub mod data;
mod logger;

use db::{IndusDatabase};
use data::{Counts, Gender};
use postgres::{Connection, IntoConnectParams, ConnectParams, UserInfo, ConnectTarget, SslMode};
use rustc_serialize::json;
use std::fs::File;
use std::io::{Write, self};

fn main() {
	fn db_test() {
		let mut database = IndusDatabase::new();
		database.insert_student("Anshuman", "Medhi", Gender::Male, 
			"C1 English SL, C2 Spanish SL, C3 Math HL, C4 Economics SL, C5 Chemistry HL, C6 Physics HL",
			11, "B", "2cool4uuu");
		database.insert_teacher("Hari", "Prasad", Gender::Male,
			"Economics", "Wiggle Wiggle Wiggle Wiggle Wiggle YEAH", false,
			"killalion");
		println!( "{:#?}", database.login_profile("anshuman.medhi", "2cool4uuu") );
	}
	fn maud_test() {
		let name = "Lyra";
		let markup = html! {
		    p { "Hi, " $name "!" }
		};
		markup.render(&mut io::stdout()).unwrap();
	}
	db_test();
	maud_test();
}