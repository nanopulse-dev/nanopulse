# Architecture

```d2
vars: {
	d2-config: {
		layout-engine: elk
	}
}

vm: Cloud / VM
np_gateway: NanoPulse Gateway
np_device: NanoPulse Device

vm: {
    np_server: NanoPulse Server
	mqtt_broker: MQTT Broker

	np_server <> mqtt_broker
}

np_gateway <> vm.mqtt_broker
np_device <> np_gateway {
    style.stroke-dash: 3
}
```
