use rain_disaster_2::{LangFile, Translate};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct Uwu(LangFile);

impl Translate for Uwu {
	const LANGUAGE_JSON: &'static str = include_str!("uwu.json");

	fn translate(&mut self) -> anyhow::Result<()> {
		for (key, value) in &mut self.0.strings {
			if key.contains("FORMAT")
				|| key.contains("LORE")
				|| self.0.goal_strings.contains_key(key)
			{
				continue;
			}
			let original = value.clone();
			let filtered_value = value
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
			*value = uwuifier::uwuify_str_sse(&filtered_value);
			println!("{} -> {}", original, value);
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

fn main() {
	rain_disaster_2::go::<Uwu>("en", "uwu").unwrap();
}
