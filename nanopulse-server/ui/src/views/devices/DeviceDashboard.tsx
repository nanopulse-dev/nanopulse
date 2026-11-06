import type { ReactElement } from "react";
import { Grid, DataList, Card, Text } from "@mantine/core";

import type { Device, Workspace, Profile } from "../../storage/types";
import { formatDateTime } from "../helpers";
import { DeviceTelemetry } from "./DeviceTelemetry";
import { DeviceState } from "./DeviceState";
import { DeviceConfiguration } from "./DeviceConfiguration";

interface IProps {
  device: Device;
  profile: Profile | null;
  workspace: Workspace;
}

export function DeviceDashboard(props: IProps) {
  let telemetry: ReactElement = <></>;
  let state: ReactElement = <></>;
  let configuration: ReactElement = <></>;

  if (props.profile) {
    let t = props.device.telemetry as Map<string, any> | undefined;
    let s = props.device.state as Map<string, any> | undefined;
    let sd = props.device.state_desired as Map<string, any> | undefined;
    let c = props.device.configuration as Map<string, any> | undefined;
    let cd = props.device.configuration_desired as Map<string, any> | undefined;

    telemetry = <DeviceTelemetry profile={props.profile} telemetry={t} />;
    state = (
      <DeviceState
        workspace={props.workspace}
        device={props.device}
        profile={props.profile}
        state={s}
        state_desired={sd}
      />
    );
    configuration = (
      <DeviceConfiguration
        workspace={props.workspace}
        device={props.device}
        profile={props.profile}
        configuration={c}
        configuration_desired={cd}
      />
    );
  }

  return (
    <Grid>
      <Grid.Col span={{ base: 12, md: 6, lg: 4 }}>
        <Card withBorder>
          <Card.Section inheritPadding py="xs">
            <Text fw={500}>Details</Text>
          </Card.Section>
          <DataList orientation="vertical">
            <DataList.Item>
              <DataList.ItemLabel>Short ID</DataList.ItemLabel>
              <DataList.ItemValue>
                <pre>{props.device.short_id}</pre>
              </DataList.ItemValue>
            </DataList.Item>
            <DataList.Item>
              <DataList.ItemLabel>Key Exchange at:</DataList.ItemLabel>
              <DataList.ItemValue>
                {formatDateTime(props.device.key_exchange_at)}
              </DataList.ItemValue>
            </DataList.Item>
            <DataList.Item>
              <DataList.ItemLabel>Activated at:</DataList.ItemLabel>
              <DataList.ItemValue>
                {formatDateTime(props.device.activation_at)}
              </DataList.ItemValue>
            </DataList.Item>
            <DataList.Item>
              <DataList.ItemLabel>Last seen at:</DataList.ItemLabel>
              <DataList.ItemValue>
                {formatDateTime(props.device.last_seen_at)}
              </DataList.ItemValue>
            </DataList.Item>
            <DataList.Item>
              <DataList.ItemLabel>Description:</DataList.ItemLabel>
              <DataList.ItemValue>
                {props.device.description}
              </DataList.ItemValue>
            </DataList.Item>
          </DataList>
        </Card>
      </Grid.Col>
      {telemetry}
      {state}
      {configuration}
    </Grid>
  );
}
