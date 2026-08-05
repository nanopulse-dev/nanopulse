import { useEffect, useState } from "react";

import { Loader, Table, Badge } from "@mantine/core";
import { JsonTree } from "@gfazioli/mantine-json-tree";

import type { Workspace, WsEvent } from "../storage/types";
import { events } from "../storage/events";
import { PageHeader } from "../components/PageHeader";
import { Paper } from "../components/Paper";
import { formatTime } from "./helpers";

export function Console(props: { workspace: Workspace }) {
  const [integrationEvents, setIntegrationEvents] = useState<WsEvent[]>([]);

  const onWsEvent = (e: WsEvent) => {
    if (
      e.key_exchange_event ||
      e.activation_event ||
      e.telemetry_event ||
      e.state_event
    ) {
      setIntegrationEvents((prevEvents) => [e, ...prevEvents]);
    }
  };

  useEffect(() => {
    events.on("event", onWsEvent);

    return () => {
      events.removeListener("event", onWsEvent);
    };
  }, [props.workspace]);

  const rows = integrationEvents.map((v) => {
    if (v.key_exchange_event) {
      return (
        <Table.Tr key={v.key_exchange_event.timestamp}>
          <Table.Td>{formatTime(v.key_exchange_event.timestamp)}</Table.Td>
          <Table.Td>
            <Badge>KeyExchange</Badge>
          </Table.Td>
          <Table.Td>
            <JsonTree
              size="sm"
              data={v.key_exchange_event}
              defaultExpanded
              maxDepth={5}
            />
          </Table.Td>
        </Table.Tr>
      );
    }

    if (v.activation_event) {
      return (
        <Table.Tr key={v.activation_event.timestamp}>
          <Table.Td>{formatTime(v.activation_event.timestamp)}</Table.Td>
          <Table.Td>
            <Badge>Activation</Badge>
          </Table.Td>
          <Table.Td>
            <JsonTree
              size="sm"
              data={v.activation_event}
              defaultExpanded
              maxDepth={5}
            />
          </Table.Td>
        </Table.Tr>
      );
    }

    if (v.telemetry_event) {
      return (
        <Table.Tr key={v.telemetry_event.timestamp}>
          <Table.Td>{formatTime(v.telemetry_event.timestamp)}</Table.Td>
          <Table.Td>
            <Badge>Telemetry</Badge>
          </Table.Td>
          <Table.Td>
            <JsonTree
              size="sm"
              data={v.telemetry_event}
              defaultExpanded
              maxDepth={5}
            />
          </Table.Td>
        </Table.Tr>
      );
    }

    if (v.state_event) {
      return (
        <Table.Tr key={v.state_event.timestamp}>
          <Table.Td>{formatTime(v.state_event.timestamp)}</Table.Td>
          <Table.Td>
            <Badge>State</Badge>
          </Table.Td>
          <Table.Td>
            <JsonTree
              size="sm"
              data={v.state_event}
              defaultExpanded
              maxDepth={5}
            />
          </Table.Td>
        </Table.Tr>
      );
    }
  });

  return (
    <>
      <PageHeader
        title="Console"
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
              <Table.Td width={100}>Time</Table.Td>
              <Table.Td width={150}>Event</Table.Td>
              <Table.Td>Payload</Table.Td>
            </Table.Tr>
          </Table.Thead>
          <Table.Tbody>{rows}</Table.Tbody>
        </Table>
      </Paper>
    </>
  );
}
