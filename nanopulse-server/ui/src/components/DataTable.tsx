import { useState, useEffect, useCallback, ReactElement } from "react";
import { Table, Pagination, Group, Loader } from "@mantine/core";
import type { TableProps } from "@mantine/core";

interface IProps<T> extends TableProps {
  header: ReactElement;
  getPage: (limit: number, offset: number) => Promise<Page<T>>;
  renderRow: (row: T) => ReactElement;
}

export interface Page<T> {
  total_count: number;
  result: T[];
}

export function DataTable<T>(props: IProps<T>) {
  const pageSize = 20;

  const [totalCount, setTotalCount] = useState<number>(0);
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [data, setData] = useState<T[]>([]);

  const loadPage = useCallback(
    async (page: number, pz: number) => {
      const p = await props.getPage(pz, (page - 1) * pz);
      setTotalCount(p.total_count);
      setData(p.result);
    },

    [props.getPage, pageSize],
  );

  useEffect(() => {
    loadPage(currentPage, pageSize);
  }, [currentPage, pageSize, loadPage]);

  if (!data) {
    return (
      <Group justify="flex-end" mt="md">
        <Loader color="blue" type="dots" />
      </Group>
    );
  }

  const { header, getPage, renderRow, ...otherProps } = props;

  const pages = Math.ceil(totalCount / pageSize);
  const tableRows = data.map((v) => props.renderRow(v));

  return (
    <>
      <Table horizontalSpacing="sm" verticalSpacing="sm" {...otherProps}>
        <Table.Thead>{props.header}</Table.Thead>
        <Table.Tbody>{tableRows}</Table.Tbody>
      </Table>
      <Group justify="flex-end" mt="md">
        <Pagination total={pages} onChange={setCurrentPage} />
      </Group>
    </>
  );
}
