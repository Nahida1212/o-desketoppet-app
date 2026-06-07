import { createApp } from "vue"
import { createPinia } from "pinia"
import { Quasar, Notify, Dialog } from "quasar"
import router from "@/router"
import App from "./App.vue"
import "./styles/main.css"

// Quasar CSS
import "quasar/src/css/index.sass"
// 图标库
import "@quasar/extras/material-icons/material-icons.css"

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(Quasar, {
  plugins: { Notify, Dialog },
  config: {
    dark: "auto", // 跟随系统暗色模式
  },
})

app.mount("#app")
