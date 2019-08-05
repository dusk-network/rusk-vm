use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use structopt::StructOpt;
use wabt::wasm2wat;

#[derive(Debug, StructOpt)]
#[structopt(name = "wasm2wat", about = "Convert wasm to wat")]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn main() {
    let opt = Opt::from_args();

    let mut file = File::open(&opt.input).unwrap();

    let mut buffer = vec![];

    file.read_to_end(&mut buffer).unwrap();

    let wat = wasm2wat(&buffer).unwrap();
    for line in wat.lines() {
        println!("{}", line);
    }
}
