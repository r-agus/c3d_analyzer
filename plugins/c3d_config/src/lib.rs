use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use toml;

#[derive(Deserialize, Debug)]
pub struct Config {
    visible_points: Option<Vec<String>>,
    joins: Option<Vec<Vec<String>>>,
    point_color: Option<String>,
    join_color: Option<String>,
    line_thickness: Option<f32>,
    point_size: Option<f32>,
}

#[derive(Deserialize, Debug)]
pub struct PointGroupConfig {
    point_color: Option<String>,
    join_color: Option<String>,
    line_thickness: Option<f32>,
    point_size: Option<f32>,
}

#[derive(Deserialize, Debug)]
pub struct ConfigFile {
    config_name: HashMap<String, Config>,
    point_groups: Option<HashMap<String, Vec<String>>>,
    point_groups_config: Option<HashMap<String, PointGroupConfig>>,
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