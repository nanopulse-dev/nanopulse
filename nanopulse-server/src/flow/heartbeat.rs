use std::sync::Arc;

use anyhow::Result;
use tracing::{error, info, trace};

use nanopulse_structs::gateway::HeartbeatEvent;

use crate::errors::Error;
use crate::region;
use crate::storage::gateway;

pub async fn handle(gateway_name: &str, heartbeat_pl: HeartbeatEvent) {
    if let Err(e) = _handle(gateway_name, heartbeat_pl).await {
        error!(error = %e, "Error handling heartbeat event");
    }
}

async fn _handle(gateway_name: &str, heartbeat_pl: HeartbeatEvent) -> Result<(), Error> {
    let gateway = gateway::get_by_name(gateway_name).await?;
    let region = region::get(&gateway.region_module)?;

    let mut ctx = Context {
        gateway,
        heartbeat_pl,
        region,
    };
    let res = ctx.handle().await;

    if let Err(Error::Abort) = res {
        Ok(())
    } else {
        res
    }
}

struct Context {
    gateway: gateway::Gateway,
    heartbeat_pl: HeartbeatEvent,
    region: Arc<region::Region>,
}

impl Context {
    async fn handle(&mut self) -> Result<(), Error> {
        self.push_config().await?;
        self.update_gateway().await?;

        Ok(())
    }

    async fn push_config(&self) -> Result<(), Error> {
        if self.heartbeat_pl.config_version == self.region.version {
            trace!("Gateway config already up-to-date");
            return Ok(());
        }

        info!(gateway_config = %self.heartbeat_pl.config_version, config = %self.region.version, "Gateway configuration is out-of-date, sending new configuration");
        crate::gateway::sync_configuration(&self.gateway.name, self.region.clone()).await?;

        Ok(())
    }

    async fn update_gateway(&mut self) -> Result<()> {
        trace!("Updating gateway");
        gateway::heartbeat_update(&self.gateway.name).await?;

        Ok(())
    }
}
