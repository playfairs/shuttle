use std::net::IpAddr;
use std::net::SocketAddr;

pub struct NetworkInfo {
    pub local_ips: Vec<IpAddr>,
    pub bind_addr: SocketAddr,
    pub port: u16,
}

impl NetworkInfo {
    pub fn new(bind_addr: SocketAddr) -> Self {
        let local_ips = get_local_ips();
        NetworkInfo {
            local_ips,
            bind_addr,
            port: bind_addr.port(),
        }
    }

    pub fn get_access_urls(&self) -> Vec<String> {
        let mut urls = Vec::new();

        let bind_ip = self.bind_addr.ip();

        if bind_ip.is_loopback() {
            urls.push(format!("http://localhost:{}", self.port));
            urls.push(format!("http://127.0.0.1:{}", self.port));
        } else if bind_ip.is_unspecified() {
            urls.push(format!("http://localhost:{}", self.port));

            for ip in &self.local_ips {
                if !ip.is_loopback() {
                    urls.push(format!("http://{}:{}", ip, self.port));
                }
            }
        } else {
            urls.push(format!("http://{}:{}", bind_ip, self.port));
        }

        urls.sort();
        urls.dedup();
        urls
    }

    pub fn show_access_info(&self, token: Option<&str>) {
        let urls = self.get_access_urls();

        println!("\n-------------------------------------------------------");
        println!("|         File Server Ready for Download              |");
        println!("-------------------------------------------------------\n");

        if urls.len() == 1 {
            let url_with_token = if let Some(t) = token {
                format!("{}?token={}", urls[0], t)
            } else {
                urls[0].clone()
            };
            println!("Access URL: {}", url_with_token);
        } else {
            println!("Access URLs:");
            for url in &urls {
                let url_with_token = if let Some(t) = token {
                    format!("{}?token={}", url, t)
                } else {
                    url.clone()
                };
                println!("  - {}", url_with_token);
            }
        }

        if token.is_some() {
            println!("\nAccess Token Required");
        }

        println!("\nTips:");
        println!("  • Share the URL above with anyone on your network");
        println!("  • To access from outside your network:");
        println!("    - Set up port forwarding on your router");
        println!("    - Find your public IP: https://whatismyipaddress.com");
        println!("    - Use: http://<PUBLIC_IP>:{}", self.port);
        println!("  • Press Ctrl+C to stop the server\n");
    }
}

fn get_local_ips() -> Vec<IpAddr> {
    match local_ip_address::list_afinet_netifas() {
        Ok(interfaces) => interfaces.into_iter().map(|(_, ip)| ip).collect(),
        Err(_) => vec![],
    }
}
