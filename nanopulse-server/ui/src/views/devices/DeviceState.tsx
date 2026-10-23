import type { ReactElement } from "react";
import { Card, Center, Text, Grid, Switch, Group, Loader } from "@mantine/core";

import type { Profile, Workspace, Device } from "../../storage/types";
import { client } from "../../storage/client";

interface IProps {
  workspace: Workspace;
  device: Device;
  profile: Profile;
  state?: Record<string, any>;
  state_desired?: Record<string, any>;
}

export function DeviceState(props: IProps) {
  const onChange = (key: string): ((val: any) => void) => {
    return async (val: any) => {
      const body = {
        [key]: val,
      };

      await client.PATCH(
        "/api/workspaces/{workspace_name}/devices/{device_name}/state",
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

  let state = Object.entries(props.profile.state_schema).map(([k, v]) => {
    let content: ReactElement = <></>;
    let inSync = false;

    if ("switch" in v.component && typeof v.component.switch === "object") {
      const val = props.state?.[k] as boolean | undefined;
      const desired = props.state_desired?.[k] as boolean | undefined;
      inSync = desired === undefined || val === desired;
      content = (
        <SwitchToggle
          value={val}
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
  });

  return state;
}

function SwitchToggle(props: {
  name: string;
  value?: boolean;
  desired?: boolean;
  onChange: (value: any) => void;
}) {
  if (props.value === undefined) {
    return <Card.Section py="md">No data.</Card.Section>;
  } else {
    return (
      <Center py="md">
        <Switch
          checked={props.desired || props.value}
          label={`${props.name} switch`}
          onChange={(event) => props.onChange(event.currentTarget.checked)}
        />
      </Center>
    );
  }
}
