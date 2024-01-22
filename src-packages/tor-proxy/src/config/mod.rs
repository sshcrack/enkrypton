#[cfg(feature = "snowflake")]
use anyhow::anyhow;
use shared::config::TorConfig;
use std::path::PathBuf;


#[cfg(feature = "snowflake")]
use crate::consts::{get_pluggable_transport, get_rel_snowflake};
use anyhow::Result;
#[cfg(feature = "snowflake")]
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};

use async_trait::async_trait;

#[async_trait]
pub trait ConfigExt {
    async fn to_text(&self) -> Result<String>;
}

#[cfg(feature = "snowflake")]
fn fix_bridges(bridges: Vec<String>) -> Vec<String> {
    if cfg!(windows) && false {
        bridges
    } else {
        bridges
        .iter()
        .map(|e| e.replace(".net.global.prod.fastly", ""))
        .collect()
    }
}

#[async_trait]
impl ConfigExt for TorConfig {
    //noinspection SpellCheckingInspection
    /// Converts the configuration to a `torrc` file format
    ///
    /// # Returns
    ///
    /// The `torrc` file as a string
    async fn to_text(&self) -> Result<String> {
        let data = PathBuf::from(self.data_dir());

        let geo_ip = data.clone().join("geoip");
        let geo_ip6 = data.clone().join("geoip6");

        #[allow(unused_mut)]
        let mut config = format!(
            "SocksPort {}
HiddenServiceDir \"{}\"
HiddenServicePort 80 {}
DataDirectory \"{}\"
GeoIPFile \"{}\"
GeoIPv6File \"{}\"",
            self.get_socks_host(),
            self.service_dir().to_string_lossy().replace("\\", "/"),
            self.get_hidden_service_host(),
            self.data_dir().to_string_lossy().replace("\\", "/"),
            geo_ip.to_string_lossy().replace("\\", "/"),
            geo_ip6.to_string_lossy().replace("\\", "/"),
        );

        #[cfg(feature = "snowflake")]
        {
            let pt_config = get_pluggable_transport().join("pt_config.json");
            let pt_config_f = File::open(pt_config).await?;
            let mut pt_config_f = BufReader::new(pt_config_f);

            let mut pt_config = String::new();
            pt_config_f.read_to_string(&mut pt_config).await?;

            let pt_config = serde_json::from_str::<serde_json::Value>(&pt_config)?;

            let bridges = pt_config["bridges"]["snowflake"]
                .as_array()
                .ok_or(anyhow!("Failed to get snowflake bridges"))?;

            let bridges: Vec<String> = bridges
                .iter()
                .filter_map(|e| e.as_str())
                .map(|e| format!("Bridge {}", e))
                .collect();

            let bridges = fix_bridges(bridges);

            config = format!(
                "{}
ClientTransportPlugin snowflake exec ./{} -log snowflake.log

{}
UseBridges 1",
                config,
                get_rel_snowflake(),
                bridges.join("\n")
            );
        }

        Ok(config)
    }
}
