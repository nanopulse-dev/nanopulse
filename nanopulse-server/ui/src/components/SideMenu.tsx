import { useState, useEffect, useCallback } from "react";
import { Link, useNavigate, useLocation } from "react-router-dom";
import {
  IconDashboard,
  IconApps,
  IconRouter,
  IconDevices,
  IconTerminal,
  IconLogs,
} from "@tabler/icons-react";
import { Group, Select } from "@mantine/core";

import sideMenuClasses from "./SideMenu.module.css";
import selectClasses from "./SideMenuSelect.module.css";
import type { Workspace } from "../storage/types";
import { client } from "../storage/client";
import { session } from "../storage/session";

export function SideMenu(props: { onClick: () => void }) {
  const location = useLocation();

  const [workspace, setWorkspace] = useState<Workspace | undefined>(undefined);
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [activeLink, setActiveLink] = useState("");
  const navigate = useNavigate();

  const setWorkspaceFn = () => {
    getWorkspaces();
    setWorkspace(session.getWorkspace());
  };

  const getWorkspaces = async () => {
    let workspaces: Workspace[] = [];
    let offset = 0;
    const limit = 100;

    while (true) {
      const { data } = await client.GET("/api/workspaces", {
        params: {
          query: { offset: offset, limit: limit },
        },
      });

      if (data) {
        workspaces.push(...data.result);
        if (offset + limit < data.total_count) {
          continue;
        }
      }

      break;
    }

    setWorkspaces(workspaces);
  };

  useEffect(() => {
    getWorkspaces();
  }, []);

  const parseLocation = useCallback(() => {
    if (/\/workspaces\/create/g.exec(location.pathname)) {
      setActiveLink("");
      return;
    }

    if (/\/workspaces\/[\w-]*/g.exec(location.pathname)) {
      setActiveLink("Dashboard");
    }

    if (/\/workspaces\/[\w-]*\/gateways.*/g.exec(location.pathname)) {
      setActiveLink("Gateways");
    }

    if (/\/workspaces\/[\w-]*\/devices.*/g.exec(location.pathname)) {
      setActiveLink("Devices");
    }

    if (/\/workspaces\/[\w-]*\/logs/g.exec(location.pathname)) {
      setActiveLink("Logs");
    }

    if (/\/workspaces\/[\w-]*\/console/g.exec(location.pathname)) {
      setActiveLink("Console");
    }
  }, [location.pathname]);

  useEffect(() => {
    session.on("workspace.set", setWorkspaceFn);
    parseLocation();

    return () => {
      session.removeListener("workspace.set", setWorkspaceFn);
    };
  }, [parseLocation]);

  const linkItems = workspace
    ? [
        {
          link: `/workspaces/${workspace.name}`,
          label: "Dashboard",
          icon: IconDashboard,
        },
        {
          link: `/workspaces/${workspace.name}/gateways`,
          label: "Gateways",
          icon: IconRouter,
        },
        {
          link: `/workspaces/${workspace.name}/devices`,
          label: "Devices",
          icon: IconDevices,
        },
        {
          link: `/workspaces/${workspace.name}/console`,
          label: "Console",
          icon: IconTerminal,
        },
        {
          link: `/workspaces/${workspace.name}/logs`,
          label: "Logs",
          icon: IconLogs,
        },
      ]
    : [];

  const links = linkItems.map((item) => (
    <Link
      onClick={props.onClick}
      key={item.label}
      to={item.link}
      className={sideMenuClasses.link}
      data-active={item.label === activeLink || undefined}
    >
      <item.icon className={sideMenuClasses.linkIcon} stroke={1.5} />
      <span>{item.label}</span>
    </Link>
  ));

  return (
    <nav className={sideMenuClasses.navbar}>
      <div className={sideMenuClasses.navbarMain}>
        <Group
          className={sideMenuClasses.workspaceSelect}
          justify="space-between"
        >
          <Select
            label="Select workspace"
            data={workspaces.map((v) => ({
              value: v.name,
              label: v.name,
            }))}
            value={workspace?.name}
            classNames={selectClasses}
            onChange={(value) => value && navigate(`/workspaces/${value}`)}
          />
        </Group>
        {workspace && links}
      </div>

      <div className={sideMenuClasses.footer}>
        <Link
          to="/workspaces/create"
          className={sideMenuClasses.link}
          onClick={props.onClick}
        >
          <IconApps className={sideMenuClasses.linkIcon} stroke={1.5} />
          <span>Add workspace</span>
        </Link>
      </div>
    </nav>
  );
}
