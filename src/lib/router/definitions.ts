interface BootingRoute {
  resource: "booting";
}

interface AuthenticationErrorRoute {
  resource: "authenticationError";
  params: {
    error: string;
    hint?: string;
  };
}

interface HomeRoute {
  resource: "home";
}

interface DesignSystemRoute {
  resource: "designSystem";
}

export type Route =
  | BootingRoute
  | DesignSystemRoute
  | HomeRoute
  | AuthenticationErrorRoute;

export type LoadedRoute =
  | BootingRoute
  | DesignSystemRoute
  | HomeRoute
  | AuthenticationErrorRoute;

export async function loadRoute(
  route: Route,
  _previousLoaded: LoadedRoute,
): Promise<LoadedRoute> {
  return route;
}
