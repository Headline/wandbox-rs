use std::error::Error;

use std::collections::HashMap;

use crate::{Language, Compiler};

pub type CompilerCache = HashMap<String, Language>;

pub async fn load() -> Result<CompilerCache, Box<dyn Error>> {
    // grab wandbox compilers
    let res = reqwest::get("https://wandbox.org/api/list.json").await?;
    // retrieve compilers as vector
    let result : Vec<Compiler> = res.json().await?;

    // we have to build our cache, iterating our vector and organizing
    // compilers by their language. The language id should be lowercase.
    let mut comp_cache : CompilerCache = HashMap::new();
    for c in result {
        let language_name = c.language.to_ascii_lowercase();

        // see if we can grab a mutable |Language|
        let entry = comp_cache.get_mut(&language_name);
        match entry {
            Some(p) => {
                p.compilers.push(c);
            }

            // create one then..
            None => {
                let mut lang = Language {
                    name : language_name.clone(),
                    compilers : Vec::new()
                };
                lang.compilers.push(c);
                comp_cache.insert(language_name, lang);
            }
        }
    }

    Ok(comp_cache)
}