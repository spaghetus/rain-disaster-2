use std::{collections::HashMap, io::Read};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub trait Translate {
	const LANGUAGE_JSON: &'static str;

	fn translate(&mut self) -> anyhow::Result<()>;

	fn put_cache_strings(&mut self, cache_strings: &HashMap<String, String>) -> anyhow::Result<()>;
}

pub fn go<'a, T: Translate + Serialize + DeserializeOwned>(
	src_lang_dir: &str,
	to_lang_dir: &str,
) -> anyhow::Result<()> {
	assert!(std::path::Path::new(&format!("./ror2-lang/{}/Items.txt", src_lang_dir)).exists());
	eprintln!("Creating language directory.");
	std::fs::create_dir_all(format!("./ror2-lang/{}", to_lang_dir)).unwrap();
	eprintln!("Translating language files.");
	for file in std::fs::read_dir(format!("./ror2-lang/{}", src_lang_dir))
		.unwrap()
		.inspect(|v| eprintln!("{:?}", v))
		.flatten()
		.filter(|v| v.file_name().to_str().unwrap().ends_with("txt"))
	{
		eprintln!("Translating {}", file.path().to_str().unwrap());
		// Load file
		let mut lang_file = vec![];
		std::fs::File::open(file.path())
			.unwrap()
			.read_to_end(&mut lang_file)
			.unwrap();
		// Convert to string (for whatever reason fs::read_to_string doesn't work)
		let lang_file_str = String::from_utf8_lossy(&lang_file).to_string();
		// Deserialize
		let mut lang_file: T = json5::from_str(&lang_file_str).unwrap();
		if std::path::Path::new(&format!(
			"./ror2-lang/{}/{}",
			to_lang_dir,
			file.file_name().to_str().unwrap()
		))
		.exists()
		{
			lang_file.put_cache_strings(
				&json5::from_str::<HashMap<String, HashMap<String, String>>>(
					&std::fs::read_to_string(&format!(
						"./ror2-lang/{}/{}",
						to_lang_dir,
						file.file_name().to_str().unwrap()
					))
					.unwrap(),
				)
				.unwrap()
				.get("strings")
				.unwrap(),
			)?;
		}
		lang_file.translate()?;
		std::fs::write(
			format!(
				"./ror2-lang/{}/{}",
				to_lang_dir,
				file.file_name().to_str().unwrap()
			),
			json5::to_string(&lang_file).unwrap(),
		)
		.unwrap()
	}
	for file in std::fs::read_dir(format!("./ror2-lang/{}", src_lang_dir))
		.unwrap()
		.flatten()
		.filter(|v| {
			v.file_name().to_str().unwrap().ends_with("json")
				|| v.file_name().to_str().unwrap().ends_with("png")
		}) {
		std::fs::copy(
			file.path(),
			format!(
				"./ror2-lang/{}/{}",
				to_lang_dir,
				file.file_name().to_str().unwrap()
			),
		)
		.unwrap();
	}
	std::fs::write(
		format!("./ror2-lang/{}/language.json", to_lang_dir),
		T::LANGUAGE_JSON,
	)?;
	Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct LangFile {
	pub strings: HashMap<String, String>,
	#[serde(skip)]
	pub goal_strings: HashMap<String, String>,
}
