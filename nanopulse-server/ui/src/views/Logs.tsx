import { useEffect, useState } from "react";

import { Loader, Table, Group, Badge } from "@mantine/core";

import type { Workspace, Log, WsEvent } from "../storage/types";
import { events } from "../storage/events";
import { PageHeader } from "../components/PageHeader";
import { Paper } from "../components/Paper";
import { formatTime } from "./helpers";

export function Logs(props: { workspace: Workspace }) {
  const [logs, setLogs] = useState<Log[]>([]);

  const onWsEvent = (e: WsEvent) => {
    const newLog = e.log;
    if (newLog) {
      setLogs((prevLogs) => [newLog, ...prevLogs]);
    }
  };

  useEffect(() => {
    events.on("event", onWsEvent);

    return () => {
      events.removeListener("event", onWsEvent);
    };
  }, [props.workspace]);

  const rows = logs.map((v) => (
    <Table.Tr key={v.timestamp}>
      <Table.Td>{formatTime(v.timestamp)}</Table.Td>
      <Table.Td>{v.level}</Table.Td>
      <Table.Td>{v.target}</Table.Td>
      <Table.Td>
        <Group>
          {Object.entries(v.attributes).map(([k, v]) => (
            <Badge>
              {k}: {v}
            </Badge>
          ))}
        </Group>
      </Table.Td>
      <Table.Td>{v.message}</Table.Td>
    </Table.Tr>
  ));

  return (
    <>
      <PageHeader
        title="Logs"
        breadcrumb={[
          { label: "Workspaces" },
          {
            label: props.workspace.name,
            link: `/workspaces/${props.workspace.name}`,
          },
        ]}
        extra={<Loader size="sm" />}
      />
      <Paper>
        <Table horizontalSpacing="md" verticalSpacing="md">
          <Table.Thead>
            <Table.Tr>
              <Table.Td>Time</Table.Td>
              <Table.Td>Level</Table.Td>
              <Table.Td>Target</Table.Td>
              <Table.Td>Attributes</Table.Td>
              <Table.Td>Message</Table.Td>
            </Table.Tr>
          </Table.Thead>
          <Table.Tbody>{rows}</Table.Tbody>
        </Table>
      </Paper>
    </>
  );
}
