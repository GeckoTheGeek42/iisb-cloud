use postgres::types::ToSql;
use postgres::{Connection, IntoConnectParams, ConnectParams, ConnectTarget, SslMode, UserInfo};
use postgres::error::Error as pgError;

use std::convert::Into;

use crypt::{encrypt, self};

use rustc_serialize::json::{encode, decode};
use std::io::{Read, Write};

use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};

use data::*;

pub struct PostgreDatabase {
	pub conn: Connection,
}

impl PostgreDatabase {
	#[inline]
	pub fn new<S1: Into<String>, S2: Into<String>, S3: Into<String>>(host: S1, port: Option<u16>, user: S2, pass: Option<S3>) -> PostgreDatabase {
		PostgreDatabase {
			conn: Connection::connect(ConnectParams {
				target: ConnectTarget::Tcp(host.into()),
				port: port,
				user: Some(UserInfo {
					user: user.into(),
					password: pass.map(Into::into),
				}),
				database: None,
				options: Vec::new()
			}, &SslMode::None).unwrap()
		}
	}
	#[inline]
	pub fn connect<S1: Into<String>, S2: Into<String>, S3: Into<String>, S4: Into<String>>
		(host: S1, port: Option<u16>, user: S2, pass: Option<S3>, db: Option<S4>, args: Vec<(String, String)>, mode: SslMode) -> PostgreDatabase {
		PostgreDatabase {
			conn: Connection::connect(ConnectParams {
				target: ConnectTarget::Tcp(host.into()),
				port: port,
				user: Some(UserInfo {
					user: user.into(),
					password: pass.map(Into::into),
				}),
				database: db.map(Into::into),
				options: args
			}, &mode).unwrap()
		}
		// PostgreDatabase::cons(&format!("postgres://{}:{}@{}", user, pass, host), &SslMode::None)
	}

	#[inline]	
	pub fn cons<I: IntoConnectParams>(i: I, mode: &SslMode) -> PostgreDatabase {
		PostgreDatabase {
			conn: Connection::connect(i, mode).unwrap()
		}
	}

	#[inline]
	pub fn exec(&self, query: &str, args: &[&ToSql]) -> Result<u64, pgError> {
		self.conn.execute(query, args)
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoginFailure {
	NoAccount, DuplicateAccounts, PasswordMismatch
}

pub struct IndusDatabase {
	database: PostgreDatabase,
	props: PathBuf, pub cnts: Counts,
}

impl IndusDatabase {
	pub fn new() -> IndusDatabase {
		let props = PathBuf::from("props.cfg");
		let file = File::open(&props);
		let counts = decode(
			&file.map(|mut f| {
				let mut buf = String::new();
				f.read_to_string(&mut buf).unwrap();
				buf
			}).unwrap_or(encode(&Counts::new()).unwrap())
		).unwrap();
		println!("{:?}", counts);
		IndusDatabase {
			database: PostgreDatabase::connect(
				"localhost", Some(5432), 
				"rusti", Some("2cool4uuu"), 
				Some("postgres"), 
				Vec::new(), SslMode::None
			),
			props: props, cnts: counts
		}
	}

	pub fn login(&self, username: &str, password: &str) -> Result<i32, LoginFailure> {
		let stmt = self.database.conn
			.prepare("SELECT id FROM users WHERE username = $1 AND password = $2").unwrap();
		let pwd = encrypt(password);
		let rows = match stmt.query(&[&username, &pwd]) {
			Ok(rows) => rows, 
			Err(_) => return Result::Err(LoginFailure::NoAccount),
		};
		if rows.len() > 1 {
			Err(LoginFailure::DuplicateAccounts)
		} else {
			Ok(rows.get(0).get(0))
		}
	}

	pub fn login_profile(&self, username: &str, password: &str) -> Result<IndusUser, LoginFailure> {
		let stmt = self.database.conn.prepare(
			"SELECT id, first_name, last_name, gender 
			FROM users WHERE username = $1 AND password = $2").unwrap();
		let pwd = encrypt(password);
		let rows = match stmt.query(&[&username, &pwd]) {
			Ok(rows) => rows, 
			Err(_) => return Result::Err(LoginFailure::NoAccount),
		};
		if rows.len() > 1 {
			Err(LoginFailure::DuplicateAccounts)
		} else {
			let row = rows.get(0);
			let id: i32 = row.get(0);
			let idargs: &[&ToSql] = &[&id];
			let first_name = { 
				let temp: String = row.get(1);
				String::from( temp.trim() )
			};
			let last_name = { 
				let temp: String = row.get(2);
				String::from( temp.trim() )
			};
			let gender = {
				let temp: bool = row.get(3);
				Gender::from(temp)
			};
			let role = {
				let srows_stmt = self.database.conn
					.prepare("SELECT classes, grade, section FROM students WHERE id = $1").unwrap();
				let student_rows = srows_stmt.query(idargs);
				match student_rows {
					Ok(srows) => {
						let srow = srows.get(0);
						let classes: String = srow.get(0);
						let section: String = srow.get(2);
						StudentTeacher::Student(IndusStudent {
							classes: Classes::from_student(srow.get(1), classes.trim()),
							grade: srow.get(1), section: section.char_at(0)
						})
					},
					Err(_) => {
						let trows_stmt = self.database.conn
							.prepare("SELECT subject, classes, hod FROM teachers WHERE id = $1").unwrap();
						let teacher_rows = trows_stmt.query(idargs);
						match teacher_rows {
							Ok(trows) => {
								let trow = trows.get(0);
								let subject: String = trow.get(0);
								let classes: String = trow.get(1);
								let hod: bool = trow.get(2);
								StudentTeacher::Teacher(IndusTeacher {
									subject: subject.trim().into(),
									classes: Classes::from_teacher(
										&subject, &format!("{} {}", first_name, last_name),
										classes.trim()
									),
									hod: hod,
								})
							},
							Err(_) => return Err(LoginFailure::NoAccount)
						}
					}
				}
			};
			Ok(IndusUser {
				first_name: first_name,
				last_name: last_name,
				gender: gender,
				role: role,
			})
		}	
	}

	pub fn change_pwd(&self, username: &str, password: &str) -> Result<u64, pgError> {
		let hash = encrypt(password);
		self.database.exec("UPDATE users SET password='$1' WHERE username='$2'", 
			&[&hash, &username])
	}

	pub fn insert_student<S1: Into<String>, S2: Into<String>, S3: Into<String>, S4: Into<String>, G: Into<bool>>
		(&mut self, first_name: S1, last_name: S2, gender: G, classes: S3, grade: i16, section: S4, password: &str)
		-> Result<u64, pgError> {
		let firstname = first_name.into();
		let lastname = last_name.into();
		try!( self.database.exec(
			"INSERT INTO Users (ID, first_name, last_name, gender, username, password) VALUES ($1, $2, $3, $4, $5, $6)", 
			&[&self.cnts.usrcnt, &firstname, &lastname, &gender.into(), &format!("{}.{}", firstname.to_lowercase(), lastname.to_lowercase()), &encrypt(password)]
		) );
		try!( self.database.exec(
			"INSERT INTO Students (ID, classes, grade, section) VALUES ($1, $2, $3, $4)",
			&[&self.cnts.usrcnt, &classes.into(), &grade, &section.into()]
		) );
		self.cnts.incr_students();
		Ok(2)

	}
	pub fn insert_teacher<S1: Into<String>, S2: Into<String>, S3: Into<String>, S4: Into<String>, G: Into<bool>>
		(&mut self, first_name: S1, last_name: S2, gender: G, subject: S3, classes: S4, hod: bool, password: &str)
		-> Result<u64, pgError> {
		let firstname = first_name.into();
		let lastname = last_name.into();
		try!( self.database.exec(
			"INSERT INTO Users (ID, first_name, last_name, gender, username, password) VALUES ($1, $2, $3, $4, $5, $6)", 
			&[&self.cnts.usrcnt, &firstname, &lastname, &gender.into(), &format!("{}.{}", firstname.to_lowercase(), lastname.to_lowercase()), &encrypt(password)]
		) );
		try!( self.database.exec(
			"INSERT INTO Teachers (ID, subject, classes, hod) VALUES ($1, $2, $3, $4)",
			&[&self.cnts.usrcnt, &subject.into(), &classes.into(), &hod]
		) );
		self.cnts.incr_teachers();
		Ok(2)
	}

	pub fn insert(&mut self, user: IndusUser, password: &str) -> Result<u64, pgError> {
		match user.role {
			StudentTeacher::Student(stdnt) => 
				self.insert_student(
					user.first_name, user.last_name, user.gender, 
					Classes::to_student(stdnt.classes), stdnt.grade, stdnt.section.to_string(), password),
			StudentTeacher::Teacher(tchr) => 
				self.insert_teacher(
					user.first_name, user.last_name, user.gender,
					tchr.subject, Classes::to_teacher(tchr.classes), tchr.hod, password)
		}
	}

	pub fn clear(&self) -> Result<u64, pgError> {
		self.database.exec("TRUNCATE users CASCADE", &[])
	}
}

use std::ops::Drop;
impl Drop for IndusDatabase {
	fn drop(&mut self) {
		write!(&mut File::create("props.cfg").unwrap(), "{}", encode(&self.cnts).unwrap());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use test::Bencher;
	use std::fs::File;
	use rustc_serialize::json::encode;
	use std::io::Write;

	#[test]
	fn it_works() {
		let mut database = IndusDatabase::new();
		database.clear();
		write!(&mut File::create("props.cfg").unwrap(), "{}", encode(&Counts::new()).unwrap());

		println!("Testing Anshuman Login (NoAcc)");
		assert_eq!( database.login("anshuman.medhi", "2cool4uuu"), Err(LoginFailure::NoAccount) );
		println!("Testing Hari Prasad Login (NoAcc)");
		assert_eq!( database.login("hari.prasad", "killthelion"), Err(LoginFailure::NoAccount) );

		println!("Testing Anshuman Insert");
		database.insert_student(
			"Anshuman", "Medhi", Gender::Male, 
			"C1 English SL, C2 Spanish SL, C3 Math HL, C4 Economics SL, C5 Chemistry HL, C6 Physics HL",
			11, "B", "2cool4uuu"
		).unwrap();
		println!("Testing Hari Prasad Insert");
		database.insert_teacher(
			"Hari", "Prasad", Gender::Male,
			"Economics", "Wiggle Wiggle Wiggle Wiggle Wiggle YEAH", false,
			"killthelion"
		).unwrap();

		println!("Testing Anshuman Login (Succ)");
		database.login("anshuman.medhi", "2cool4uuu").unwrap();
		println!("Testing Hari Prasad Login (Succ)");
		database.login("hari.prasad", "killthelion").unwrap();

		println!("Testing Anshuman Login (PwdMismatch)");
		assert_eq!( database.login("anshuman.medhi", "coolpoopbags"), Err(LoginFailure::PasswordMismatch) );
		println!("Testing Hari Prasad Login (PwdMismatch)");
		assert_eq!( database.login("hari.prasad", "aosdjbd"), Err(LoginFailure::PasswordMismatch) );

		println!("Testing Anshuman InsertObj (Succ)");
		database.insert(IndusUser {
			first_name: "Anshuman".into(),
			last_name: "Medhi".into(),
			gender: Gender::Male,
			role: StudentTeacher::Student(IndusStudent {
				classes: "C1 English SL, C2 Spanish SL, C3 Math HL, C4 Economics SL, C5 Chemistry HL, C6 Physics HL".into(),
				grade: 11, section: 'B'
			})
		}, "2cool4uuu").unwrap();
		println!("Testing Hari Prasad InsertObj (Succ)");
		database.insert(IndusUser {
			first_name: "Hari".into(),
			last_name: "Prasad".into(),
			gender: Gender::Male,
			role: StudentTeacher::Teacher(IndusTeacher {
				subject: "Economics".into(),
				classes: "Wiggle Wiggle Wiggle Wiggle Wiggle YEAH".into(), hod: false
			})
		}, "killthelion").unwrap();

		println!("Testing Anshuman Login (DupAcc)");
		assert_eq!( database.login("anshuman.medhi", "2cool4uuu"), Err(LoginFailure::DuplicateAccounts) );
		println!("Testing Hari Prasad Login (DupAcc)");
		assert_eq!( database.login("hari.prasad", "killthelion"), Err(LoginFailure::DuplicateAccounts) );
	}

	// #[bench]
	// fn bench_insert_student(b: &mut Bencher) {
	// 	let mut database = IndusDatabase::new();
	// 	b.iter(|| {
	// 		database.insert_student(
	// 			"Anshuman", "Medhi", Gender::Male, 
	// 			"C1 English SL, C2 Spanish SL, C3 Math HL, C4 Economics SL, C5 Chemistry HL, C6 Physics HL",
	// 			11, "B", "2cool4uuu"
	// 		);
	// 	})
	// }

	// #[bench]
	// fn bench_insert_student2(b: &mut Bencher) {
	// 	let mut database = IndusDatabase::new();
	// 	b.iter(|| {
	// 		database.insert(IndusUser {
	// 			first_name: "Anshuman".into(),
	// 			last_name: "Medhi".into(),
	// 			gender: Gender::Male,
	// 			role: StudentTeacher::Student(IndusStudent {
	// 				classes: "C1 English SL, C2 Spanish SL, C3 Math HL, C4 Economics SL, C5 Chemistry HL, C6 Physics HL".into(),
	// 				grade: 11, section: 'B'
	// 			})
	// 		}, "2cool4uuu")
	// 	})
	// }

	// #[bench]
	// fn bench_insert_teacher(b: &mut Bencher) {
	// 	let mut database = IndusDatabase::new();
	// 	b.iter(|| {
	// 		database.insert_teacher(
	// 			"Hari", "Prasad", Gender::Male,
	// 			"Economics", "Wiggle Wiggle Wiggle Wiggle Wiggle YEAH", false,
	// 			"killalion"
	// 		);
	// 	})
	// }

	// #[bench]
	// fn bench_insert_teacher2(b: &mut Bencher) {
	// 	let mut database = IndusDatabase::new();
	// 	b.iter(|| {
	// 		database.insert(IndusUser {
	// 			first_name: "Hari".into(),
	// 			last_name: "Prasad".into(),
	// 			gender: Gender::Male,
	// 			role: StudentTeacher::Teacher(IndusTeacher {
	// 				subject: "Economics".into(),
	// 				classes: "Wiggle Wiggle Wiggle Wiggle Wiggle YEAH".into(), hod: false
	// 			})
	// 		}, "killthelion")
	// 	})
	// }

	// #[bench]
	// fn bench_login_anshuman(b: &mut Bencher) {
	// 	let mut database = IndusDatabase::new();
	// 	b.iter(|| {
	// 		database.login("anshuman.medhi", "2cool4uuu")
	// 	})
	// }

	// #[bench]
	// fn bench_login_teacher(b: &mut Bencher) {
	// 	let mut database = IndusDatabase::new();
	// 	b.iter(|| {
	// 		database.login("hari.prasad", "killthelion")
	// 	})
	// }

	// #[bench]
	// fn bench_baseline(b: &mut Bencher) {
	// 	b.iter(||{})
	// }
}