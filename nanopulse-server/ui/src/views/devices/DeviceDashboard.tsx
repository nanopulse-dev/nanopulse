import { Grid, DataList } from "@mantine/core";
import { JsonTree } from "@gfazioli/mantine-json-tree";

import type { Device, Workspace } from "../../storage/types";
import { formatDateTime } from "../helpers";

interface IProps {
  device: Device;
  workspace: Workspace;
}

export function DeviceDashboard(props: IProps) {
  return (
    <Grid>
      <Grid.Col span={{ base: 12, md: 6, lg: 4 }}>
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
            <DataList.ItemLabel>Description:</DataList.ItemLabel>
            <DataList.ItemValue>{props.device.description}</DataList.ItemValue>
          </DataList.Item>
        </DataList>
      </Grid.Col>
      <Grid.Col span={{ base: 12, md: 6, lg: 4 }}>
        <JsonTree
          title="Telemetry"
          size="sm"
          data={props.device.telemetry}
          maxDepth={1}
          defaultExpanded
        />
      </Grid.Col>
      <Grid.Col span={{ base: 12, md: 6, lg: 4 }}>
        <JsonTree
          style={{ width: "100%" }}
          title="State"
          data={props.device.state}
          maxDepth={1}
          defaultExpanded
        />
      </Grid.Col>
    </Grid>
  );
}
