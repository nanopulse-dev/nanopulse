import { ReactElement, useState, useCallback } from "react";
import { Link } from "react-router-dom";
import {
  Paper,
  Table,
  Anchor,
  Button,
  Checkbox,
  Group,
  Menu,
} from "@mantine/core";
import { IconExchange, IconTrash } from "@tabler/icons-react";
import { notifications } from "@mantine/notifications";
import { Led } from "@gfazioli/mantine-led";

import { client } from "../../storage/client";
import type { Workspace, Gateway } from "../../storage/types";
import { PageHeader } from "../../components/PageHeader";
import { DataTable } from "../../components/DataTable";
import type { Page } from "../../components/DataTable";

function Status(props: { dateTime: string | null }) {
  if (!props.dateTime) {
    return null;
  }

  const ts = Date.parse(props.dateTime);
  const now = Date.now();

  if (ts > now) {
    return <Led color="green" />;
  } else {
    return null;
  }
}

export function ListGateways(props: { workspace: Workspace }) {
  const [selectedRows, setSelectedRows] = useState<string[]>([]);
  const [refreshCounter, setRefreshCounter] = useState<number>(0);

  const getPage = useCallback(
    async (limit: number, offset: number): Promise<Page<Gateway>> => {
      const { data } = await client.GET(
        "/api/workspaces/{workspace_name}/gateways",
        {
          params: {
            path: {
              workspace_name: props.workspace.name,
            },
            query: { limit: limit, offset: offset },
          },
        },
      );

      if (data) {
        return data;
      }

      return { total_count: 0, result: [] };
    },
    [props.workspace, refreshCounter],
  );

  const allowKeyExchange = async () => {
    for (const name of selectedRows) {
      await client.POST(
        "/api/workspaces/{workspace_name}/gateways/{gateway_name}/allow-key-exchange",
        {
          params: {
            path: {
              workspace_name: props.workspace.name,
              gateway_name: name,
            },
          },
          body: {
            duration_sec: 300,
          },
        },
      );
    }

    notifications.show({
      title: "Allow Key Exchange",
      message:
        "Selected gateways will allow Key Exchange requests for 5 minutes",
      color: "blue",
    });
    setSelectedRows([]);
    setRefreshCounter(refreshCounter + 1);
  };

  const deleteGateways = async () => {
    if (
      window.confirm(
        "Are you sure you would like to delete the selected gateways?",
      )
    ) {
      for (const name of selectedRows) {
        await client.DELETE(
          "/api/workspaces/{workspace_name}/gateways/{gateway_name}",
          {
            params: {
              path: {
                workspace_name: props.workspace.name,
                gateway_name: name,
              },
            },
          },
        );

        notifications.show({
          title: "Gateways deleted",
          message: "The selected gateways have been deleted",
          color: "orange",
        });
        setSelectedRows([]);
        setRefreshCounter(refreshCounter + 1);
      }
    }
  };

  const header: ReactElement = (
    <Table.Tr>
      <Table.Td width={50}></Table.Td>
      <Table.Td>Name</Table.Td>
      <Table.Td>Description</Table.Td>
      <Table.Td width={120}>Key Exchange</Table.Td>
    </Table.Tr>
  );

  const renderRow = (row: Gateway): ReactElement => {
    return (
      <Table.Tr key={row.name}>
        <Table.Td>
          <Checkbox
            checked={selectedRows.includes(row.name)}
            onChange={(event) =>
              setSelectedRows(
                event.currentTarget.checked
                  ? [...selectedRows, row.name]
                  : selectedRows.filter((item) => item !== row.name),
              )
            }
          />
        </Table.Td>
        <Table.Td>
          <Anchor component={Link} to={`${row.name}`} size="sm">
            {row.name}
          </Anchor>
        </Table.Td>
        <Table.Td>{row.description}</Table.Td>
        <Table.Td>
          <Status dateTime={row.allow_key_exchange_until} />
        </Table.Td>
      </Table.Tr>
    );
  };

  return (
    <>
      <PageHeader
        title="Gateways"
        breadcrumb={[
          { label: "Workspaces" },
          {
            label: props.workspace.name,
            link: `/workspaces/${props.workspace.name}`,
          },
          { label: "Gateways" },
        ]}
        extra={
          <Link to="create">
            <Button>Add gateway</Button>
          </Link>
        }
      />
      <Paper mt="md" style={{ padding: "var(--mantine-spacing-md)" }}>
        <Group justify="flex-end">
          <Menu shadow="md" width={200}>
            <Menu.Target>
              <Button variant="default" disabled={selectedRows.length === 0}>
                {selectedRows.length} selected gateway(s)
              </Button>
            </Menu.Target>
            <Menu.Dropdown>
              <Menu.Item
                leftSection={<IconExchange size={14} />}
                onClick={allowKeyExchange}
              >
                Allow key-exchange
              </Menu.Item>
              <Menu.Item
                leftSection={<IconTrash size={14} />}
                onClick={deleteGateways}
              >
                Delete gateways
              </Menu.Item>
            </Menu.Dropdown>
          </Menu>
        </Group>
        <DataTable
          mt="md"
          header={header}
          getPage={getPage}
          renderRow={renderRow}
        />
      </Paper>
    </>
  );
}
