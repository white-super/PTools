import {createRouter, createWebHistory, RouteRecordRaw} from "vue-router";

import Main from "./layouts/pages/Main.vue";
import Setting from "./layouts/pages/Setting.vue";
import ShortcutHelp from "./layouts/pages/ShortcutHelp.vue";

const TextFormatter = () => import("./layouts/pages/TextFormatter.vue");
const TextDiff = () => import("./layouts/pages/TextDiff.vue");

const routes: Array<RouteRecordRaw> = [
    { path: "/", component: Main },
    { path: '/setting', component: Setting },
    { path: '/shortcut-help', component: ShortcutHelp },
    { path: '/formatter', component: TextFormatter },
    { path: '/diff', component: TextDiff },
];

const router = createRouter({
    history: createWebHistory(),
    routes,
});

export default router;
