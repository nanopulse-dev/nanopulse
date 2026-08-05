pub enum DataRate {
    #[cfg(feature = "lora_sx126x")]
    Lora(lora::Lora),
    Fsk,
}

#[cfg(feature = "lora_sx126x")]
pub mod lora {
    pub use lora_modulation::BaseBandModulationParams;
    pub use lora_phy::mod_params::{Bandwidth, CodingRate, SpreadingFactor};

    pub struct Lora {
        pub spreading_factor: SpreadingFactor,
        pub coding_rate: CodingRate,
        pub bandwidth: Bandwidth,
    }

    impl Lora {
        pub fn get_bb_mod_params(&self) -> BaseBandModulationParams {
            BaseBandModulationParams::new(self.spreading_factor, self.bandwidth, self.coding_rate)
        }
    }
}
