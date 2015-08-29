use std::str::FromStr;

#[derive(Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub struct Counts {
	pub usrcnt: i32, stdcnt: i32, tchcnt: i32,
}

impl Counts {
	#[inline] pub fn new() -> Counts {
		Counts {
			usrcnt: 0, stdcnt: 0, tchcnt: 0,
		}
	}
	#[inline] pub fn from(u: i32, s: i32, t: i32) -> Counts {
		Counts {
			usrcnt: u, stdcnt: s, tchcnt: t,
		}
	}
	pub fn incr_students(&mut self) {
		self.usrcnt += 1;
		self.stdcnt += 1;
		println!("{:?}", self);
	}
	pub fn incr_teachers(&mut self) {
		self.usrcnt += 1;
		self.tchcnt += 1;
		println!("{:?}", self);
	}
}

#[derive(Debug, Clone, Copy, PartialEq, RustcEncodable, RustcDecodable)]
pub enum Gender {
	Male, Female
}
impl Into<bool> for Gender {
	#[inline] fn into(self) -> bool {
		match self {
			Gender::Male => true,
			Gender::Female => false,
		}
	}
}
impl From<bool> for Gender {
	#[inline] fn from(b: bool) -> Gender {
		if b { Gender::Male }
		else { Gender::Female }
	}
}

#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub struct IndusUser {
	pub first_name: String,
	pub last_name: String,
	pub gender: Gender,
	pub role: StudentTeacher,
}
#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub enum StudentTeacher {
	Student(IndusStudent),
	Teacher(IndusTeacher)
}
#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub struct IndusStudent {
	pub classes: Vec<Class>,
	pub grade: i16,
	pub section: char,
}
#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub struct IndusTeacher {
	pub subject: String,
	pub classes: Vec<Class>,
	pub hod: bool
}

#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub enum Subject {
	Economics, Business,
	Physics, Chemistry, Biology,
	CompSci, ICT, ITGS,
	Psychology,
	English, ESL,
	Spanish, French, German, Hindi,
	Art, Music
}

pub struct InvalidSubject;

impl FromStr for Subject {
	type Err = InvalidSubject;
	fn from_str(s: &str) -> Result<Subject, InvalidSubject> {
		Ok( match s {
			"Economics" => Subject::Economics,
			"Business" => Subject::Business,
			"Physics" => Subject::Physics,
			"Chemistry" => Subject::Chemistry,
			"Biology" => Subject::Biology,
			"CompSci" => Subject::CompSci,
			"ICT" => Subject::ICT,
			"ITGS" => Subject::ITGS,
			"Psychology" => Subject::Psychology,
			"English" => Subject::English,
			"ESL" => Subject::ESL,
			"Spanish" => Subject::Spanish,
			"French" => Subject::French,
			"German" => Subject::German,
			"Hindi" => Subject::Hindi,
			"Art" => Subject::Art,
			"Music" => Subject::Music,
			_ => return Err(InvalidSubject)
		} )
	}
}

impl Into<String> for Subject {
	fn into(self) -> String {
		match self {
			Subject::Economics => "Economics",
			Subject::Business => "Business",
			Subject::Physics => "Physics",
			Subject::Chemistry => "Chemistry",
			Subject::Biology => "Biology",
			Subject::CompSci => "CompSci",
			Subject::ICT => "ICT",
			Subject::ITGS => "ITGS",
			Subject::Psychology => "Psychology",
			Subject::English => "English",
			Subject::ESL => "ESL",
			Subject::Spanish => "Spanish",
			Subject::French => "French",
			Subject::German => "German",
			Subject::Hindi => "Hindi",
			Subject::Art => "Art",
			Subject::Music => "Music",
		}.into()
	}
}

impl Subject {
	#[inline] fn tostr(self) -> String {
		Into::<String>::into(self)
	}
}

#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Class {
	pub subject: Subject, pub teacher: String, pub block: String, pub grade: i16
}

pub struct InvalidClass;

pub struct Classes;
impl Classes {
	pub fn from_student(grade: i16, src: &str) -> Vec<Class> {
		src.split(",").flat_map(|s| {
			let mut values = s.split("-").map(str::trim);
			Some( Class {
				block: match values.next() {
					Some(s) => s.into(),
					None => return None
				},
				subject: match values.next() {
					Some(s) => match s.parse() {
						Ok(o) => o,
						Err(_) => return None,
					},
					None => return None
				},
				teacher: match values.next() {
					Some(s) => s.into(),
					None => return None
				},
				grade: grade,
			} )
		}).collect()
	}

	pub fn to_student(from: Vec<Class>) -> String {
		from.iter()
			.map(|elem| format!("{} - {} - {}", elem.block, elem.subject.clone().tostr(), elem.teacher))
			.collect::<Vec<_>>()
			.connect(", ")
	}

	pub fn from_teacher(teacher: &str, subject: &str, src: &str) 
		-> Vec<Class> {
		src.split(",").flat_map(|s| {
			let mut values = s.split("-").map(str::trim);
			Some( Class {
				block: match values.next() {
					Some(s) => s.into(),
					None => return None
				},
				subject: match subject.parse() {
					Ok(o) => o,
					Err(_) => return None,
				},
				teacher: teacher.into(),
				grade: match values.next() {
					Some(s) => match s.parse() {
						Ok(o) => o,
						Err(_) => return None,
					},
					None => return None
				},
			} )
		}).collect()
	}

	pub fn to_teacher(from: Vec<Class>) -> String {
		from.iter().map(|elem| format!("{} {}", elem.block, elem.grade)).collect::<Vec<_>>().connect(", ")
	}
}

impl FromStr for Class {
	type Err = InvalidClass;
	fn from_str(s: &str) -> Result<Class, InvalidClass> {
		let mut values = s.split("-").map(str::trim);
		Ok( Class {
			subject: match values.next() {
				Some(s) => match s.parse() {
					Ok(o) => o,
					Err(_) => return Err(InvalidClass),
				},
				None => return Err(InvalidClass)
			},
			teacher: match values.next() {
				Some(s) => s.into(),
				None => return Err(InvalidClass)
			},
			block: match values.next() {
				Some(s) => s.into(),
				None => return Err(InvalidClass)
			},
			grade: match values.next() {
				Some(s) => match s.parse() {
					Ok(o) => o,
					Err(_) => return Err(InvalidClass),
				},
				None => return Err(InvalidClass)
			},
		} )
	}
}