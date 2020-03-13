use std::fs;
use std::path::PathBuf;
use nodespeak;

#[test]
fn ct_asserts() {
    let mut test_file_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_file_directory.push("tests/ct_assert_sources");
    for file in fs::read_dir(test_file_directory).expect("Could not find test directory.") {
        let file = file.expect("Could not list test directory.");
        let code = fs::read_to_string(file.path()).expect("Could not read from file.");
        let source_set =
            nodespeak::SourceSet::from_raw_string(&format!("{:?}", file.path()), &code);
        match nodespeak::resolve(&source_set) {
            Result::Ok(_program) => (),
            Result::Err(err) => {
                eprintln!("{}", err);
                panic!("Compilation failed.")
            }
        }
    }
}
