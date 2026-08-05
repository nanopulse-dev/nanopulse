use defmt::info;
pub use embedded_storage::nor_flash::NorFlash as Flash;

use super::{Counters, RootSecurity, get_counters_start_end, get_root_security_start_end};

pub fn get_root_security<F>(flash: &mut F) -> RootSecurity
where
    F: Flash,
{
    let (start, _) = get_root_security_start_end();
    let mut b = [0u8; RootSecurity::LENGTH];

    info!(
        "reading root-security, start: {}, length: {}",
        start,
        RootSecurity::LENGTH
    );

    flash.read(start, &mut b).unwrap();
    RootSecurity::from(&b)
}

pub fn write_root_security<F>(flash: &mut F, context: &RootSecurity)
where
    F: Flash,
{
    #[cfg(feature = "no_flash_write")]
    return;

    let (start, end) = get_root_security_start_end();
    info!("writing root-security, start: {}, end: {}", start, end);
    flash.erase(start, end).expect("erase flash");
    flash
        .write(
            start,
            Into::<[u8; RootSecurity::LENGTH]>::into(context).as_ref(),
        )
        .expect("write");
}

pub fn get_counters<F>(flash: &mut F) -> Counters
where
    F: Flash,
{
    let (start, _) = get_counters_start_end();
    let mut b = [0u8; Counters::LENGTH];

    info!(
        "reading counters, start: {}, length: {}",
        start,
        Counters::LENGTH
    );

    flash.read(start, &mut b).unwrap();
    Counters::from(&b)
}

pub fn write_counters<F>(flash: &mut F, counters: &Counters)
where
    F: Flash,
{
    #[cfg(feature = "no_flash_write")]
    return;

    let (start, end) = get_counters_start_end();
    info!("writing coutners, start: {}, end: {}", start, end);
    flash.erase(start, end).expect("erase flash");
    flash
        .write(
            start,
            Into::<[u8; Counters::LENGTH]>::into(counters).as_ref(),
        )
        .expect("write");
}
