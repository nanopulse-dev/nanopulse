local M = require("regions.lora")

-- Region name
M.name = "EU868"

-- Region description
M.description = [[
EU868 (8 channels)
]]

M.version = "1.0.0"

M.uplink_channels = {
	[0] = { frequency = 868100000, data_rates = { 0, 1, 2, 3, 4, 5 } },
	[1] = { frequency = 868300000, data_rates = { 0, 1, 2, 3, 4, 5 } },
	[2] = { frequency = 868500000, data_rates = { 0, 1, 2, 3, 4, 5 } },
	[3] = { frequency = 867100000, data_rates = { 0, 1, 2, 3, 4, 5 } },
	[4] = { frequency = 867300000, data_rates = { 0, 1, 2, 3, 4, 5 } },
	[5] = { frequency = 867500000, data_rates = { 0, 1, 2, 3, 4, 5 } },
	[6] = { frequency = 867700000, data_rates = { 0, 1, 2, 3, 4, 5 } },
	[7] = { frequency = 867900000, data_rates = { 0, 1, 2, 3, 4, 5 } },
}

M.downlink_channels = M.uplink_channels

M.uplink_downlink_channel_mapping = {
	[0] = 0,
	[1] = 1,
	[2] = 2,
	[3] = 3,
	[4] = 4,
	[5] = 5,
	[6] = 6,
	[7] = 7,
}

M.data_rates = {
	[0] = { modulation = "LORA", bandwidth = 125000, spreading_factor = 12, coding_rate = "CR4_5" },
	[1] = { modulation = "LORA", bandwidth = 125000, spreading_factor = 11, coding_rate = "CR4_5" },
	[2] = { modulation = "LORA", bandwidth = 125000, spreading_factor = 10, coding_rate = "CR4_5" },
	[3] = { modulation = "LORA", bandwidth = 125000, spreading_factor = 9, coding_rate = "CR4_5" },
	[4] = { modulation = "LORA", bandwidth = 125000, spreading_factor = 8, coding_rate = "CR4_5" },
	[5] = { modulation = "LORA", bandwidth = 125000, spreading_factor = 7, coding_rate = "CR4_5" },
}

M.uplink_downlink_data_rate_mapping = {
	[0] = 0,
	[1] = 1,
	[2] = 2,
	[3] = 3,
	[4] = 4,
	[5] = 5,
}

M.downlink_tx_power_eirp = 16

return M
