use anyhow::Result;
use binlang::generate_c;

fn main() -> Result<()> {
    let source = std::fs::read_to_string("examples/greycat_abi.bl")?;
    let output = generate_c(&source)?;
    println!("{output}");
    Ok(())
}
