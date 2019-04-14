pub struct Config {
    pub filepath: String,
    pub start_point: String,
    pub end_point: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 4 {
            Err("not enough arguments")
        } else {
            let filepath = args[1].clone();
            let start_point = args[2].clone();
            let end_point = args[3].clone();

            Ok(Config { filepath, start_point, end_point })
        }
    }
}
