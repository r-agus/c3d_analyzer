use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use toml::{Value, map::Map};

#[derive(Deserialize, Debug)]
pub struct Config {
    visible_points: Option<Vec<String>>,
    joins: Option<Vec<Vec<String>>>,
    point_color: Option<String>,
    join_color: Option<String>,
    line_thickness: Option<f64>,
    point_size: Option<f64>,
}

impl Config {
    pub fn default() -> Self {
        Config {
            visible_points: None,
            joins: None,
            point_color: None,
            join_color: None,
            line_thickness: None,
            point_size: None,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct PointGroupConfig {
    point_color: Option<String>,
    join_color: Option<String>,
    line_thickness: Option<f64>,
    point_size: Option<f64>,
}

impl PointGroupConfig {
    pub fn default() -> Self {
        PointGroupConfig {
            point_color: None,
            join_color: None,
            line_thickness: None,
            point_size: None,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct ConfigFile {
    config_name: HashMap<String, Config>,
    point_groups: Option<HashMap<String, Vec<String>>>,
    point_groups_config: Option<HashMap<String, PointGroupConfig>>,
}

impl ConfigFile {
    pub fn default() -> Self {
        ConfigFile {
            config_name: HashMap::new(),
            point_groups: None,
            point_groups_config: None,
        }
    }

    pub fn get_config(&self, config_name: &str) -> Option<&Config> {
        self.config_name.get(config_name)
    }

    pub fn get_point_group(&self, point_group_name: &str) -> Option<&Vec<String>> {
        match &self.point_groups {
            Some(point_groups) => point_groups.get(point_group_name),
            None => None,
        }
    }

    pub fn get_point_group_config(&self, point_group_name: &str) -> Option<&PointGroupConfig> {
        match &self.point_groups_config {
            Some(point_groups_config) => point_groups_config.get(point_group_name),
            None => None,
        }
    }

    pub fn add_point_group(&mut self, point_group_name: String, points: Vec<String>) {
        if let Some(point_groups) = &mut self.point_groups {
            point_groups.insert(point_group_name, points);
        } else {
            let mut point_groups = HashMap::new();
            point_groups.insert(point_group_name, points);
            self.point_groups = Some(point_groups);
        }
    }

    pub fn add_point_group_config(&mut self, point_group_name: String, config: PointGroupConfig) {
        if let Some(point_groups_config) = &mut self.point_groups_config {
            point_groups_config.insert(point_group_name, config);
        } else {
            let mut point_groups_config = HashMap::new();
            point_groups_config.insert(point_group_name, config);
            self.point_groups_config = Some(point_groups_config);
        }
    }
}

pub fn merge_configs(base: &Config, override_config: &PointGroupConfig) -> Config {
    Config {
        point_color: override_config.point_color.clone().or_else(|| base.point_color.clone()),
        join_color: override_config.join_color.clone().or_else(|| base.join_color.clone()),
        line_thickness: override_config.line_thickness.or(base.line_thickness),
        point_size: override_config.point_size.or(base.point_size),
        visible_points: base.visible_points.clone(),
        joins: base.joins.clone(),
    }
}

fn read_config(filename: &str) -> Result<HashMap<String, Value>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(filename)?;
    let config: HashMap<String, Value> = toml::from_str(&content)?;
    Ok(config)
}

fn parse_config(config_map: HashMap<String, Value>) -> Result<ConfigFile, String> {
    let mut config_file = ConfigFile::default();

    for (key, value) in config_map {
        match key.as_str() {
            "point_groups" => {
                if let Value::Table(groups) = value {
                    for (group_name, group_value) in groups {
                        if let Value::Array(points) = group_value {
                            let points_vec: Vec<String> = points
                                .iter()
                                .filter_map(|v| match v {
                                    Value::String(s) => Some(s.to_string()),
                                    _ => None,
                                })
                                .collect();
                            config_file.add_point_group(group_name, points_vec);
                        }
                    }
                }
            }
            _ => {
                if let Value::Table(sub_table) = value {  // En el toml especificamos point_group.config, que nos crea una tabla con el nombre del point_group, con un campo config, que es el que nos interesa
                    if let Some(config) = sub_table.get("config") {
                        let point_group_config = parse_point_group_config(
                            config
                                .as_table()
                                .ok_or("Error parsing point group config")?
                                .clone(),
                        )?;
                        config_file.add_point_group_config(key, point_group_config);
                    }  else {
                        let config = parse_individual_config(sub_table)?;
                        config_file.config_name.insert(key, config);
                    }
                }
            }
        }
    }

    Ok(config_file)
}

fn parse_individual_config(table: Map<String, Value>) -> Result<Config, String> {
    let mut config = Config::default();

    if let Some(Value::Array(visible_points)) = table.get("visible_points") {
        config.visible_points = Some(visible_points
            .iter()
            .filter_map(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .collect::<Vec<String>>());
    }

    if let Some(Value::Array(joins)) = table.get("joins") {
        let mut joins_vec = Vec::new();
        for join in joins {
            if let Value::Array(points) = join {
                let points_vec: Vec<String> = points
                    .iter()
                    .filter_map(|p| match p {
                        Value::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .collect();
                joins_vec.push(points_vec);
            }
        }
        config.joins = Some(joins_vec);
    }

    config.point_color = table.get("point_color").and_then(|v| v.as_str().map(String::from));
    config.join_color = table.get("join_color").and_then(|v| v.as_str().map(String::from));
    config.line_thickness = table.get("line_thickness").and_then(|v| v.as_float());
    config.point_size = table.get("point_size").and_then(|v| v.as_float());

    Ok(config)
}


fn parse_point_group_config(table: Map<String, Value>) -> Result<PointGroupConfig, String> {
    let mut group_config = PointGroupConfig::default();

    group_config.point_color = table.get("point_color").and_then(|v| v.as_str().map(String::from));
    group_config.point_size = table.get("point_size").and_then(|v| v.as_float());
    group_config.join_color = table.get("join_color").and_then(|v| v.as_str().map(String::from));
    group_config.line_thickness = table.get("line_thickness").and_then(|v| v.as_float());
    println!("{:?}", group_config);
    Ok(group_config)
}

fn main() {
    let config_map = read_config("./assets/example.toml").unwrap();
    let config_file = parse_config(config_map).unwrap();
    println!("{:?}", config_file);
}