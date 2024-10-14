use config_plugin::*;

fn main() {
    let config_file = parse_config("./assets/example.toml").unwrap();
    println!("{:?}", config_file.get_config_map());
}