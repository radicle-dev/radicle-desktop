import { mount } from "svelte";
import { hotkeyKeyUX, hotkeyMacCompat, startKeyUX } from "keyux";
import "overlayscrollbars/overlayscrollbars.css";
import App from "./App.svelte";

const app = mount(App, { target: document.body });

const mac = hotkeyMacCompat();
startKeyUX(window, [hotkeyKeyUX([mac])]);

export default app;
