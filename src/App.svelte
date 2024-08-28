<script lang="ts">
  import { Router, Route } from "svelte-routing";
  import { theme } from "@app/components/ThemeSwitch.svelte";

  import DesignSystem from "@app/views/DesignSystem.svelte";
  import Repos from "@app/views/Repos.svelte";
  import Startup from "@app/views/Startup.svelte";

  $: document.documentElement.setAttribute("data-theme", $theme);
</script>

<Router>
  <Route path="/" component={Startup} />
  <Route path="/repos" component={Repos} />
  <Route path="/design-system" component={DesignSystem} />
</Router>
