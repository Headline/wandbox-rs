mod tests;
mod cache;

use core::fmt;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use crate::cache::CompilerCache;
use std::sync::{RwLock, Arc};
use std::error::Error;

use std::collections::HashSet;

/// The main cache that holds on to the compiler cache
pub struct Wandbox {
    cache : Arc<RwLock<CompilerCache>>,
}
impl Wandbox {
    /// Initializes the cache for Wandbox requests
    ///
    /// You may also choose to block certain compilers or languages from being supported.
    /// This is useful if wandbox has any issues with certain compilers or languages.
    ///
    /// # Arguments
    /// * `comps` - A vector of compiler identifiers that the library should ignore
    /// * `langs` - A vector of language identifiers that the library should ignore
    /// # Example
    /// ```edition2018
    ///use std::collections::HashSet;
    ///use wandbox::Wandbox;
    ///
    ///#[tokio::main]
    ///async fn main() {
    ///    let mut set : HashSet<String> = HashSet::new();
    ///    set.insert(String::from("gcc-head"));
    ///
    ///    let wbox : Wandbox = match Wandbox::new(Some(set), None).await {
    ///        Ok(wbox) => wbox,
    ///        Err(e) => return println!("{}", e)
    ///    };
    /// }
    ///```
    pub async fn new(comps : Option<HashSet<String>>, langs : Option<HashSet<String>>) -> Result<Wandbox, Box<dyn Error>> {

        let mut cache : CompilerCache = cache::load().await?;

        if let Some(langs) = langs {
            cache = cache.into_iter().filter(|(_x, v)| !langs.contains(&v.name)).collect();
        }

        if let Some(comps) = comps {
            for (_k, v) in cache.iter_mut() {
                for str in &comps {
                    v.remove_compiler(str);
                }
            }
        }

        // adjust language names to lower
        for (_k, v) in cache.iter_mut() {
            for mut c in v.compilers.iter_mut() {
                c.language = c.language.to_ascii_lowercase();
            }
        }

        Ok(Wandbox {
            cache: Arc::new(RwLock::new(cache))
        })
    }

    /// Gets a list of compilers given a certain language
    ///
    /// # Arguments
    /// * `lang` - The language identifier to return the compilers for
    pub fn get_compilers(&self, lang : &str) -> Option<Vec<Compiler>> {
        let lock = self.cache.read().unwrap();
        let language_option = lock.get(lang);
        let lang = match language_option {
            Some(l) => l,
            None => return None
        };

        Some(lang.compilers.clone())
    }

    /// Returns a list of every language
    pub fn get_languages(&self) -> Vec<Language> {
        let lock = self.cache.read().unwrap();

        let mut vec : Vec<Language> = Vec::new();
        for (_k, v) in lock.iter() {
            vec.push(v.clone());
        }
        vec
    }

    /// Determines if the compiler string supplied is a valid compiler
    ///
    /// # Arguments
    /// * `c` - compiler identifier to check for
    //  n^2 :(
    pub fn is_valid_compiler_str(&self, c : &str) -> bool {
        // aquire our lock
        let lock = self.cache.read().unwrap();
        for (_l, k) in lock.iter() {
            for v in k.compilers.iter() {
                if v.name == c {
                    return true;
                }
            }
        }

        return false;
    }

    pub fn get_compiler_language_str(&self, c : &str) -> Option<String> {
        // aquire our lock
        let lock = self.cache.read().unwrap();

        for (_k, v) in lock.iter() {
            for comp in &v.compilers {
                if comp.name == c {
                    return Some(v.name.clone());
                }
            }
        }

        return None;
    }

    pub fn is_valid_language(&self, l : &str) -> bool {
        let lock = self.cache.read().unwrap();
        return lock.get(l).is_some();
    }

    pub fn get_default_compiler(&self, l : &str) -> Option<String> {
        let lock = self.cache.read().unwrap();
        if let Some(lang) = lock.get(l) {
            Some(lang.compilers.get(0).expect("awd").name.clone())
        }
        else {
            None
        }
    }
}

/// Representation of a compiler
#[derive(Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Compiler {
    #[serde(rename = "compiler-option-raw")]
    pub compiler_option_raw : bool,
    #[serde(rename = "display-compile-command")]
    pub display_compile_command : String,
    #[serde(rename = "runtime-option-raw")]
    pub runtime_option_raw : bool,

    pub version : String,
    pub language : String,
    pub name : String,
}
impl Clone for Compiler {
    fn clone(&self) -> Self {
        Compiler {
            compiler_option_raw : self.compiler_option_raw,
            display_compile_command : self.display_compile_command.clone(),
            runtime_option_raw : self.runtime_option_raw,
            version : self.version.clone(),
            language : self.language.clone(),
            name : self.name.clone(),
        }
    }
}
impl fmt::Debug for Compiler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{} {}] : {}", self.name, self.version, self.language)
    }
}

/// A builder to allow you to easily build requests
///
/// ```edition2018
///use std::collections::HashSet;
///use tokio::macros::*;
///use std::error::Error;
///use wandbox::{Wandbox, CompilationBuilder};
///#[tokio::main]
///async fn main() {
///    let wbox : Wandbox = match Wandbox::new(None, None).await {
///        Ok(wbox) => wbox,
///        Err(e) => return println!("{}", e)
///    };
///    let mut builder = CompilationBuilder::new();
///    builder.target("gcc-6.3.0");
///    builder.options_str(vec!["-Wall", "-Werror"]);
///    builder.code("#include<iostream>\nint main()\n{\nstd::cout<<\"test\";\n}");
///    let result = match builder.build(&wbox) {
///        Ok(res) => res,
///        Err(e) => return println!("{}", e)
///    };
///}
/// ```
#[derive(Default, Serialize)]
pub struct CompilationBuilder {
    #[serde(skip)]
    target : String,
    pub lang : String,
    compiler : String,
    code : String,
    stdin : String,
    #[serde(skip)]
    options : Vec<String>,
    #[serde(rename = "compiler-option-raw")]
    compiler_options_raw : String,
    save : bool
}
impl CompilationBuilder {
    /// Creates a new CompilationBuilder with default values to be filled in later
    pub fn new() -> CompilationBuilder {
        return CompilationBuilder { ..Default::default()}
    }

    /// Sets the target of the compilation
    ///
    /// # Arguments
    /// * `target` - The target of a compilation, this can be a language ('c++'), or a compiler ('gcc-head')
    pub fn target(&mut self, target : &str) -> () {
        self.target = target.trim().to_string();
    }

    /// Sets the code to be compiled
    ///
    /// # Arguments
    /// * `code` - String of code to be compiled
    pub fn code(&mut self, code : &str)  -> () {
        self.code = code.trim().to_string();
    }

    /// Sets the stdin to directed towards the application
    ///
    /// # Arguments
    /// * `stdin` - program input
    pub fn stdin(&mut self, stdin : &str) -> () {
        self.stdin = stdin.trim().to_string();
    }

    /// Determines whether or not Wandbox saves the compilation & replies with a link for you
    ///
    /// # Arguments
    /// * `save` - true if Wandbox should save this compilation
    pub fn save(&mut self, save : bool) -> () {
        self.save = save;
    }

    /// Sets the list of compilation options. Useful for languages like c++ to pass linker/optimization
    /// flags.
    ///
    /// # Arguments
    /// * `options` - A list of compiler options i.e ["-Wall", "-Werror"]
    pub fn options(&mut self, options : Vec<String>) -> () {
        self.options = options;
    }

    /// Sets the list of compilation options. Useful for languages like c++ to pass linker/optimization
    /// flags.
    ///
    /// This version allows you to pass a `Vec<&str>`
    ///
    /// # Arguments
    /// * `options` - A list of compiler options i.e ["-Wall", "-Werror"]
    pub fn options_str(&mut self, options : Vec<&str>) -> () {
        self.options = options.into_iter().map(|f| f.to_owned()).collect();
    }

    /// Finalizes the builder & prepares itself for compilation dispatch.
    ///
    /// # Arguments
    /// * `wb` - An instance of the Wandbox cache to resolve the compilation target
    pub fn build(&mut self, wb : &Wandbox) -> Result<(), WandboxError> {
        self.compiler_options_raw = self.options.join("\n");

        if wb.is_valid_language(&self.target) {
            let comp = match wb.get_default_compiler(&self.target) {
                Some(def) => def,
                None => return Err(WandboxError::new("Unable to determine default compiler for input language"))
            };
            self.compiler = comp;
            self.lang = self.target.clone();
        }
        else if wb.is_valid_compiler_str(&self.target) {
            let lang = match wb.get_compiler_language_str(&self.target) {
                Some(lang) => lang,
                None => return Err(WandboxError::new("Unable to determine language for compiler}"))
            };

            self.lang = lang;
            self.compiler = self.target.clone();
        }
        else {
            return Err(WandboxError::new("Unable to find compiler or language for target"));
        }
        Ok(())
    }

    /// Dispatches the built request to Wandbox
    pub async fn dispatch(&self) -> Result<CompilationResult, WandboxError> {
        let client = reqwest::Client::new();

        let result = client.post("https://wandbox.org/api/compile.json")
            .json(&self)
            .header("Content-Type", "application/json; charset=utf-8")
            .send().await;

        let response = match result {
            Ok(r) => r,
            Err(e) => return Err(WandboxError::new(&format!("{}", e)))
        };

        let status_code = response.status().clone();
        let res : CompilationResult = match response.json().await {
            Ok(res) => res,
            Err(_e) => return Err(WandboxError::new(&format!("Wandbox replied with: {}\n\
            This could mean WandBox is experiencing an outage, or a network connection error has occured", status_code)))
        };
        return Ok(res);
    }
}

/// Information regarding the result of a compilation request.
#[derive(Default, Deserialize)]
pub struct CompilationResult {
    #[serde(default)]
    pub status : String,
    #[serde(default)]
    pub signal : String,
    #[serde(rename = "compiler_output", default)]
    pub compiler_stdout : String,
    #[serde(rename = "compiler_error", default)]
    pub compiler_stderr : String,
    #[serde(rename = "compiler_message", default)]
    pub compiler_all : String,
    #[serde(rename = "program_output", default)]
    pub program_stdout : String,
    #[serde(rename = "program_error", default)]
    pub program_stderr : String,
    #[serde(rename = "program_message", default)]
    pub program_all : String,
    #[serde(default)]
    pub permlink : String,
    #[serde(default)]
    pub url : String,
}

impl fmt::Debug for CompilationResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{} {}] {}: {}", self.status, self.signal, self.compiler_all, self.program_all)
    }
}


/// A representation of a language with a list of it's compilers
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct Language {
    pub name : String,
    pub compilers : Vec<Compiler>
}

impl Language {
    fn remove_compiler(&mut self, str : &str) {
        let mut copy = self.compilers.clone();
        copy = copy.into_iter().filter(|v| v.name != str).collect();
        self.compilers = copy;
    }
}


#[derive(Debug)]
pub struct WandboxError {
    details: String
}

impl WandboxError {
    fn new(msg: &str) -> WandboxError {
        WandboxError{details: msg.to_string()}
    }
}

impl fmt::Display for WandboxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl std::error::Error for WandboxError {
    fn description(&self) -> &str {
        &self.details
    }
}