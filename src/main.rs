use std::io::BufReader;

enum ActionType {
    Unknown = 0,
    CopyPath = 100,
    FileTransfer = 101,
    Realise = 102,
    CopyPaths = 103,
    Builds = 104,
    Build = 105,
    OptimiseStore = 106,
    VerifyPaths = 107,
    Substitute = 108,
    QueryPathInfo = 109,
    PostBuildHook = 110,
    BuildWaiting = 111,
}

fn main() {
    let path = std::env::args().nth(1).expect("missing file path");
    let file = std::fs::File::open(&path).expect("could not open file");
    let data = BufReader::new(file);
    println!("Hello, world! {path}");
}
