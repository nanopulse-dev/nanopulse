# Counter

The _Counter_ type is a `u32` value starting at `0`. This value must never
repeat or be reset within the same security context. In case the maximum counter
value has been reached, the device must perform a new Activation Request.

## Life-time counters

These counters must be persisted for the life-time of the device:

- Key Exchange Response Counter
- Activation Request Counter
- Activation Response Counter

## Session counters

These counters are kept in memory, and reset after a power-cycle, device reset
or after a new _Activation_:

- Counter Up (Device > Server)
- Counter Down (Server > Device)
