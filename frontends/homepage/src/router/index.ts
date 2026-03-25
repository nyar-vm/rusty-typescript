import { createRouter, createWebHistory, RouteRecordRaw } from "vue-router";

const routes: RouteRecordRaw[] = [
    {
        path: "/",
        name: "home",
        component: () => import("../views/Home.vue"),
    },
    {
        path: "/install",
        name: "install",
        component: () => import("../views/Install.vue"),
    },
    {
        path: "/playground",
        name: "playground",
        component: () => import("../views/Playground.vue"),
    },
];

const router = createRouter({
    history: createWebHistory(),
    routes,
});

export default router;
