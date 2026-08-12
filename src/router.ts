import {createRouter, createWebHistory, RouteRecordRaw} from "vue-router";

import Main from "./layouts/pages/Main.vue";
import Setting from "./layouts/pages/Setting.vue";

const routes: Array<RouteRecordRaw> = [
    { path: "/", component: Main },
    { path: '/setting', component: Setting },
];

const router = createRouter({
    history: createWebHistory(),
    routes,
});

export default router;
