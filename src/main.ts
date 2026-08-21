import "overlayscrollbars/overlayscrollbars.css";
import { mount } from "svelte";

import App from "./App.svelte";
import { disableNativeContextMenu } from "./lib/disableNativeContextMenu";

const app = mount(App, { target: document.body });

disableNativeContextMenu();

export default app;
