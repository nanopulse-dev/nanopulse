local M = require("vendors.00000000.profiles.0000")

M.region = require("regions.lora.eu868")

-- Returns the telemetry schema.
function M.telemetry_schema()
	return {
		temperature = {
			name = "Temperature",
			component = {
				sensor = {
					temperature = {
						unit = "C",
					},
				},
			},
		},
	}
end

-- Decode telemetry.
function M.decode_telemetry(payload)
	if #payload ~= 2 then
		error("invalid payload length")
	end

	return {
		temperature = np.decode_i16_le(payload[1], payload[2]) / 10.0,
	}
end

-- Returns the state schema.
function M.state_schema()
	return {
		led = {
			name = "LED",
			component = {
				switch = {},
			},
		},
	}
end

-- Default state.
function M.default_state()
	return {
		led = false,
	}
end

-- Decode state payload.
function M.decode_state(payload)
	if #payload ~= 1 then
		error("invalid payload length")
	end

	if payload[1] == 0x01 then
		return { led = true }
	end

	return { led = false }
end

-- Encode state payload
function M.encode_state(payload)
	if payload.led == true then
		return { 0x01 }
	end

	return { 0x00 }
end

return M
