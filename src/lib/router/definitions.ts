import type { HomeLoadedRoute, HomeRoute } from "@app/views/home/router";
import type { NodesLoadedRoute, NodesRoute } from "@app/views/nodes/router";
import type {
  ProjectLoadedRoute,
  ProjectRoute,
} from "@app/views/projects/router";

import { loadHomeRoute } from "@app/views/home/router";
import { loadNodeRoute } from "@app/views/nodes/router";
import { loadProjectRoute } from "@app/views/projects/router";

interface BootingRoute {
  resource: "booting";
}

export interface NotFoundRoute {
  resource: "notFound";
  params: { title: string };
}

interface SessionRoute {
  resource: "session";
  params: { id: string; signature: string; publicKey: string };
}

export interface LoadErrorRoute {
  resource: "loadError";
  params: {
    title: string;
    errorMessage: string;
    stackTrace: string;
  };
}

export type Route =
  | BootingRoute
  | HomeRoute
  | LoadErrorRoute
  | NotFoundRoute
  | ProjectRoute
  | NodesRoute
  | SessionRoute;

export type LoadedRoute =
  | BootingRoute
  | HomeLoadedRoute
  | LoadErrorRoute
  | NotFoundRoute
  | ProjectLoadedRoute
  | NodesLoadedRoute
  | SessionRoute;

export async function loadRoute(route: Route): Promise<LoadedRoute> {
  if (route.resource === "nodes") {
    return await loadNodeRoute(route.params);
  } else if (route.resource === "home") {
    return await loadHomeRoute();
  } else if (
    route.resource === "project.source" ||
    route.resource === "project.history" ||
    route.resource === "project.commit" ||
    route.resource === "project.issues" ||
    route.resource === "project.newIssue" ||
    route.resource === "project.issue" ||
    route.resource === "project.patches" ||
    route.resource === "project.patch" ||
    route.resource === "project.board"
  ) {
    return await loadProjectRoute(route);
  } else {
    return route;
  }
}
