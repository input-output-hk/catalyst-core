use color_eyre::Result;
use microtype::secrecy::ExposeSecret;

use crate::config::DbConfig;

pub async fn connect(
    DbConfig {
        name,
        user,
        host,
        password,
    }: &DbConfig,
) -> Result<()> {
    let connection_str = format!(
        "postgres://{user}:{password}@{host}/{name}",
        password = password.expose_secret()
    );

    Ok(())
}
