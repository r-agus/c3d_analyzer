use bevy:: prelude::*;
use bevy::reflect::TypePath;

use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use toml::{Value, map::Map};

#[derive(Deserialize, Debug)]
pub struct Config {
    visible_points: Option<Vec<String>>, // Contains a regex for each point that should be visible
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
    pub fn get_visible_points(&self) -> Option<&Vec<String>> {
        self.visible_points.as_ref()
    }
    pub fn get_joins(&self) -> Option<&Vec<Vec<String>>> {
        self.joins.as_ref()
    }
    pub fn add_visible_point(&mut self, point: String) {
        if let Some(visible_points) = &mut self.visible_points {
            visible_points.push(point);
        } else {
            self.visible_points = Some(vec![point]);
        }
    }
    pub fn add_visible_point_group(&mut self, group: Vec<String>) {
        if let Some(visible_points) = &mut self.visible_points {
            visible_points.extend(group);
        } else {
            self.visible_points = Some(group);
        }
    }
    #[deprecated(
        since = "1.0.0",
        note = "contains_point is deprecated, use contains_point_regex instead"
    )]
    pub fn contains_point(&self, label: &str) -> bool {
        match &self.visible_points {
            Some(points) => points.contains(&label.to_string()),
            None => false,
        }
    }

    pub fn contains_point_regex(&self, label: &str) -> bool {
        match &self.visible_points {
            Some(points) => {
                for point in points {
                    let re = 
                        if point.starts_with("_"){ point.strip_prefix("_").unwrap() }
                        else {&("^".to_owned() + point + "$")};
                    if regex::Regex::new(re).unwrap().is_match(label) {
                        return true;
                    }
                }
                false
            }
            None => false,
        }
    }

    pub fn get_all_points_that_match(&self, label: &str) -> Vec<String> {
        let mut matching_points = Vec::new();
        match &self.visible_points {
            Some(points) => {
                for point in points {
                    let re = 
                        if point.starts_with("_"){ point.strip_prefix("_").unwrap() }
                        else {&("^".to_owned() + point + "$")};
                    if regex::Regex::new(re).unwrap().is_match(label) {
                        matching_points.push(point.clone());
                    }
                }
            }
            None => {}
        }
        matching_points
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

#[derive(Asset, TypePath, Deserialize, Debug)]
#[type_path = "conf_plugin::c3d_config::ConfigFile"]
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

    pub fn get_config_map(&self) -> &HashMap<String, Config> {
        &self.config_name
    }

    pub fn get_config(&self, config_name: &str) -> Option<&Config> {
        self.config_name.get(config_name)
    }

    pub fn get_all_configs(&self) -> Vec<&Config> {
        self.config_name.values().collect()
    }

    pub fn get_all_config_names(&self) -> Vec<String> {
        self.config_name.keys().cloned().collect()
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

    #[deprecated(
        since = "1.0.0",
        note = "contains_point is deprecated, use contains_point_regex instead"
    )]
    pub fn contains_point(&self, config: &str, label: &str) -> bool {
        match self.config_name.get(config) {
            Some(config) => config.contains_point(label),
            None => false,
        }
    }

    pub fn contains_point_regex(&self, config: &str, label: &str) -> bool {
        match self.config_name.get(config) {
            Some(config) => config.contains_point_regex(label),            
            None => false,
        }
    }

    pub fn get_all_points_that_match(&self, config: &str, label: &str) -> Vec<String> {
        let matching_points = match self.config_name.get(config) {
            Some(config) => {
                config.get_all_points_that_match(label)
            }
            None => { Vec::new() }
        };
        matching_points
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


fn read_config(file_or_string: &str, from_file: bool) -> Result<HashMap<String, Value>, Box<dyn std::error::Error>> {
    let content = if from_file {fs::read_to_string(file_or_string)?} else {file_or_string.to_string()};
    let config: HashMap<String, Value> = toml::from_str(&content)?;
    Ok(config)
}

pub fn parse_config(file_or_string: &str, from_file: bool) -> Result<ConfigFile, String> {
    let config = read_config(file_or_string, from_file).unwrap();
    let mut config_file = ConfigFile::default();

    // Caution: The order of the keys in the config file is not guaranteed, because it is a hashmap.
    // We need to parse the point groups first, as they are used in the individual configs. 
    for (key, value) in config.iter() {
        if key == "point_groups" {
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
                        config_file.add_point_group(group_name.clone(), points_vec);
                    }
                }
            }
        }
    }

    for (key, value) in config {
        match key.as_str() {
            "point_groups" => {} // Already parsed
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
                        let config = parse_individual_config(sub_table, &config_file.point_groups)?;
                        config_file.config_name.insert(key, config);
                    }
                }
            }
        }
    }
    // TODO: merge the point group configs with the individual configs

    Ok(config_file)
}

fn parse_individual_config(
    table: Map<String, Value>,
    point_groups: &Option<HashMap<String, Vec<String>>>,
) -> Result<Config, String> {
    let mut config = Config::default();

    if let Some(Value::Array(visible_points)) = table.get("visible_points") {
        for item in visible_points.iter() {
            match item {
                // normal case: Add a single point
                Value::String(point) => config.add_visible_point(point.clone()),
                // case where we want to add a group of points
                Value::Array(group_ref) if (group_ref.len() == 1 && point_groups.is_some()) => {
                    if let Some(Value::String(group_name)) = group_ref.get(0) {
                        if let Some(points) = point_groups.as_ref().unwrap().get(group_name) {
                            config.add_visible_point_group(points.clone());
                            // println!("Detected group {:?} with points {:?}", group_name, points);
                        }
                    }
                },
                Value::Array(group_ref) => println!("Detected unexpected group reference: {:?}", group_ref),
                _ => println!("Invalid point in visible_points: {:?}", item),
                
            }
        }
    }

    if let Some(Value::Array(joins)) = table.get("joins") {
        for join in joins {
            if let Value::Array(points) = join {
                let mut expanded_points = Vec::new();
                for point in points {
                    match point {
                        Value::String(point_name) => expanded_points.push(point_name.clone()),
                        Value::Array(group_ref) if (group_ref.len() == 1 && point_groups.is_some()) => {
                            if let Some(Value::String(group_name)) = group_ref.get(0) {
                                if let Some(points) = point_groups.as_ref().unwrap().get(group_name) {
                                    expanded_points.extend(points.clone());
                                }
                            }
                        },
                        Value::Array(group_ref) => println!("Detected unexpected group reference: {:?}. Length of group reference: {}. point_groups: {:?}", group_ref, group_ref.len(), point_groups),
                        _ => {
                            println!("Invalid point in joins: {:?}", point);
                            continue
                        },
                    }
                }
                if expanded_points.len() > 1 {
                    config.joins.get_or_insert(Vec::new()).push(expanded_points);
                }
            }
        }
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