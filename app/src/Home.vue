<script setup lang="ts">
import { onMounted, ref, Ref } from "vue";
import type { AppInfo, MenuEntry } from "./types.ts"
import { invoke } from "@tauri-apps/api/core";
import { OhVueIcon } from "oh-vue-icons";

const menu: MenuEntry[] = [
    { action: "Search in your Markdown files", icon: "co-search", keymap: "s" },
    { action: "Grammars installation page", icon: "co-paint", keymap: "Ctrl+g" },
    { action: "Help page", icon: "io-help", keymap: "?" },
]

const appInfo: Ref<AppInfo> = ref({ version: "??" });
onMounted(async () => {
    const result = await invoke("get_app_info");
    if (result != null) {
        appInfo.value = result as AppInfo
    }
})
</script>

<template>
    <div class="p-2 sm:mx-5 md:mx-10 lg:mx-10 max-w-[1300px] flex items-center flex-col">
        <img class="w-96 max-w-[80vw]" src="/logo.svg" />
        <div>
            <span class="text-slate-700 my-3 pr-1">dme v{{ appInfo.version }} -</span>
            <a target="_blank" href="https://github.com/samuelroland/dme">Git repository</a>
        </div>
        <h1 class="text-3xl md:text-4xl my-5 nice-gradient">Delightful Markdown Experience</h1>
        <nav class="text-2xl md:text-3xl my-4 text-slate-700">
            <table class="not-prose">
                <tr v-for="entry in menu" class="h-12 p-2">
                    <td class="pr-7 xs:pr-10 sm:pr-20 md:pr-30">
                        <OhVueIcon width="30" height="30" :name="entry.icon" class="mr-3" />
                        <span class="nice-gradient"> {{ entry.action }} </span>
                    </td>
                    <td class="text-center ">
                        <span class="rounded-md px-2 text-black bg-slate-100">{{ entry.keymap }}</span>
                    </td>
                </tr>
            </table>
        </nav>
    </div>
</template>
