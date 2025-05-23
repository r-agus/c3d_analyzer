use bevy:: prelude::*;
use bevy::reflect::TypePath;

use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use toml::{Value, map::Map};

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    visible_points: Option<Vec<String>>, // Contains a regex for each point that should be visible
    joins: Option<Vec<(Vec<String>, JoinShape)>>, // Contains a list of joins between points and the shape of the join
    vectors: Option<HashMap<String, Vec<(String, f64)>>>, // Map where the key is the point and the value are the vectors fixed to that point, with their name and the scale
    point_color: Option<Vec<u8>>,
    join_color: Option<Vec<u8>>,
    line_thickness: Option<f64>,
    point_size: Option<f64>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum JoinShape {
    Line,
    Cylinder(f64),      // Radius
    SemiCone(f64, f64), // Radius of one end, radius of the other end
    RectangularPrism(f64, f64, Option<[String;3]>), // Width, height, vectores de orientación unitarios (Iv, Jv, Kv)
}

impl Config {
    pub fn default() -> Self {
        Config {
            visible_points: None,
            joins: None,
            vectors: None,
            point_color: None,
            join_color: None,
            line_thickness: None,
            point_size: None,
        }
    }
    pub fn get_visible_points(&self) -> Option<&Vec<String>> {
        self.visible_points.as_ref()
    }
    pub fn get_joins(&self) -> Option<&Vec<(Vec<String>, JoinShape)>> {
        self.joins.as_ref()
    }
    pub fn get_vectors(&self) -> Option<&HashMap<String, Vec<(String, f64)>>> {
        self.vectors.as_ref()
    }
    pub fn get_vectors_for_point(&self, point: &str) -> Option<&Vec<(String, f64)> > {
        self.vectors.as_ref().and_then(|v| v.get(point))
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
    point_color: Option<Vec<u8>>,
    join_color: Option<Vec<u8>>,
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
/// This contains the configuration of the C3D file
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

    pub fn get_config_name(&self, config: &Config) -> String {
        self.config_name.iter().find_map(|(name, c)| if c == config { Some(name.clone()) } else { None }).unwrap_or("Others".to_string())
    }

    pub fn get_all_configs(&self) -> Vec<&Config> {
        self.config_name.values().collect()
    }

    pub fn get_all_config_names(&self) -> Vec<String> {
        let mut configs: Vec<String> = self.config_name.keys().cloned().collect();
        configs.sort();
        configs
    }

    pub fn get_point_group(&self, point_group_name: &str) -> Option<&Vec<String>> {
        match &self.point_groups {
            Some(point_groups) => point_groups.get(point_group_name),
            None => None,
        }
    }

    /// Searches for the color of a point in the config file. Returns the point_color config if exists, if not, the default config color, and if it is not set, None.
    /// If a point is in more than one point group, the first one found will be used. The order of the point groups is not guaranteed.
    pub fn get_point_color(&self, label: &str, config: &str) -> Option<Vec<u8>> {
        // First check if the point is in a point group
        if let Some(point_groups) = &self.point_groups {
            for (group_name, points) in point_groups.iter() {
                if points.contains(&label.to_string()) {
                    if let Some(point_group_config) = self.point_groups_config.as_ref().and_then(|c| c.get(group_name)) {
                        return point_group_config.point_color.clone();
                    }
                }
            }
        }
        // If not, check the individual config
        self.config_name.get(config).and_then(|c| c.point_color.clone()).or_else(|| None)        
    }

    /// Searches for the size of a point in the config file. Returns the point_color config if exists, if not, the default config color, and if it is not set, None.
    /// If a point is in more than one point group, the first one found will be used. The order of the point groups is not guaranteed.
    pub fn get_point_size (&self, label: &str, config: &str) -> Option<f64> {
        // First check if the point is in a point group
        if let Some(point_groups) = &self.point_groups {
            for (group_name, points) in point_groups.iter() {
                if points.contains(&label.to_string()) {
                    if let Some(point_group_config) = self.point_groups_config.as_ref().and_then(|c| c.get(group_name)) {
                        return point_group_config.point_size;
                    }
                }
            }
        }
        // If not, check the individual config
        self.config_name.get(config).and_then(|c| c.point_size).or_else(|| None)
    }

    /// Searches for the thickness of a join between two points in the config file. Returns the line_thickness config if exists, if not, the default config color, and if it is not set, None.
    /// The order between the two points does not matter.
    pub fn get_line_thickness(&self, point1: &str, point2: &str, config: &str) -> Option<f64> {
        // First check if the points are in a point group
        if let Some(point_groups) = &self.point_groups {
            for (group_name, points) in point_groups.iter() {
                if points.contains(&point1.to_string()) && points.contains(&point2.to_string()) {
                    if let Some(point_group_config) = self.point_groups_config.as_ref().and_then(|c| c.get(group_name)) {
                        return point_group_config.line_thickness;
                    }
                }
            }
        }
        // If not, check the individual config
        self.config_name.get(config).and_then(|c| c.line_thickness).or_else(|| None)
    }

    /// Searches for the color of a join between two points in the config file. Returns the join_color config if exists, if not, the default config color, and if it is not set, None.
    /// The order between the two points does not matter.
    pub fn get_join_color(&self, point1: &str, point2: &str, config: &str) -> Option<Vec<u8>> {
        // First check if the points are in a point group
        if let Some(point_groups) = &self.point_groups {
            for (group_name, points) in point_groups.iter() {
                if points.contains(&point1.to_string()) && points.contains(&point2.to_string()) {
                    if let Some(point_group_config) = self.point_groups_config.as_ref().and_then(|c| c.get(group_name)) {
                        return point_group_config.join_color.clone();
                    }
                }
            }
        }
        // If not, check the individual config
        self.config_name.get(config).and_then(|c| c.join_color.clone()).or_else(|| None)
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
            Some(config) => config.contains_point_regex(label), //config.contains_point(label),
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

    pub fn get_all_configs_that_contain_point(&self, label: &str) -> Vec<&Config> {
        let mut matching_configs = Vec::new();
        for config in self.config_name.values() {
            if config.contains_point_regex(label) {
                matching_configs.push(config);
            }
        }
        matching_configs
    }

    pub fn get_all_config_names_that_contain_point(&self, label: &str) -> Vec<String> {
        let mut matching_configs = Vec::new();
        for (config_name, config) in self.config_name.iter() {
            if config.contains_point_regex(label) {
                matching_configs.push(config_name.clone());
            }
        }
        matching_configs
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
        vectors: base.vectors.clone(),
    }
}

fn read_config(file_or_string: &str, from_file: bool) -> Result<HashMap<String, Value>, Box<dyn std::error::Error>> {
    let content = if from_file {fs::read_to_string(file_or_string)?} else {file_or_string.to_string()};
    let config: HashMap<String, Value> = toml::from_str(&content)?;
    Ok(config)
}

pub fn parse_config(file_or_string: &str, from_file: bool) -> Result<ConfigFile, String> {
    let config = read_config(file_or_string, from_file).unwrap_or(HashMap::new());
    let mut config_file = ConfigFile::default();

    // Caution: The order of the keys in the config file is not guaranteed, because it is a hashmap.
    // We need to parse the point groups first, as they are used in the individual configs. 
    config.get("point_groups").and_then(|v| v.as_table()).map(|groups| {
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
    });

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

    // Now merge the point group configs with the individual configs
    for (group_name, group_config) in config_file.point_groups_config.iter().flatten() {
        if let Some(point_groups) = &config_file.point_groups {
            if let Some(points) = point_groups.get(group_name) {
                for point in points {
                    if let Some(config) = config_file.config_name.get_mut(point) {
                        *config = merge_configs(config, group_config);
                    }
                }
            }
        }
    }


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
                        }
                    }
                },
                Value::Array(group_ref) => println!("Detected unexpected group reference: {:?}", group_ref),
                _ => println!("Invalid point in visible_points: {:?}", item),
                
            }
        }
    }

    if let Some(Value::Array(vectors)) = table.get("vectors") {
        let mut vector_map: HashMap<String, Vec<(String, f64)>> = HashMap::new();
        for vector in vectors {
            let mut vectors_in_map = Vec::new();
            if let Value::Array(vector_pair) = vector {
                if vector_pair.len() == 2 {
                    if let Some(Value::String(point)) = vector_pair.get(0) {
                        match vector_pair.get(1) {
                            Some(Value::String(vector_name)) => {
                                if vector_map.contains_key(point) {
                                    vectors_in_map = vector_map.get(point).unwrap().clone();  // If the point already has vectors, we need to keep them
                                }
                                vectors_in_map.push((vector_name.clone(), 1.0));
                                vector_map.insert(point.clone(), vectors_in_map);
                            },
                            Some(Value::Float(_)) | Some(Value::Integer(_)) => println!("Scale not permitted without point and vector: {:?}", vector_pair),
                            Some(Value::Array(values)) => {
                                vectors_in_map = vector_map.get(point).unwrap_or(&Vec::new()).to_vec();
                                for vector in values {
                                    match vector {
                                        Value::String(vector) => vectors_in_map.push((vector.clone(), 1.0)),
                                        _ => println!("Value not recognized at {vector}")
                                    }
                                }
                                vector_map.insert(point.to_string(), vectors_in_map);
                            },
                            _ => (),
                        }
                    } else {
                        println!("Invalid vector pair: {:?}", vector_pair)
                    }
                } else if vector_pair.len() == 3 {
                    if let Some(Value::String(point)) = vector_pair.get(0) {
                        match vector_pair.get(1) {
                            Some(Value::String(vector_name)) => {
                                if let Some(Value::Float(scale)) = vector_pair.get(2) {
                                    if vector_map.contains_key(point) {
                                        vectors_in_map = vector_map.get(point).unwrap().clone();  // If the point already has vectors, we need to keep them
                                    }
                                    vectors_in_map.push((vector_name.clone(), *scale));
                                    vector_map.insert(point.clone(), vectors_in_map);
                                }
                            },
                            Some(Value::Float(_)) | Some(Value::Integer(_)) => println!("Scale not permitted without point and vector. Maybe incorrect order. Set Point, Vector, Scale: {:?}", vector_pair),
                            Some(Value::Array(values)) => {
                                vectors_in_map = vector_map.get(point).unwrap_or(&Vec::new()).to_vec();
                                let scale = vector_pair.get(2).and_then(|x| x.as_float()).unwrap_or(1.0);
                                for vector in values {
                                    match vector {
                                        Value::String(vector) => vectors_in_map.push((vector.clone(), scale)),
                                        _ => println!("Value not recognized at {vector}")
                                    }
                                }
                                vector_map.insert(point.to_string(), vectors_in_map);  
                            },
                            _ => (),
                        }
                    }
                } else {
                    println!("Invalid vector pair: {:?}", vector_pair);
                }
            } else {
                println!("Invalid vector: {:?}", vector);
            }
        }
        config.vectors = Some(vector_map);
    }

    if let Some(Value::Array(joins)) = table.get("joins") {
        for join in joins {
            match join {
                Value::Array(points) => {
                    generate_expanded_points(point_groups, &mut config, points, JoinShape::Line);
                }
                Value::Table(join_table) => {
                    let shape = join_table.get("shape");
                    if let Some(shape) = shape {
                        match shape {
                            Value::String(s) if s.to_lowercase() == "line" => {
                                if let Some(Value::Array(points)) = join_table.get("points") {
                                    generate_expanded_points(point_groups, &mut config, points, JoinShape::Line);
                                }
                            }
                            Value::Table(shapes_table) if shapes_table.contains_key("type") => {
                                match shapes_table.get("type") {
                                    Some(Value::String(s)) if s.to_lowercase() == "line" => {
                                        if let Some(Value::Array(points)) = join_table.get("points") {
                                            generate_expanded_points(point_groups, &mut config, points, JoinShape::Line);
                                        }
                                    }
                                    Some(Value::String(s)) 
                                        if (s.to_lowercase() == "cylinder" || 
                                            s.to_lowercase() == "cilindro"
                                    ) => {
                                        if let Some(Value::Array(points)) = join_table.get("points") {
                                            match shapes_table.get("radius") {
                                                Some(Value::Float(radius)) => generate_expanded_points(point_groups, &mut config, points, JoinShape::Cylinder(*radius)),
                                                Some(Value::Integer(radius)) => generate_expanded_points(point_groups, &mut config, points, JoinShape::Cylinder(*radius as f64)),                                      
                                                _ => {
                                                    println!("Cylinder join without radius: {:?}", shapes_table);
                                                    generate_expanded_points(point_groups, &mut config, points, JoinShape::Line);
                                                }
                                            }
                                        } else {
                                            println!("Cylinder join without points: {:?}", join_table);
                                        }
                                    },
                                    Some(Value::String(s)) 
                                        if (s.to_lowercase() == "cone" ||
                                            s.to_lowercase() == "cono"
                                    ) => {
                                        if let Some(Value::Array(points)) = join_table.get("points") {
                                            match shapes_table.get("radius") {
                                                Some(Value::Float(radius)) => generate_expanded_points(point_groups, &mut config, points, JoinShape::SemiCone(*radius, 0.0)),
                                                Some(Value::Integer(radius)) => generate_expanded_points(point_groups, &mut config, points, JoinShape::SemiCone(*radius as f64, 0.0)),
                                                _ => {
                                                    println!("Cone join without proper radius: {:?}", shapes_table);
                                                    generate_expanded_points(point_groups, &mut config, points, JoinShape::SemiCone(0.5, 0.0));
                                                },
                                            }
                                        }
                                    },
                                    Some(Value::String(s)) 
                                        if (s.to_lowercase() == "semicone" ||
                                            s.to_lowercase() == "semicono" ||
                                            s.to_lowercase() == "cone frustum" ||
                                            s.to_lowercase() == "cono truncado" ||
                                            s.to_lowercase() == "partial cone" ||
                                            s.to_lowercase() == "cono parcial" ||
                                            s.to_lowercase() == "truncated cone" ||
                                            s.to_lowercase() == "cono truncado"
                                    ) => {
                                        if let Some(Value::Array(points)) = join_table.get("points") {
                                            match (shapes_table.get("radius1"), shapes_table.get("radius2")) {
                                                (Some(Value::Float(radius1)), Some(Value::Float(radius2))) => 
                                                    generate_expanded_points(point_groups, &mut config, points, JoinShape::SemiCone(*radius1, *radius2)),
                                                (Some(Value::Integer(radius1)), Some(Value::Integer(radius2))) => 
                                                    generate_expanded_points(point_groups, &mut config, points, JoinShape::SemiCone(*radius1 as f64, *radius2 as f64)),
                                                (Some(Value::Float(radius1)), Some(Value::Integer(radius2))) =>
                                                    generate_expanded_points(point_groups, &mut config, points, JoinShape::SemiCone(*radius1, *radius2 as f64)),
                                                (Some(Value::Integer(radius1)), Some(Value::Float(radius2))) =>
                                                    generate_expanded_points(point_groups, &mut config, points, JoinShape::SemiCone(*radius1 as f64, *radius2)),
                                                _ => {
                                                    println!("SemiCone join without proper radius: {:?}", shapes_table);
                                                    generate_expanded_points(point_groups, &mut config, points, JoinShape::Line);
                                                },
                                            }
                                        } else {
                                            println!("SemiCone join without points: {:?}", join_table);
                                        }
                                    },
                                    Some(Value::String(s))
                                        if (s.to_lowercase() == "prisma rectangular" ||
                                            s.to_lowercase() == "rectangular prism" ||
                                            s.to_lowercase() == "prisma" ||
                                            s.to_lowercase() == "prism" ||
                                            s.to_lowercase() == "paralelepipedo" ||
                                            s.to_lowercase() == "paralelepípedo" ||
                                            s.to_lowercase() == "parallelepiped"
                                    ) => {
                                        if let Some(Value::Array(points)) = join_table.get("points") {
                                            let width = shapes_table.get("width").and_then(Value::as_float).or_else(|| shapes_table.get("width").and_then(Value::as_integer).map(|v| v as f64));
                                            let height = shapes_table.get("height").and_then(Value::as_float).or_else(|| shapes_table.get("height").and_then(Value::as_integer).map(|v| v as f64));
                                            
                                            let orientation_point = shapes_table.get("vector").and_then(|v| v.as_str());
                                            let orientation_vectors = orientation_point
                                                .and_then(|orientation_vectors| {
                                                    config.get_vectors_for_point(orientation_vectors)
                                                        .filter(|vectors| vectors.len() == 3)
                                                        .map(|vectors| [
                                                            vectors[0].0.clone(),
                                                            vectors[1].0.clone(),
                                                            vectors[2].0.clone(),
                                                        ])
                                                });
                                            
                                            if width.is_none() || height.is_none() {
                                                println!("Rectangular prism join without proper width or height: {:?}", shapes_table);
                                                generate_expanded_points(point_groups, &mut config, points, JoinShape::Line);
                                                continue;
                                            }
                                            
                                            let width = width.unwrap();
                                            let height = height.unwrap();

                                            match orientation_point {
                                                Some(orientation_point) => {
                                                    match orientation_vectors {
                                                        Some(vectors) => generate_expanded_points(point_groups, &mut config, points, JoinShape::RectangularPrism(width, height, Some(vectors))),
                                                        None => {
                                                            println!("Orientation point {:?} not found in vectors", orientation_point);
                                                            generate_expanded_points(point_groups, &mut config, points, JoinShape::RectangularPrism(width, height, None));
                                                        }
                                                    }
                                                },
                                                None => generate_expanded_points(point_groups, &mut config, points, JoinShape::RectangularPrism(width, height, None)),
                                            }
                                        }
                                    },
                                    _ => {
                                        if let Some(Value::Array(points)) = join_table.get("points") {
                                            generate_expanded_points(point_groups, &mut config, points, JoinShape::SemiCone(0.75, 0.25));
                                        }
                                        println!("Shape {:?} not implemented", shape)
                                    },
                                    
                                }
                            }
                            _ => {
                                if let Some(Value::Array(points)) = join_table.get("points") {
                                    generate_expanded_points(point_groups, &mut config, points, JoinShape::Line);
                                }
                                println!("Shape {:?} not implemented", shape)
                            },
                        }
                    }
                },
                _ => (),
            }
        }
    }

    config.point_color = table.get("point_color").and_then(|v| v.as_array()).and_then(|v| {
        if v.len() == 3 {
            Some(vec![v[0].as_integer().unwrap() as u8, v[1].as_integer().unwrap() as u8, v[2].as_integer().unwrap() as u8])
        } else if v.len() == 4 {
            Some(vec![v[0].as_integer().unwrap() as u8, v[1].as_integer().unwrap() as u8, v[2].as_integer().unwrap() as u8, v[3].as_integer().unwrap() as u8])
        } else {
            None
        }
    });
    config.join_color = table.get("join_color").and_then(|v| v.as_array()).and_then(|v| {
        if v.len() == 3 {
            Some(vec![v[0].as_integer().unwrap() as u8, v[1].as_integer().unwrap() as u8, v[2].as_integer().unwrap() as u8])
        } else if v.len() == 4 {
            Some(vec![v[0].as_integer().unwrap() as u8, v[1].as_integer().unwrap() as u8, v[2].as_integer().unwrap() as u8, v[3].as_integer().unwrap() as u8])
        } else {
            None
        }
    });
    config.line_thickness = table.get("line_thickness").and_then(|v| v.as_float());
    config.point_size = table.get("point_size").and_then(|v| v.as_float());

    Ok(config)
}

fn generate_expanded_points(point_groups: &Option<HashMap<String, Vec<String>>>, config: &mut Config, points: &Vec<Value>, join_shape: JoinShape) {
    let mut expanded_points = Vec::new();
    for point in points {
        match point {
            Value::String(point_name) => expanded_points.push(point_name.clone()),
            Value::Array(group_ref) if (group_ref.len() == 1 && point_groups.is_some()) => {
                expand_point_group(point_groups, &mut expanded_points, group_ref);
            },
            Value::Array(group_ref) => println!("Detected unexpected group reference: {:?}. Length of group reference: {}. point_groups: {:?}", group_ref, group_ref.len(), point_groups),
            _ => {
                println!("Invalid point in joins: {:?}", point);
                continue
            },                        
        }
    }
    if expanded_points.len() > 1 {
        if config.joins.is_none() {
            config.joins = Some(vec![(expanded_points, join_shape)]);
        } else {
            config.joins.as_mut().unwrap().push((expanded_points, join_shape));
        }
    }
}

fn expand_point_group(point_groups: &Option<HashMap<String, Vec<String>>>, expanded_points: &mut Vec<String>, group_ref: &Vec<Value>) {
    if let Some(Value::String(group_name)) = group_ref.get(0) {
        if let Some(points) = point_groups.as_ref().unwrap().get(group_name) {
            expanded_points.extend(points.clone());
        }
    }
}


fn parse_point_group_config(table: Map<String, Value>) -> Result<PointGroupConfig, String> {
    let mut group_config = PointGroupConfig::default();

    group_config.point_color = table.get("point_color").and_then(|v| v.as_array()).and_then(|v| {
        if v.len() == 3 {
            Some(vec![v[0].as_integer().unwrap() as u8, v[1].as_integer().unwrap() as u8, v[2].as_integer().unwrap() as u8])
        } else if v.len() == 4 {
            Some(vec![v[0].as_integer().unwrap() as u8, v[1].as_integer().unwrap() as u8, v[2].as_integer().unwrap() as u8, v[3].as_integer().unwrap() as u8])
        } else {
            None
        }
    });
    group_config.point_size = table.get("point_size").and_then(|v| v.as_float());
    group_config.join_color = table.get("join_color").and_then(|v| v.as_array()).and_then(|v| {
        if v.len() == 3 {
            Some(vec![v[0].as_integer().unwrap_or(27) as u8, v[1].as_integer().unwrap_or(210) as u8, v[2].as_integer().unwrap_or(27) as u8])
        } else if v.len() == 4 {
            Some(vec![v[0].as_integer().unwrap_or(27) as u8, v[1].as_integer().unwrap_or(210) as u8, v[2].as_integer().unwrap_or(27) as u8, v[3].as_integer().unwrap_or(50) as u8])
        } else {
            None
        }
    });
    group_config.line_thickness = table.get("line_thickness").and_then(|v| v.as_float());
    Ok(group_config)
}