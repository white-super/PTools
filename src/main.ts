import { createApp } from "vue";
import "element-plus/es/components/message/style/css";
import "element-plus/es/components/message-box/style/css";
import App from "./App.vue";
import "./styles/app-themes.css";
import router from "./router.ts"

createApp(App).use(router).mount("#app");
