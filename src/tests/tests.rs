use crate::Wandbox;
use std::error::Error;
use std::collections::HashSet;

#[tokio::test]
async fn is_valid_language() -> Result<(), Box<dyn Error>> {
    let wbox : Wandbox = Wandbox::new(None, None).await?;

    let cache = wbox.cache.clone();
    let lock = cache.read().unwrap();
    for (_k, v) in &*lock {
        assert!(wbox.get_default_compiler(&v.name).is_some());
    }

    Ok(())
}

#[tokio::test]
async fn get_default_controller() -> Result<(), Box<dyn Error>> {
    let wbox : Wandbox = Wandbox::new(None, None).await?;

    let cache = wbox.cache.clone();
    let lock = cache.read().unwrap();
    for (_k, v) in &*lock {
        assert!(wbox.get_default_compiler(&v.name).is_some());
    }

    Ok(())
}

#[tokio::test]
async fn is_valid_compiler_str() -> Result<(), Box<dyn Error>> {

    let wbox : Wandbox = Wandbox::new(None, None).await?;

    assert!(wbox.is_valid_compiler_str("gcc-head"));
    Ok(())
}


#[tokio::test]
async fn ignore_broken_compiler() -> Result<(), Box<dyn Error>> {
    let mut set : HashSet<String> = HashSet::new();
    set.insert(String::from("gcc-head"));

    let wbox : Wandbox = Wandbox::new(Some(set), None).await?;

    assert!(!wbox.is_valid_compiler_str("gcc-head"));
    Ok(())
}

#[tokio::test]
async fn compilation_builder_lang() -> Result<(), Box<dyn Error>> {
    let wbox : Wandbox = Wandbox::new(None, None).await?;

    let mut builder = crate::CompilationBuilder::new();
    builder.target("c++");
    builder.options_str(vec!["-Wall", "-Werror"]);
    builder.code("#include<iostream>\nint main()\n{\nstd::cout<<\"test\";\n}");
    builder.build(&wbox)?;

    let res = builder.dispatch().await.expect("Failed to lookup");
    assert_eq!(res.program_all, "test");

    Ok(())
}


#[tokio::test]
async fn compilation_builder_compiler() -> Result<(), Box<dyn Error>> {
    let wbox : Wandbox = Wandbox::new(None, None).await?;

    let mut builder = crate::CompilationBuilder::new();
    builder.target("gcc-6.3.0");
    builder.options_str(vec!["-Wall", "-Werror"]);
    builder.code("#include<iostream>\nint main()\n{\nstd::cout<<\"test\";\n}");
    builder.build(&wbox)?;

    let res = builder.dispatch().await.expect("Failed to lookup");
    assert_eq!(res.program_all, "test");

    Ok(())
}
