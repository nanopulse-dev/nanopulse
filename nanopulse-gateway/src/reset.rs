use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use gpiocdev::{Request, line::Value};
use tracing::info;

pub struct Reset {
    sequence: Vec<(Request, Value, Duration)>,
}

impl Reset {
    pub fn new() -> Self {
        Reset { sequence: vec![] }
    }

    // The default will be the inverted value. If duration is 0, the line will remain on the set
    // value at reset.
    pub fn add(&mut self, chip: &str, line: u32, value: Value, duration: Duration) -> Result<()> {
        info!(
            chip = chip,
            line = line,
            value = ?value,
            duration = ?duration,
            "Adding pin configuration"
        );
        self.sequence.push((
            Request::builder()
                .on_chip(chip)
                .with_line(line)
                .as_output(value.not())
                .request()?,
            value,
            duration,
        ));

        sleep(Duration::from_millis(100));

        Ok(())
    }

    pub fn reset(&self) -> Result<()> {
        info!("Triggering chip reset");

        for (line, value, duration) in &self.sequence {
            line.set_lone_value(*value)?;

            if !duration.is_zero() {
                sleep(*duration);
                line.set_lone_value(value.not())?;
                sleep(*duration);
            } else {
                sleep(Duration::from_millis(100));
            }
        }

        Ok(())
    }
}
