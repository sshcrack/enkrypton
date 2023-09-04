use url::Url;

pub trait UrlOnion {
    fn to_hostname(&self) -> Option<String>;
}

impl UrlOnion for Url {
    /**
     * REVIEW - Yeah I know, this is quite janky but it gets the job done
     */
    fn to_hostname(&self) -> Option<String> {
        let res = self.host_str();
        if let Some(host) = res {
            let without = host.replace(".onion", "");
            return Some(without);
        }

        return None;
    }
}
