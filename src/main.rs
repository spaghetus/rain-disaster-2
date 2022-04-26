use std::{collections::HashMap, io::Read, time::Duration};
#[macro_use]
extern crate serde;
use rand::prelude::SliceRandom;
#[macro_use]
extern crate serde_json;
use rayon::prelude::*;

const LANGS: &str = include_str!("langs.txt");
const TRANSLATION_COUNT: usize = 3;
const SOURCE_LANG: &str = "en";
const DELAY: f64 = 0.5;
const API_ENDPOINT: &str = "http://127.0.0.1:5000/translate";

#[derive(Serialize, Deserialize)]
pub struct LangFile {
	pub strings: HashMap<String, String>,
}
impl LangFile {
	pub fn translate(&mut self) {
		let langs: Vec<&str> = LANGS.lines().collect();
		self.strings.iter_mut().for_each(|(key, text)| {
			let text = text
				.chars()
				.fold((String::new(), false), |mut acc, c| {
					if c == '<' {
						(acc.0, true)
					} else if c == '>' {
						(acc.0, false)
					} else if !acc.1 {
						(acc.0 + &c.to_string(), false)
					} else {
						acc
					}
				})
				.0
				.chars()
				.collect::<String>();
			let original_length = text.len();
			let client = reqwest::blocking::Client::new();
			let mut rng = rand::thread_rng();
			let mut last_lang = SOURCE_LANG;
			for _ in 0..TRANSLATION_COUNT {
				let next_lang = langs.choose(&mut rng).unwrap();
				loop {
					let result = client
						.post(API_ENDPOINT)
						.json(&json!({
							"source": last_lang,
							"target": next_lang.to_string(),
							"format": "text",
							"q": text.clone(),
						}))
						.send();
					if let Ok(result) = result {
						let result: HashMap<String, String> =
							serde_json::from_str(&result.text().unwrap()).unwrap();
						let new = result.get("translatedText");
						if let Some(new) = new {
							*text = new.to_string().chars().take(original_length).collect();
						}
						break;
					} else {
						eprintln!("{}", result.unwrap_err());
						std::thread::sleep(Duration::from_secs_f64(DELAY));
					}
				}
				last_lang = *next_lang;
				std::thread::sleep(Duration::from_secs_f64(DELAY));
			}
			loop {
				let result = client
					.post(API_ENDPOINT)
					.json(&json!({
						"source": last_lang,
						"target": SOURCE_LANG.to_string(),
						"q": text.clone(),
						"format": "text",
					}))
					.send();
				if let Ok(result) = result {
					let result: HashMap<String, String> =
						serde_json::from_str(&result.text().unwrap()).unwrap();
					let new = result.get("translatedText");
					if let Some(new) = new {
						*text = new.to_string().chars().take(original_length * 2).collect();
					}
					break;
				} else {
					eprintln!("{}", result.unwrap_err());
					std::thread::sleep(Duration::from_secs_f64(DELAY));
				}
			}
			eprintln!("{}: {}", key, text.clone());
		});
	}
}

fn main() {
	assert!(std::path::Path::new("./ror2-lang/en/Items.txt").exists());
	eprintln!("Creating rd2 language directory.");
	std::fs::create_dir_all("./ror2-lang/rd2").unwrap();
	eprintln!("Translating language files.");
	for file in std::fs::read_dir("./ror2-lang/en")
		.unwrap()
		.inspect(|v| eprintln!("{:?}", v))
		.flatten()
		.filter(|v| v.file_name().to_str().unwrap().ends_with("txt"))
	{
		if std::path::Path::new(&format!(
			"./ror2-lang/rd2/{}",
			file.file_name().to_str().unwrap()
		))
		.exists()
		{
			continue;
		}
		eprintln!("Translating {}", file.path().to_str().unwrap());
		// Load file
		let mut lang_file = vec![];
		std::fs::File::open(file.path())
			.unwrap()
			.read_to_end(&mut lang_file)
			.unwrap();
		// Convert to string (for whatever reason fs::read_to_string doesn't work)
		let lang_file = String::from_utf8_lossy(&lang_file).to_string();
		// Deserialize
		let mut lang_file: LangFile = json5::from_str(&lang_file).unwrap();
		lang_file.translate();
		std::fs::write(
			format!("./ror2-lang/rd2/{}", file.file_name().to_str().unwrap()),
			json5::to_string(&lang_file).unwrap(),
		)
		.unwrap()
	}
	for file in std::fs::read_dir("./ror2-lang/en")
		.unwrap()
		.flatten()
		.filter(|v| {
			v.file_name().to_str().unwrap().ends_with("json")
				|| v.file_name().to_str().unwrap().ends_with("png")
		}) {
		std::fs::copy(
			file.path(),
			format!("./ror2-lang/rd2/{}", file.file_name().to_str().unwrap()),
		)
		.unwrap();
	}
	std::fs::copy("./language.json", "./ror2-lang/rd2/language.json").unwrap();
}
