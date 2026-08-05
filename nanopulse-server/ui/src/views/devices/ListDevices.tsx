import { ReactElement, useCallback } from "react";
import { useState } from "react";
import { Link } from "react-router-dom";
import { IconTrash } from "@tabler/icons-react";
import { Checkbox, Table, Anchor, Group, Menu, Button } from "@mantine/core";

import { client } from "../../storage/client";
import type { Workspace, Device } from "../../storage/types";
import { PageHeader } from "../../components/PageHeader";
import { DataTable } from "../../components/DataTable";
import type { Page } from "../../components/DataTable";
import { notifications } from "@mantine/notifications";
import { Paper } from "../../components/Paper";

export function ListDevices(props: { workspace: Workspace }) {
  const [selectedRows, setSelectedRows] = useState<string[]>([]);
  const [refreshCounter, setRefreshCounter] = useState<number>(0);

  const getPage = useCallback(
    async (limit: number, offset: number): Promise<Page<Device>> => {
      const { data } = await client.GET(
        "/api/workspaces/{workspace_name}/devices",
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

  const deleteDevices = async () => {
    if (
      window.confirm(
        "Are you sure you would like to delete the selected devices?",
      )
    ) {
      for (const name of selectedRows) {
        await client.DELETE(
          "/api/workspaces/{workspace_name}/devices/{device_name}",
          {
            params: {
              path: {
                workspace_name: props.workspace.name,
                device_name: name,
              },
            },
          },
        );

        notifications.show({
          title: "Devices deleted",
          message: "The selected devices have been deleted",
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
      <Table.Td>Short ID</Table.Td>
      <Table.Td>Description</Table.Td>
    </Table.Tr>
  );

  const renderRow = (row: Device): ReactElement => {
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
        <Table.Td>
          <pre>{row.short_id}</pre>
        </Table.Td>
        <Table.Td>{row.description}</Table.Td>
      </Table.Tr>
    );
  };

  return (
    <>
      <PageHeader
        title="Devices"
        breadcrumb={[
          { label: "Workspaces" },
          {
            label: props.workspace.name,
            link: `/workspaces/${props.workspace.name}`,
          },
          { label: "Devices" },
        ]}
      />
      <Paper>
        <Group justify="flex-end">
          <Menu shadow="md" width={200}>
            <Menu.Target>
              <Button variant="default" disabled={selectedRows.length === 0}>
                {selectedRows.length} selected devices(s)
              </Button>
            </Menu.Target>
            <Menu.Dropdown>
              <Menu.Item
                leftSection={<IconTrash size={14} />}
                onClick={deleteDevices}
              >
                Delete devices
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
