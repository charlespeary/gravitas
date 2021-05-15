use parser::parse_file;
use std::fs;
use std::io;

#[test]
fn run_sample_programs() -> io::Result<()> {
    for sample_program in fs::read_dir("./tests/sample_programs")?.map(|p| p.unwrap().path()) {
        let file_name = sample_program.file_name().unwrap().to_str().unwrap();
        let err_msg = &format!("Code inside {} didn't succeed to compile", file_name);
        parse_file(sample_program).expect(err_msg);
    }

    Ok(())
}
