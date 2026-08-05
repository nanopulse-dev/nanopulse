use std::sync::Arc;

use anyhow::Result;
use tracing::warn;

use nanopulse::frames::FrameType;
use nanopulse_structs::gateway::RxEvent;

use crate::region;
use crate::storage::gateway;

mod activation_request;
mod key_exchange_request;
mod state;
mod telemetry;

#[derive(Debug, Clone)]
pub struct RxInfo {
    pub gateway_name: String,
    pub context: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct RxFrameSet {
    pub ul_ch: u8,
    pub ul_dr: u8,
    pub region: Arc<region::Region>,
    pub frame: nanopulse::frames::Frame,
    pub rx_info: Vec<RxInfo>,
}

impl RxFrameSet {
    pub fn get_downlink_gateway(&self) -> RxInfo {
        // TODO: implement gateway selection
        self.rx_info[0].clone()
    }
}

pub async fn deduplicate_and_handle(gateway_name: &str, rx_pl: &RxEvent) -> Result<()> {
    let gw = gateway::get_by_name(gateway_name).await?;
    let reg = region::get(&gw.region_module)?;

    let dedup = RxFrameSet {
        ul_ch: rx_pl.channel,
        ul_dr: rx_pl.data_rate,
        frame: nanopulse::frames::Frame::try_from(rx_pl.payload.as_ref())?,
        rx_info: vec![RxInfo {
            gateway_name: gateway_name.into(),
            context: rx_pl.context.clone(),
        }],
        region: reg,
    };

    match dedup.frame.frame_type() {
        FrameType::KeyExchangeRequest => key_exchange_request::Flow::new(dedup)?.handle().await,
        FrameType::ActivationRequest => activation_request::Flow::new(dedup)?.handle().await,
        FrameType::Telemetry => telemetry::Flow::new(dedup)?.handle().await,
        FrameType::State => state::Flow::new(dedup)?.handle().await,
        _ => {
            warn!(frame_type = ?dedup.frame.frame_type(), "Invalid frame-type received");
        }
    }

    Ok(())
}
