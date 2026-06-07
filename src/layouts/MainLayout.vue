<script setup lang="ts">
import { ref, computed } from "vue"
import { useQuasar } from "quasar"
import { useRoute } from "vue-router"

const $q = useQuasar()
const route = useRoute()
const leftDrawerOpen = ref(false)

const navItems = [
  {
    label: "桌宠配置",
    icon: "pets",
    to: "/pet-config",
    separator: false,
  },
  {
    label: "角色模型配置",
    icon: "smart_toy",
    to: "/character-config",
    separator: false,
  },
  {
    label: "API 测试",
    icon: "science",
    to: "/api-test",
    separator: false,
  },
]

function toggleLeftDrawer() {
  leftDrawerOpen.value = !leftDrawerOpen.value
}

/** 判断是否为独立窗口（如宠物窗口），不显示布局框架 */
const isStandaloneWindow = computed(() => route.meta?.hideLayout === true)

/** 判断导航项是否激活 */
function isActive(item: (typeof navItems)[number]): boolean {
  return route.path.startsWith(item.to)
}
</script>

<style scoped>
.q-layout {
  height: 100vh;
}

/* q-page-container: border-box 模型，height: 100vh 使内容区 = 100vh - padding(50px) */
.q-page-container {
  height: 100vh;
}
</style>

<template>
  <!-- 独立窗口模式（如宠物窗口）：不显示布局框架 -->
  <router-view v-if="isStandaloneWindow" />

  <!-- 正常布局模式 -->
  <q-layout v-else view="hHh Lpr lFf">
    <!-- 顶部导航栏 -->
    <q-header elevated class="bg-primary text-white">
      <q-toolbar>
        <q-btn
          dense
          flat
          round
          icon="menu"
          aria-label="Menu"
          @click="toggleLeftDrawer"
        />

        <q-toolbar-title class="text-weight-medium">
          <q-icon name="pets" class="q-mr-sm" />
          Desktop Pet
        </q-toolbar-title>

        <!-- 暗色模式切换 -->
        <q-btn
          dense
          flat
          round
          :icon="$q.dark.isActive ? 'dark_mode' : 'light_mode'"
          @click="$q.dark.toggle()"
        >
          <q-tooltip>
            {{ $q.dark.isActive ? "切换亮色模式" : "切换暗色模式" }}
          </q-tooltip>
        </q-btn>
      </q-toolbar>
    </q-header>

    <!-- 侧边栏导航 -->
    <q-drawer
      v-model="leftDrawerOpen"
      show-if-above
      bordered
      :width="240"
      class="bg-grey-1"
    >
      <q-scroll-area class="fit">
        <q-list padding class="q-mt-sm">
          <template v-for="(item, index) in navItems" :key="index">
            <!-- 分隔线 -->
            <q-separator
              v-if="item.separator"
              spaced
              class="q-my-sm"
            />

            <q-item
              clickable
              v-ripple
              :to="item.to"
              :active="isActive(item)"
              active-class="bg-primary text-white"
              class="q-my-xs q-py-sm"
            >
              <q-item-section avatar>
                <q-icon :name="item.icon" />
              </q-item-section>
              <q-item-section>
                {{ item.label }}
              </q-item-section>
            </q-item>
          </template>
        </q-list>
      </q-scroll-area>
    </q-drawer>

    <!-- 主内容区域（q-page-container 处理头部偏移，q-scroll-area 处理滚动） -->
    <q-page-container>
      <q-scroll-area class="fit">
        <router-view />
      </q-scroll-area>
    </q-page-container>
  </q-layout>
</template>
