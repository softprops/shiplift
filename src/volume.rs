//! Create and manage persistent storage that can be attached to containers.
//!
//! API Reference: <https://docs.docker.com/engine/api/v1.41/#tag/Volume>

use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

use hyper::Body;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    errors::{Error, Result},
    Docker,
};

#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};

/// Interface for docker volumes
///
/// API Reference: <https://docs.docker.com/engine/api/v1.41/#tag/Volume>
pub struct Volumes<'docker> {
    docker: &'docker Docker,
}

impl<'docker> Volumes<'docker> {
    /// Exports an interface for interacting with docker volumes
    pub fn new(docker: &'docker Docker) -> Self {
        Volumes { docker }
    }

    /// Creates a new docker volume.
    ///
    /// API Reference: <https://docs.docker.com/engine/api/v1.41/#operation/VolumeCreate>
    pub async fn create(
        &self,
        opts: &VolumeCreateOptions,
    ) -> Result<VolumeCreateInfo> {
        let body: Body = opts.serialize()?.into();
        let path = vec!["/volumes/create".to_owned()];

        self.docker
            .post_json(&path.join("?"), Some((body, mime::APPLICATION_JSON)))
            .await
    }

    /// Lists the docker volumes on the current docker host
    ///
    /// API Reference: <https://docs.docker.com/engine/api/v1.41/#operation/VolumeList>
    pub async fn list(&self) -> Result<Vec<VolumeInfo>> {
        let path = vec!["/volumes".to_owned()];

        let volumes_rep = self.docker.get_json::<VolumesInfo>(&path.join("?")).await?;
        Ok(match volumes_rep.volumes {
            Some(volumes) => volumes,
            None => vec![],
        })
    }

    /// Returns a reference to a set of operations available for a named volume
    pub fn get(
        &self,
        name: &str,
    ) -> Volume<'docker> {
        Volume::new(self.docker, name)
    }
}

/// Interface for accessing and manipulating a named docker volume
///
/// API Reference: <https://docs.docker.com/engine/api/v1.41/#tag/Volume>
pub struct Volume<'docker> {
    docker: &'docker Docker,
    name: String,
}

impl<'docker> Volume<'docker> {
    /// Exports an interface for operations that may be performed against a named volume
    pub fn new<S>(
        docker: &'docker Docker,
        name: S,
    ) -> Self
    where
        S: Into<String>,
    {
        Volume {
            docker,
            name: name.into(),
        }
    }

    /// Deletes a volume
    ///
    /// API Reference: <https://docs.docker.com/engine/api/v1.41/#operation/VolumeDelete>
    pub async fn delete(&self) -> Result<()> {
        self.docker
            .delete(&format!("/volumes/{}", self.name)[..])
            .await?;
        Ok(())
    }
}

/// Interface for creating volumes
#[derive(Serialize, Debug)]
pub struct VolumeCreateOptions {
    params: HashMap<&'static str, Value>,
}

impl VolumeCreateOptions {
    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Result<String> {
        serde_json::to_string(&self.params).map_err(Error::from)
    }

    pub fn parse_from<'a, K, V>(
        &self,
        params: &'a HashMap<K, V>,
        body: &mut BTreeMap<String, Value>,
    ) where
        &'a HashMap<K, V>: IntoIterator,
        K: ToString + Eq + Hash,
        V: Serialize,
    {
        for (k, v) in params.iter() {
            let key = k.to_string();
            let value = serde_json::to_value(v).unwrap();

            body.insert(key, value);
        }
    }

    /// return a new instance of a builder for options
    pub fn builder() -> VolumeCreateOptionsBuilder {
        VolumeCreateOptionsBuilder::new()
    }
}

#[derive(Default)]
pub struct VolumeCreateOptionsBuilder {
    params: HashMap<&'static str, Value>,
}

impl VolumeCreateOptionsBuilder {
    pub(crate) fn new() -> Self {
        let params = HashMap::new();
        VolumeCreateOptionsBuilder { params }
    }

    pub fn driver(
        &mut self,
        driver_name: &str,
        driver_opts: Option<&HashMap<&str, &str>>,
    ) -> &mut Self {
        self.params.insert("Driver", json!(driver_name));
        if let Some(opts) = driver_opts {
            self.params.insert("DriverOpts", json!(opts));
        }
        self
    }

    pub fn name(
        &mut self,
        name: &str,
    ) -> &mut Self {
        self.params.insert("Name", json!(name));
        self
    }

    pub fn labels(
        &mut self,
        labels: &HashMap<&str, &str>,
    ) -> &mut Self {
        self.params.insert("Labels", json!(labels));
        self
    }

    pub fn build(&self) -> VolumeCreateOptions {
        VolumeCreateOptions {
            params: self.params.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VolumeCreateInfo {
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VolumesInfo {
    pub volumes: Option<Vec<VolumeInfo>>,
    pub warnings: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VolumeInfo {
    #[cfg(feature = "chrono")]
    pub created_at: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    pub created_at: String,
    pub driver: String,
    pub labels: Option<HashMap<String, String>>,
    pub name: String,
    pub mountpoint: String,
    pub options: Option<HashMap<String, String>>,
    pub scope: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct VolumeTestInfo {
        pub driver: Option<String>,
        pub name: Option<String>,
        pub driver_opts: Option<HashMap<String, String>>,
    }

    #[test]
    fn test_volumecreateoptionsbuilder_driver() {
        let volume = VolumeCreateOptions::builder()
            .driver("my_driver", None)
            .build();

        let serialized = volume.serialize().unwrap();
        let volume_info: VolumeTestInfo = serde_json::from_str(&serialized).unwrap();

        assert_eq!(volume_info.driver, Some("my_driver".to_string()));
        assert_eq!(volume_info.name, None);
        assert_eq!(volume_info.driver_opts, None);
    }

    #[test]
    fn test_volumecreateoptionsbuilder_driver_opts() {
        let opts: HashMap<&str, &str> = [("option", "value")].iter().copied().collect();
        let volume = VolumeCreateOptions::builder()
            .driver("my_driver", Some(&opts))
            .build();

        let serialized = volume.serialize().unwrap();
        let volume_info: VolumeTestInfo = serde_json::from_str(&serialized).unwrap();
        let mut driver_options = HashMap::new();
        driver_options.insert("option".to_string(), "value".to_string());

        assert_eq!(volume_info.driver, Some("my_driver".to_string()));
        assert_eq!(volume_info.name, None);
        assert_eq!(volume_info.driver_opts, Some(driver_options));
    }
}
