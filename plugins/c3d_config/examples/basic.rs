use config_plugin::*;

fn main() {
    let config_map = read_config("./assets/example.toml").unwrap();
    let config_file = parse_config(config_map).unwrap();
    println!("{:?}", config_file);
}