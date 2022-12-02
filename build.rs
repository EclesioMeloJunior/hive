use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let result = tonic_build::compile_protos("./protos/voting.proto");
    if let Err(err) = result {
        println!("{:?}", err);
        return Ok(());
    }

    Ok(())
}
