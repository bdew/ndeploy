#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CfgObj {
    #[serde(default = "default_flake_path")]
    pub flake_path: String,
    pub hosts: HashMap<String, Host>,
}

fn default_flake_path() -> String {
    ".".into()
}

// Hack for nicer config format with untagged mode
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum HostTypeLocal {
    Local,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Host {
    Local {
        #[serde(flatten)]
        _type: HostTypeLocal,
    },
    Remote {
        user: String,
        addr: String,
        sudo: Option<bool>,
    },
}


impl CfgObj {
    pub fn load(file: impl AsRef<Path>) -> Result<Self> {
        let str = fs::read_to_string(file)?;
        let res: CfgObj = serde_yaml_ng::from_str(&str)?;
        Ok(res)
    }
}
