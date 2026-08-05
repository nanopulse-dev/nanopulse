use std::ffi::{CStr, CString};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use anyhow::{Result, anyhow};

pub mod bindings;

pub static CONCENTRATOR: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
const MAX_PKT: usize = 8;

pub fn set_i2c_device_path(path: &str) -> Result<()> {
    let _guard = CONCENTRATOR.lock().unwrap();
    let path = CString::new(path).unwrap();
    let ret = unsafe { bindings::lgw_i2c_set_path(path.as_ptr()) };
    if ret != 0 {
        return Err(anyhow!("lgw_i2c_set_path failed"));
    }

    Ok(())
}

/// Set I2C temperature device addr.
pub fn set_i2c_temp_sensor_addr(addr: u8) -> Result<()> {
    let _guard = CONCENTRATOR.lock().unwrap();
    let ret = unsafe { bindings::lgw_i2c_set_temp_sensor_addr(addr) };
    if ret != 0 {
        return Err(anyhow!("lgw_i2c_set_temp_sensor addr failed"));
    }

    Ok(())
}

/// Configure the gateway board.
pub fn board_setconf(conf: &bindings::lgw_conf_board_s) -> Result<()> {
    let mut conf = *conf;
    let _guard = CONCENTRATOR.lock().unwrap();
    let ret = unsafe { bindings::lgw_board_setconf(&mut conf) };
    if ret != 0 {
        return Err(anyhow!("lgw_board_setconf failed"));
    }

    Ok(())
}

/// Configure an RF chain (must configure before start).
pub fn rxrf_setconf(rf_chain: u8, conf: &bindings::lgw_conf_rxrf_s) -> Result<()> {
    let mut conf = *conf;
    let _guard = CONCENTRATOR.lock().unwrap();
    let ret = unsafe { bindings::lgw_rxrf_setconf(rf_chain, &mut conf) };
    if ret != 0 {
        return Err(anyhow!("lgw_rxrf_setconf failed"));
    }

    Ok(())
}

/// Configure an IF chain + modem (must configure before start).
pub fn rxif_setconf(if_chain: u8, conf: &bindings::lgw_conf_rxif_s) -> Result<()> {
    let mut conf = *conf;
    let _guard = CONCENTRATOR.lock().unwrap();
    let ret = unsafe { bindings::lgw_rxif_setconf(if_chain, &mut conf) };
    if ret != 0 {
        return Err(anyhow!("lgw_rxif_setconf failed"));
    }

    Ok(())
}

pub fn txgain_setconf(rf_chain: u8, conf: &bindings::lgw_tx_gain_lut_s) -> Result<()> {
    let mut conf = *conf;
    let _guard = CONCENTRATOR.lock().unwrap();
    let ret = unsafe { bindings::lgw_txgain_setconf(rf_chain, &mut conf) };
    if ret != 0 {
        return Err(anyhow!("lgw_txgain_setconf failed"));
    }

    Ok(())
}

pub fn start() -> Result<()> {
    let _guard = CONCENTRATOR.lock().unwrap();
    let ret = unsafe { bindings::lgw_start() };
    if ret != 0 {
        return Err(anyhow!("lgw_start failed"));
    }

    Ok(())
}

pub fn stop() -> Result<()> {
    let _guard = CONCENTRATOR.lock().unwrap();
    let ret = unsafe { bindings::lgw_stop() };
    if ret != 0 {
        return Err(anyhow!("lgw_stop failed"));
    }

    Ok(())
}

pub fn get_eui() -> Result<[u8; 8]> {
    let _guard = CONCENTRATOR.lock().unwrap();
    let mut eui = 0u64;
    let ret = unsafe { bindings::lgw_get_eui(&mut eui) };
    if ret != 0 {
        return Err(anyhow!("lgw_get_eui failed"));
    }

    Ok(eui.to_be().to_ne_bytes())
}

pub fn version_info() -> String {
    unsafe {
        CStr::from_ptr(bindings::lgw_version_info())
            .to_string_lossy()
            .into_owned()
    }
}

pub fn send(pkt: &mut bindings::lgw_pkt_tx_s) -> Result<()> {
    let _guard = CONCENTRATOR.lock().unwrap();
    let ret = unsafe { bindings::lgw_send(pkt) };
    if ret != 0 {
        return Err(anyhow!("lgw_send failed"));
    }

    Ok(())
}

pub fn receive() -> Result<Vec<bindings::lgw_pkt_rx_s>> {
    let mut packets: [bindings::lgw_pkt_rx_s; MAX_PKT] = [Default::default(); MAX_PKT];

    let _guard = CONCENTRATOR.lock().unwrap();
    let ret = unsafe { bindings::lgw_receive(MAX_PKT.try_into().unwrap(), packets.as_mut_ptr()) };
    if ret == -1 {
        return Err(anyhow!("lgw_receive failed"));
    }

    Ok(packets[0..ret as usize].into())
}

pub fn get_instcnt() -> Result<u32> {
    let mut cnt = 0u32;

    let _guard = CONCENTRATOR.lock().unwrap();
    let ret = unsafe { bindings::lgw_get_instcnt(&mut cnt) };
    if ret != 0 {
        return Err(anyhow!("lgw_get_instcnt failed"));
    }

    Ok(cnt)
}

pub fn time_on_air(pkt: &bindings::lgw_pkt_tx_s) -> Duration {
    let _guard = CONCENTRATOR.lock().unwrap();
    let ms = unsafe { bindings::lgw_time_on_air(pkt) };
    Duration::from_millis(ms as u64)
}
