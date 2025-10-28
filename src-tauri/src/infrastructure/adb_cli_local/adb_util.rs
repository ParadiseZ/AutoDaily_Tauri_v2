use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, RwLock};
use adb_client::{ADBServer, ADBServerDevice};
use tokio::sync::OnceCell;
use crate::infrastructure::adb_cli_local::adb_error::{AdbError, AdbResult};

static ADB_SERVER : OnceLock<Arc<RwLock< AdbUtil >>> = OnceLock::new();

pub fn get_adb_util() -> Arc<RwLock<AdbUtil>> {
    ADB_SERVER.get_or_init(|| {
        Arc::new(RwLock::new(AdbUtil::default()))
    }).clone()
}

pub fn init_adb_util(ipv4addr: Ipv4Addr, adb_path: PathBuf, port: u16) {
    ADB_SERVER.set(|| {
        Arc::new(RwLock::new(AdbUtil{
            server: None,
            ipv4addr,
            adb_path,
            port
        }))
    });
}
pub struct AdbServerData{
    server: Option<ADBServer>,
    ipv4addr: Ipv4Addr,
    adb_path : Option<PathBuf>,
    port: u16
}
pub struct AdbUtil{
    server: Option<ADBServer>,
    ipv4addr: Ipv4Addr,
    adb_path : Option<PathBuf>,
    port: u16
}

impl AdbUtil {
    pub fn new(ipv4addr: Ipv4Addr, adb_path: PathBuf, port: u16) -> Self {
        AdbUtil{
            server: Some(ADBServer::new(SocketAddrV4::new(ipv4addr, port))),
            ipv4addr,
            adb_path: Some(adb_path),
            port
        }
    }
    fn get_adb_path() -> AdbResult<PathBuf>{
        if let Some(path) = std::env::var_os("ADB_PATH") {
            let path = PathBuf::from(path);
            return if path.exists() {
                Ok(path)
            } else {
                Err(AdbError::AdbNotFound)
            }
        }

        // 回退：在 PATH 中查找
        #[cfg(unix)]
        const ADB_NAME: &str = "adb";
        #[cfg(windows)]
        const ADB_NAME: &str = "adb.exe";

        if let Ok(path) = which::which(ADB_NAME) {
            return Ok(path);
        }

        Err(AdbError::AdbNotFound)
    }

    pub fn check_adb_path(&mut self) -> AdbResult<PathBuf>{
        if let Some(path) = &self.adb_path {
            if path.exists() {
                return Ok(path.clone());
            }
        }
        match Self::get_adb_path() {
            Ok(path) => {
                self.adb_path = Some(path.clone());
                Ok(path.clone())
            }
            Err(err) => Err(AdbError::AdbNotFound),
        }
    }

    pub fn start_server(&mut self){
        let server = ADBServer::new(SocketAddrV4::new(self.ipv4addr, self.port));
        self.server = Some(server);
    }

    pub fn get_device_list(&self) -> AdbResult<ADBServerDevice> {
        if let Some(mut server) = &self.server {
            server.get_device().map_err(|e| AdbError::GetDevicesFailed)
        }else { 
            Err(AdbError::ServerNotStarted)
        }
    }


}

impl Default for AdbUtil {
    fn default() -> Self {
        AdbUtil{
            server :None,
            ipv4addr: Ipv4Addr::new(127, 0, 0, 1),
            adb_path : Self::get_adb_path().ok(),
            port: 5037
        }
    }
}