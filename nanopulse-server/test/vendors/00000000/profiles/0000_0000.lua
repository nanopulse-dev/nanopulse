local M = require("vendors.00000000.profiles.0000")

M.region = require("regions.lora.eu868")

function M.telemetry_schema()
	return {
		temperature = {
			name = "Temperature",
			component = {
				sensor = {
					temperature = {
						unit = "C",
						min = -15,
						max = 80,
					},
				},
			},
		},
	}
end

function M.decode_telemetry(payload)
	if #payload ~= 2 then
		error("invalid payload length")
	end

	return {
		temperature = np.decode_i16_le(payload[1], payload[2]) / 10.0,
	}
end

function M.state_schema()
	return {
		led = {
			name = "LED",
			component = {
				switch = {
					default = false,
				},
			},
		},
	}
end

function M.decode_state(payload)
	if #payload ~= 1 then
		error("invalid payload length")
	end

	if payload[1] == 0x01 then
		return { led = true }
	end

	return { led = false }
end

function M.encode_state(payload)
	if payload.led == true then
		return { 0x01 }
	end

	return { 0x00 }
end

function M.configuration_schema()
	return {
		telemetry_interval = {
			name = "Telemetry interval",
			configuration = {
				interval = {
					unit = "s",
					default = 30,
					min = 5,
					max = 300,
				},
			},
		},
	}
end

return M
