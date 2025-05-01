<script setup lang="ts">
import type { AppInfo } from "./App.vue"

type MenuEntry = { action: string, icon: string, keymap: string }
const menu: MenuEntry[] = [
    {
        action: "Search in your Markdown files",
        icon: "search",
        keymap: "f"
    },
    {
        action: "Open history",
        icon: "history",
        keymap: "ctrl+h"
    },
    {
        action: "Export to PDF",
        icon: "pdf",
        keymap: "ctrl+e"
    },
    {
        action: "Open the help",
        icon: "help",
        keymap: "ctrl+h"
    }
]

const props = defineProps<{
    appInfo: AppInfo
}>();
</script>

<template>
    <div class="p-2 sm:mx-5 md:mx-10 lg:mx-10 max-w-[1300px] flex items-center flex-col h-[100vh] justify-center">
        <img class="w-96 max-w-[80vw]" src="/logo.svg" />
        <div>
            <span class="text-slate-700 my-3 pr-1">dme v{{ props.appInfo.version }} -</span>
            <a target="_blank" href="https://github.com/samuelroland/dme">Git repository</a>
        </div>
        <h1 class="text-3xl md:text-4xl my-5 nice-gradient">Delightful Markdown Experience</h1>
        <nav class="text-2xl md:text-3xl my-4 text-slate-700">
            <table>
                <tr v-for="entry in menu" class="h-12 p-2">
                    <td class="pr-7 xs:pr-10 sm:pr-20 md:pr-30"><span class="nice-gradient">{{ entry.action }}</span>
                    </td>
                    <td class="text-center "><span class="rounded-md px-2 bg-slate-100">{{ entry.keymap }}</span></td>
                </tr>
            </table>
        </nav>
    </div>
</template>
