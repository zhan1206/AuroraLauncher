import { createRouter, createWebHistory, type RouteRecordRaw } from "vue-router";

const routes: RouteRecordRaw[] = [
  {
    path: "/",
    name: "Home",
    component: () => import("@/views/HomeView.vue"),
    meta: { title: "首页" },
  },
  {
    path: "/instances",
    name: "InstanceList",
    component: () => import("@/views/InstanceListView.vue"),
    meta: { title: "实例列表" },
  },
  {
    path: "/instances/:id",
    name: "InstanceDetail",
    component: () => import("@/views/InstanceDetailView.vue"),
    meta: { title: "实例详情" },
  },
  {
    path: "/versions",
    name: "VersionList",
    component: () => import("@/views/VersionListView.vue"),
    meta: { title: "版本管理" },
  },
  {
    path: "/settings",
    name: "Settings",
    component: () => import("@/views/SettingsView.vue"),
    meta: { title: "设置" },
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

// Update window title on navigation
router.afterEach((to) => {
  const title = (to.meta.title as string) || "Aurora Launcher";
  document.title = `${title} - Aurora Launcher`;
});

export default router;
