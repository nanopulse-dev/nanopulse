use std::collections::HashMap;

use nanopulse_structs::gateway::{
    ChannelConfiguration, DataRateConfiguration, RxEvent, TxAckStatus, TxCommand,
};

use anyhow::Result;

pub trait Radio: Send + Sync {
    fn start(&mut self) -> Result<()>;

    fn stop(&mut self) -> Result<()>;

    fn get_id(&self) -> Result<[u8; 8]>;

    fn configure_data_rates(
        &mut self,
        data_rates: &HashMap<u8, DataRateConfiguration>,
    ) -> Result<()>;

    fn configure_channels(
        &mut self,
        uplink_channels: &HashMap<u8, ChannelConfiguration>,
        downlink_channels: &HashMap<u8, ChannelConfiguration>,
    ) -> Result<()>;

    fn send(&mut self, frame: &TxCommand) -> Result<(), TxAckStatus>;

    fn receive(&self) -> Result<Vec<RxEvent>>;
}
