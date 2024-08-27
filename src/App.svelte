<script lang="ts">
  import { Router, Route } from "svelte-routing";

  import Startup from "@app/views/Startup.svelte";
  import Repos from "@app/views/Repos.svelte";
</script>

<Router>
  <Route path="/" component={Startup} />
  <Route path="/repos" component={Repos} />
</Router>
