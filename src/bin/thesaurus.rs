use rain_disaster_2::{LangFile, Translate};
use rand::prelude::SliceRandom;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ThesaurusEntry {
	pub word: String,
	pub synonyms: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct Thesaurize(LangFile);

impl Translate for Thesaurize {
	const LANGUAGE_JSON: &'static str = include_str!("thesaurus.json");

	fn translate(&mut self) -> anyhow::Result<()> {
		let mut rng = rand::thread_rng();
		for (key, value) in &mut self.0.strings {
			if key.contains("FORMAT")
				|| key.contains("LORE")
				|| self.0.goal_strings.contains_key(key)
			{
				continue;
			}
			let original = value.clone();
			let mut applicable: Vec<String> = THESAURUS
				.clone()
				.into_par_iter()
				.filter(|(tkey, _)| value.to_lowercase().contains(tkey) && tkey.len() > 3)
				.map(|(tkey, _)| tkey)
				.collect();
			*value = value
				.chars()
				.fold((String::new(), false), |acc, c| {
					if c == '<' {
						(acc.0, true)
					} else if c == '>' {
						(acc.0, false)
					} else if acc.1 {
						(acc.0, acc.1)
					} else {
						(acc.0 + &c.to_string(), acc.1)
					}
				})
				.0;
			applicable.shuffle(&mut rng);
			*value = value.to_lowercase();
			for synonym in &applicable {
				let to = THESAURUS
					.get(synonym)
					.unwrap()
					.choose(&mut rng)
					.unwrap_or(synonym);
				*value = value.replace(synonym, &to);
			}
			println!("{} -> {}", original, value);
			// if applicable.len() > 0 {
			// println!("{}", applicable.join(", "));
			// }
		}
		Ok(())
	}

	fn put_cache_strings(
		&mut self,
		cache_strings: &std::collections::HashMap<String, String>,
	) -> anyhow::Result<()> {
		self.0.goal_strings = cache_strings.clone();
		Ok(())
	}
}

lazy_static::lazy_static! {
	static ref THESAURUS: std::collections::HashMap<String, Vec<String>> = {
		let mut thesaurus = std::collections::HashMap::new();
		let mut contents = include_str!("thesaurus.jsonl").lines();
		let entries: Vec<ThesaurusEntry> = contents.by_ref().flat_map(|line| {
			let entry: Option<ThesaurusEntry> = serde_json::from_str(line).map(|v| Some(v)).unwrap_or(None);
			entry
		}).collect();
		for entry in entries {
			thesaurus.insert(entry.word, entry.synonyms);
		}
		thesaurus
	};
}

fn main() {
	rain_disaster_2::go::<Thesaurize>("en", "thes").unwrap();
}
