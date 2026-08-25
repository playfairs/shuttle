use crate::error::Result;
use std::path::Path;

pub async fn handle_serve(
    path: &Path,
    bind: &str,
    port: u16,
    public: bool,
    token: bool,
) -> Result<()> {
    let bind_addr = if public {
        "0.0.0.0".to_string()
    } else {
        bind.to_string()
    };

    crate::server::serve(path, &bind_addr, port, token).await
}
