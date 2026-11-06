import type { ReactElement } from "react";
import {
  Card,
  Center,
  Text,
  Grid,
  Group,
  Loader,
  NumberInput,
} from "@mantine/core";

import type {
  Profile,
  Workspace,
  Device,
  IntervalConfig,
} from "../../storage/types";
import { client } from "../../storage/client";

interface IProps {
  workspace: Workspace;
  device: Device;
  profile: Profile;
  configuration?: Record<string, any>;
  configuration_desired?: Record<string, any>;
}

export function DeviceConfiguration(props: IProps) {
  const onChange = (key: string): ((val: any) => void) => {
    return async (val: any) => {
      const body = {
        [key]: val,
      };

      await client.PATCH(
        "/api/workspaces/{workspace_name}/devices/{device_name}/configuration",
        {
          params: {
            path: {
              workspace_name: props.workspace.name,
              device_name: props.device.name,
            },
          },
          body: body,
        },
      );
    };
  };

  let config = Object.entries(props.profile.configuration_schema).map(
    ([k, v]) => {
      let content: ReactElement = <></>;
      let inSync = false;

      if (
        "interval" in v.configuration &&
        typeof v.configuration.interval === "object"
      ) {
        const val = props.configuration?.[k] as number | undefined;
        const desired = props.configuration_desired?.[k] as number | undefined;
        inSync = desired === undefined || val === desired;
        content = (
          <IntervalConfig
            value={val}
            config={v.configuration.interval}
            desired={desired}
            name={v.name}
            onChange={onChange(k)}
          />
        );
      }

      return (
        <Grid.Col span={{ base: 12, md: 6, lg: 4 }} key={k}>
          <Card withBorder>
            <Card.Section inheritPadding py="xs">
              <Group justify="space-between">
                <Text fw={500}>{v.name}</Text>
                {!inSync && <Loader size="xs" />}
              </Group>
            </Card.Section>
            {content}
          </Card>
        </Grid.Col>
      );
    },
  );

  return config;
}

function IntervalConfig(props: {
  name: string;
  config: IntervalConfig;
  value?: number;
  desired?: number;
  onChange: (value: any) => void;
}) {
  if (props.value === undefined) {
    return <Card.Section py="md">No data.</Card.Section>;
  } else {
    return (
      <Center py="md">
        <NumberInput
          min={props.config.min}
          max={props.config.max}
          suffix={props.config.unit}
          value={
            props.desired === undefined || props.value === props.desired
              ? props.value
              : props.desired
          }
          onChange={(value) => props.onChange(value)}
        />
      </Center>
    );
  }
}
