# Counter

The _Counter_ type is a `u32` value starting at `0`. This value must never
repeat or be reset within the same security context. In case the maximum counter
value has been reached, the device must perform a new Activation Request. Each
[_Frame Type_](../frames/index.md) has its own _Counter_ value(s).

## Life-time counters

These counters must be persisted for the life-time of the device:

- Key Exchange Response Counter
- Activation Request Counter
- Activation Response Counter

## Session counters

These counters are kept in memory, and reset after a power-cycle, device reset
or after a new _Activation_:

- Telemetry Counter Up
- State Counter Up
- State Counter Down
