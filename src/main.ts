import "overlayscrollbars/overlayscrollbars.css";
import { hotkeyKeyUX, hotkeyMacCompat, startKeyUX } from "keyux";
import { mount } from "svelte";

import App from "./App.svelte";
import { disableNativeContextMenu } from "./lib/disableNativeContextMenu";

const app = mount(App, { target: document.body });

disableNativeContextMenu();

const mac = hotkeyMacCompat();
startKeyUX(window, [hotkeyKeyUX([mac])]);

export default app;
