use zbus::{Connection, proxy};

#[proxy(
    interface = "org.freedesktop.ScreenSaver",
    default_service = "org.freedesktop.ScreenSaver",
    default_path = "/org/freedesktop/ScreenSaver"
)]
pub trait ScreenSaver {
    fn inhibit(&self, app_name: &str, reason: &str) -> zbus::Result<u32>;
    fn un_inhibit(&self, cookie: u32) -> zbus::Result<()>;
}

pub async fn create_dbus_connection() -> zbus::Result<Connection> {
    Connection::session().await
}

pub async fn apply_inhibit(connect: &Connection) -> zbus::Result<()> {
    let proxy = ScreenSaverProxy::new(connect).await?;
    proxy
        .inhibit("cosmic-applet-caffeine", "User want to block idle.")
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::dbus::ScreenSaverProxy;
    use tokio;
    #[tokio::test]
    async fn screensaver_inhibit() {
        let connection = zbus::Connection::session().await.unwrap();
        let screensaver = ScreenSaverProxy::new(&connection).await.unwrap();
        let cookie = screensaver.inhibit("test app", "testing").await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        screensaver.un_inhibit(cookie).await.unwrap();
    }
}
