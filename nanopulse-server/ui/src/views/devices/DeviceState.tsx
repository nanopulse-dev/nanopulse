import type { ReactElement } from "react";
import { Card, Center, Text, Grid, Switch } from "@mantine/core";

import type { Profile } from "../../storage/types";

interface IProps {
  profile: Profile;
  state?: Record<string, any>;
}

export function DeviceState(props: IProps) {
  let state = Object.entries(props.profile.state_schema).map(([k, v]) => {
    let content: ReactElement = <></>;

    if ("switch" in v.component && typeof v.component.switch === "object") {
      const val = props.state?.[k] as boolean | undefined;
      content = <SwitchToggle value={val} name={v.name} />;
    }

    return (
      <Grid.Col span={{ base: 12, md: 6, lg: 4 }} key={k}>
        <Card withBorder>
          <Card.Section inheritPadding py="xs">
            <Text fw={500}>{v.name}</Text>
          </Card.Section>
          {content}
        </Card>
      </Grid.Col>
    );
  });

  return state;
}

function SwitchToggle(props: { name: string; value?: boolean }) {
  if (props.value === undefined) {
    return <Card.Section py="md">No data.</Card.Section>;
  } else {
    return (
      <Center py="md">
        <Switch checked={props.value} readOnly label={`${props.name} switch`} />
      </Center>
    );
  }
}
