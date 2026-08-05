import type { components } from "./schema";

export type Workspace = components["schemas"]["Workspace"];

export type Device = components["schemas"]["Device"];

export type Gateway = components["schemas"]["Gateway"];

export type RegionListItem = components["schemas"]["RegionListItem"];

export interface WsEvent {
  log?: Log;
  key_exchange_event?: KeyExchange;
  activation_event?: Activation;
  telemetry_event?: Telemetry;
  state_event?: State;
}

export interface Log {
  timestamp: string;
  level: string;
  target: string;
  attributes: Map<string, string>;
  message: string;
}

export interface KeyExchange {
  timestamp: string;
  workspace_name: string;
  device_short_id: string;
  device_public_key: string;
}

export interface DeviceInfo {
  device_name: string;
  device_short_id: string;
  workspace_name: string;
  vendor_name: string;
}

export interface Activation {
  timestamp: string;
  device_info: DeviceInfo;
  telemetry_schema: any;
  state_schema: any;
}

export interface Telemetry {
  timestamp: string;
  device_info: DeviceInfo;
  telemetry: any;
}

export interface State {
  timestamp: string;
  device_info: DeviceInfo;
  state: any;
}
