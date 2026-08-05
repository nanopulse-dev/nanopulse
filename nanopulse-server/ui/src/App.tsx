import React from "react";
import { useState } from "react";
import type { ReactElement } from "react";
import { useDisclosure } from "@mantine/hooks";
import { MantineProvider, AppShell, Group, Burger, Title } from "@mantine/core";
import { Notifications } from "@mantine/notifications";
import type { RouterProps } from "react-router-dom";
import { Router, Route, Routes } from "react-router-dom";

import "@mantine/core/styles.css";
import "@mantine/notifications/styles.css";
import "@gfazioli/mantine-led/styles.css";
import "@gfazioli/mantine-json-tree/styles.css";

import appClasses from "./App.module.css";
import { theme } from "./theme";
import history from "./history";
import { SideMenu } from "./components/SideMenu";

import { WorkspaceLoader } from "./views/workspaces/WorkspaceLoader";
import { CreateWorkspace } from "./views/workspaces/CreateWorkspace";
import { RedirectToFirstWorkspace } from "./views/workspaces/RedirectToFirstWorkspace";

interface IProps extends Omit<
  RouterProps,
  "location" | "navigationType" | "navigator"
> {
  history: typeof history;
  children: ReactElement | undefined;
}

const CustomRouter = ({ history, ...props }: IProps) => {
  const [state, setState] = useState({
    action: history.action,
    location: history.location,
  });

  React.useLayoutEffect(() => history.listen(setState), [history]);

  return (
    <Router
      {...props}
      location={state.location}
      navigationType={state.action}
      navigator={history}
    />
  );
};

export default function App() {
  const [opened, { toggle }] = useDisclosure();

  return (
    <CustomRouter history={history}>
      <MantineProvider theme={theme}>
        <Notifications />
        <AppShell
          navbar={{
            width: 300,
            breakpoint: "sm",
            collapsed: { mobile: !opened },
          }}
          padding="md"
          header={{ height: 60 }}
          withBorder={false}
        >
          <AppShell.Header className={appClasses.header}>
            <div>
              <Group h="100%" px="md">
                <Burger
                  opened={opened}
                  onClick={toggle}
                  hiddenFrom="sm"
                  size="sm"
                />
                <Title order={3}>NanoPulse</Title>
              </Group>
            </div>
          </AppShell.Header>
          <AppShell.Navbar>
            <SideMenu
              onClick={() => {
                if (opened) {
                  toggle();
                }
              }}
            />
          </AppShell.Navbar>
          <AppShell.Main className={appClasses.main}>
            <Routes>
              <Route path="/" element={<RedirectToFirstWorkspace />} />
              <Route path="/workspaces/create" element={<CreateWorkspace />} />
              <Route
                path="/workspaces/:workspaceName/*"
                element={<WorkspaceLoader />}
              />
            </Routes>
          </AppShell.Main>
        </AppShell>
      </MantineProvider>
    </CustomRouter>
  );
}
