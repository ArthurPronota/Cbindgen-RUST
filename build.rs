use std::error::Error;

fn main() ->Result<(), Box<dyn Error>>{
    cbindgen::Builder::new()
        .with_crate(".")
        .with_language(cbindgen::Language::C)
        .generate()?
        .write_to_file("include/my_rust_lib.h");

    Ok(())
}
