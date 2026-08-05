import React from "react";

import { Title, Breadcrumbs, Anchor, Text, Group } from "@mantine/core";
import { Link } from "react-router-dom";

import { useTitle } from "../views/helpers";

interface IProps {
  title: string;
  breadcrumb?: Breadcrumb[];
  extra?: React.ReactNode;
}

interface Breadcrumb {
  label: string;
  link?: string;
}

export function PageHeader(props: IProps) {
  if (props.breadcrumb) {
    useTitle(props.breadcrumb.map((v) => v.label));
  } else {
    useTitle([props.title]);
  }

  const items = props.breadcrumb
    ? props.breadcrumb.map((v) => {
        if (v.link) {
          return (
            <Anchor component={Link} to={v.link} key={v.label}>
              <Text size="sm">{v.label}</Text>
            </Anchor>
          );
        } else {
          return (
            <Text size="sm" key={v.label}>
              {v.label}
            </Text>
          );
        }
      })
    : [];

  return (
    <>
      <Group justify="space-between" align="center">
        <div>
          <Title order={3}>{props.title}</Title>
          {items && <Breadcrumbs separator="→">{items}</Breadcrumbs>}
        </div>
        <div>{props.extra}</div>
      </Group>
    </>
  );
}
