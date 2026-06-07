import { createRouter, createWebHistory } from "vue-router"

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      redirect: "/pet-config",
    },
    {
      path: "/pet-config",
      name: "pet-config",
      component: () => import("@/pages/pet/PetConfigPage.vue"),
      meta: { title: "桌宠配置" },
    },
    {
      path: "/character-config",
      name: "character-config",
      component: () => import("@/pages/character/CharacterConfigPage.vue"),
      meta: { title: "角色模型配置" },
    },
    {
      path: "/api-test",
      name: "api-test",
      component: () => import("@/pages/ApiTestPage.vue"),
      meta: { title: "API 测试" },
    },
    {
      path: "/character-edit/:id",
      name: "character-edit",
      component: () => import("@/pages/character/CharacterEditPage.vue"),
      meta: { title: "角色详细配置" },
    },
    {
      path: "/pet-window/:id",
      name: "pet-window",
      component: () => import("@/pages/pet/PetWindowPage.vue"),
      meta: { title: "", hideLayout: true },
    },
  ],
})

export default router
