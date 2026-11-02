import { mount } from "svelte";
import "../src/css/tokens.css";
import "../src/css/base.css";
import Gallery from "./Gallery.svelte";

const target = document.getElementById("gallery");
if (target) mount(Gallery, { target });
