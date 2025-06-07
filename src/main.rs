use anyhow::Result;
use binlang::generate_c;

fn main() -> Result<()> {
    env_logger::init();

    let filepath = std::env::args().nth(1).expect("binlang <filepath>");
    let source = std::fs::read_to_string(filepath)?;
    let output = generate_c(&source)?;
    println!("{output}");
    Ok(())
}
