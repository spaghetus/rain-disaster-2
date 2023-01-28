use indicatif::{ProgressBar, ProgressStyle};
use rain_disaster_2::{LangFile, Translate};
use rand::prelude::SliceRandom;
use rust_bert::pipelines::translation::{Language, TranslationModelBuilder};
use std::{collections::HashMap, io::Read};

const LANGS: &[Language] = &include!("langs.txt");
const TRANSLATION_COUNT: usize = 3;
const SOURCE_LANG: Language = Language::English;

struct Translator<'a>(&'a mut LangFile);

impl Translate for Translator<'_> {
	fn translate(&mut self) -> anyhow::Result<()> {
		let to_model = TranslationModelBuilder::new()
			.with_source_languages(vec![Language::English])
			.with_target_languages(LANGS.to_vec())
			.create_model()
			.unwrap();
		let from_model = TranslationModelBuilder::new()
			.with_source_languages(LANGS.to_vec())
			.with_target_languages(vec![Language::English])
			.create_model()
			.unwrap();
		let count = self.0.strings.len();
		let progress = ProgressBar::new(count as u64).with_style(
			ProgressStyle::default_bar().template("{elapsed}/{duration} - {msg} - {wide_bar}"),
		);
		self.0.strings.iter_mut().for_each(|(key, text)| {
			if key.contains("FORMAT")
				|| key.contains("LORE")
				|| self.0.goal_strings.contains_key(key)
			{
				return;
			}
			let mut rng = rand::thread_rng();
			let original_text = text.clone();
			for _ in 0..TRANSLATION_COUNT {
				let next_lang = LANGS.choose(&mut rng).unwrap();
				// Translate to the target language and back to english
				*text = to_model
					.translate(&[&text.clone()], Some(SOURCE_LANG), Some(*next_lang))
					.unwrap()[0]
					.clone();
				progress.set_message(text.clone());
				*text = from_model
					.translate(&[text.clone()], Some(*next_lang), Some(SOURCE_LANG))
					.unwrap()[0]
					.clone();
				progress.set_message(text.clone());
			}
			progress.set_message(text.clone());
			progress.inc(1);
		});
		Ok(())
	}

	const LANGUAGE_JSON: &'static str = include_str!("../language.json");

	fn put_cache_strings(&mut self, cache_strings: &HashMap<String, String>) -> anyhow::Result<()> {
		self.0.goal_strings = cache_strings.clone();
		Ok(())
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
		if std::path::Path::new(&format!(
			"./ror2-lang/rd2/{}",
			file.file_name().to_str().unwrap()
		))
		.exists()
		{
			lang_file.goal_strings = json5::from_str::<HashMap<String, HashMap<String, String>>>(
				&std::fs::read_to_string(&format!(
					"./ror2-lang/rd2/{}",
					file.file_name().to_str().unwrap()
				))
				.unwrap(),
			)
			.unwrap()
			.get("strings")
			.unwrap()
			.clone();
		}
		Translator(&mut lang_file).translate().unwrap();
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
