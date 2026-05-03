use zbus::{proxy, Connection};
use anyhow::Result;
use std::collections::HashMap;

#[proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
trait NetworkManager {
    fn get_devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
    fn activate_connection(
        &self,
        connection: zbus::zvariant::OwnedObjectPath,
        device: zbus::zvariant::OwnedObjectPath,
        specific_object: zbus::zvariant::OwnedObjectPath,
    ) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
    fn add_and_activate_connection(
        &self,
        connection: HashMap<&str, HashMap<&str, zbus::zvariant::Value<'_>>>,
        device: zbus::zvariant::OwnedObjectPath,
        specific_object: zbus::zvariant::OwnedObjectPath,
    ) -> zbus::Result<(zbus::zvariant::OwnedObjectPath, zbus::zvariant::OwnedObjectPath)>;
    #[zbus(property)]
    fn active_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Connection.Active",
    default_service = "org.freedesktop.NetworkManager"
)]
trait ActiveConnection {
    #[zbus(property)]
    fn id(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn connection_type(&self) -> zbus::Result<String>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Device",
    default_service = "org.freedesktop.NetworkManager"
)]
trait Device {
    #[zbus(property)]
    fn interface(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn device_type(&self) -> zbus::Result<u32>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Device.Wireless",
    default_service = "org.freedesktop.NetworkManager"
)]
trait Wireless {
    fn get_access_points(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
    fn request_scan(&self, options: HashMap<String, zbus::zvariant::Value<'_>>) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.AccessPoint",
    default_service = "org.freedesktop.NetworkManager"
)]
trait AccessPoint {
    #[zbus(property)]
    fn ssid(&self) -> zbus::Result<Vec<u8>>;
    #[zbus(property)]
    fn strength(&self) -> zbus::Result<u8>;
    #[zbus(property)]
    fn frequency(&self) -> zbus::Result<u32>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Settings",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager/Settings"
)]
trait Settings {
    fn list_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Settings.Connection",
    default_service = "org.freedesktop.NetworkManager"
)]
trait NMConnection {
    fn get_settings(&self) -> zbus::Result<HashMap<String, HashMap<String, zbus::zvariant::OwnedValue>>>;
    fn delete(&self) -> zbus::Result<()>;
}

pub struct NMService {
    conn: Connection,
}

impl NMService {
    pub async fn new() -> Result<Self> {
        let conn = Connection::system().await?;
        Ok(Self { conn })
    }

    pub async fn list_wifi_devices(&self) -> Result<Vec<zbus::zvariant::OwnedObjectPath>> {
        let nm = NetworkManagerProxy::new(&self.conn).await?;
        let device_paths = nm.get_devices().await?;
        let mut wifi_devices = Vec::new();
        for path in device_paths {
            let device = DeviceProxy::builder(&self.conn).path(path.clone())?.build().await?;
            if device.device_type().await? == 2 {
                wifi_devices.push(path);
            }
        }
        Ok(wifi_devices)
    }

    pub async fn scan(&self, device_path: zbus::zvariant::OwnedObjectPath) -> Result<()> {
        let wireless = WirelessProxy::builder(&self.conn).path(device_path)?.build().await?;
        wireless.request_scan(HashMap::new()).await?;
        Ok(())
    }

    pub async fn get_access_points(&self, device_path: zbus::zvariant::OwnedObjectPath) -> Result<Vec<APInfo>> {
        let wireless = WirelessProxy::builder(&self.conn).path(device_path)?.build().await?;
        let ap_paths = wireless.get_access_points().await?;
        let mut aps = Vec::new();
        for path in ap_paths {
            let ap_proxy = AccessPointProxy::builder(&self.conn).path(path.clone())?.build().await?;
            let ssid_bytes = ap_proxy.ssid().await?;
            let ssid = String::from_utf8_lossy(&ssid_bytes).to_string();
            if ssid.is_empty() { continue; }
            let strength = ap_proxy.strength().await?;
            let frequency = ap_proxy.frequency().await?;
            aps.push(APInfo { ssid, strength, frequency, path });
        }
        aps.sort_by(|a, b| a.ssid.cmp(&b.ssid).then_with(|| b.strength.cmp(&a.strength)));
        aps.dedup_by(|a, b| a.ssid == b.ssid);
        aps.sort_by(|a, b| b.strength.cmp(&a.strength));
        Ok(aps)
    }

    pub async fn get_saved_connections(&self) -> Result<Vec<SavedConnection>> {
        let settings = SettingsProxy::new(&self.conn).await?;
        let paths = settings.list_connections().await?;
        let mut connections = Vec::new();
        for path in paths {
            let conn_proxy = NMConnectionProxy::builder(&self.conn).path(path.clone())?.build().await?;
            if let Ok(s) = conn_proxy.get_settings().await {
                if let Some(conn_settings) = s.get("connection") {
                    let conn_type = conn_settings.get("type")
                        .map(|v| v.to_string().trim_matches('"').to_string())
                        .unwrap_or_default();
                    if conn_type != "802-11-wireless" && conn_type != "802-3-ethernet" {
                        continue;
                    }
                    if let Some(id_val) = conn_settings.get("id") {
                        let id = id_val.to_string().trim_matches('"').to_string();
                        connections.push(SavedConnection { id, path });
                    }
                }
            }
        }
        connections.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(connections)
    }

    pub async fn get_active_ssids(&self) -> Result<Vec<String>> {
        let nm = NetworkManagerProxy::new(&self.conn).await?;
        let paths = nm.active_connections().await?;
        let mut ssids = Vec::new();
        for path in paths {
            let active = ActiveConnectionProxy::builder(&self.conn).path(path)?.build().await?;
            if let Ok(id) = active.id().await {
                ssids.push(id);
            }
        }
        Ok(ssids)
    }

    pub async fn activate_connection(
        &self,
        conn_path: zbus::zvariant::OwnedObjectPath,
        device_path: zbus::zvariant::OwnedObjectPath,
        ap_path: Option<zbus::zvariant::OwnedObjectPath>,
    ) -> Result<()> {
        let nm = NetworkManagerProxy::new(&self.conn).await?;
        let ap = ap_path.unwrap_or_else(|| zbus::zvariant::ObjectPath::from_str_unchecked("/").into());
        nm.activate_connection(conn_path, device_path, ap).await?;
        Ok(())
    }

    pub async fn add_and_activate_wifi(
        &self,
        ssid: &str,
        password: &str,
        device_path: zbus::zvariant::OwnedObjectPath,
        ap_path: zbus::zvariant::OwnedObjectPath,
    ) -> Result<()> {
        let nm = NetworkManagerProxy::new(&self.conn).await?;
        let mut conn_map: HashMap<&str, HashMap<&str, zbus::zvariant::Value<'_>>> = HashMap::new();
        let mut connection: HashMap<&str, zbus::zvariant::Value<'_>> = HashMap::new();
        connection.insert("id", zbus::zvariant::Value::from(ssid));
        connection.insert("type", zbus::zvariant::Value::from("802-11-wireless"));
        conn_map.insert("connection", connection);
        let mut wireless: HashMap<&str, zbus::zvariant::Value<'_>> = HashMap::new();
        wireless.insert("ssid", zbus::zvariant::Value::from(ssid.as_bytes().to_vec()));
        conn_map.insert("802-11-wireless", wireless);
        let mut security: HashMap<&str, zbus::zvariant::Value<'_>> = HashMap::new();
        security.insert("key-mgmt", zbus::zvariant::Value::from("wpa-psk"));
        security.insert("psk", zbus::zvariant::Value::from(password));
        conn_map.insert("802-11-wireless-security", security);
        nm.add_and_activate_connection(conn_map, device_path, ap_path).await?;
        Ok(())
    }

    pub async fn delete_connection(&self, conn_path: zbus::zvariant::OwnedObjectPath) -> Result<()> {
        let conn_proxy = NMConnectionProxy::builder(&self.conn).path(conn_path)?.build().await?;
        conn_proxy.delete().await?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct APInfo {
    pub ssid: String,
    pub strength: u8,
    pub frequency: u32,
    pub path: zbus::zvariant::OwnedObjectPath,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SavedConnection {
    pub id: String,
    pub path: zbus::zvariant::OwnedObjectPath,
}
