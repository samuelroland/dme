<script setup lang="ts">

type MenuEntry = { action: string, icon: string, keymap: string }
const menu: MenuEntry[] = [
    {
        action: "Search in your Markdown files",
        icon: "search",
        keymap: "f"
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
    appInfo: { type: { version: number }, required: true },
}>();
</script>

<template>
    <div class=" p-2 sm:m-5 md:m-10 lg:mx-10 lg:my-30 max-w-[1300px] flex items-center flex-col">
        <img class="w-96 max-w-[80vw]" src="/logo.svg" />
        <div>
            <span class="text-slate-700 my-3 pr-1">dme v{{ props.appInfo.version }} -</span>
            <a target="_blank" href="https://github.com/samuelroland/dme">Git repository</a>
        </div>
        <h1 class="text-3xl md:text-4xl my-5 nice-gradient">Delightful Markdown Experience</h1>
        <nav class="text-3xl my-4 text-slate-700">
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
