<script setup lang="ts">
import Home from "./Home.vue"
import Search from "./Search.vue"
import { onKeyStroke } from '@vueuse/core'
import { ref, onMounted } from "vue";
import type { Ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export type AppInfo = {
    version: string
}
const mdcontent = ref(null);
const appInfo: Ref<AppInfo> = ref({ version: "??" });
const lastPathUsed = ref(null)

async function openMarkdown(path: string | null) {
    lastPathUsed.value = path
    try {
        const result = await invoke("open_markdown_file", {path: path ?? ""}) as string;
        mdcontent.value = result
        return true
    } catch (err) {
        mdcontent.value = "<h2 class='text-red-300'>" + err + "</h2>"
        return false
    }
}

async function getAppInfo() {
    const result = await invoke("get_app_info");
    if (result != null) {
        appInfo.value = result as AppInfo
    }
}

onMounted(() => {
    getAppInfo()
    openMarkdown(lastPathUsed.value)
    onKeyStroke(['r'], () => {
        openMarkdown(lastPathUsed.value)
    }
    // Forced reload manually
 )
    // TODO: remove this hacky polling based watch mode
    setInterval(() => {let lastPath = lastPathUsed.value; openMarkdown(lastPath)}, 3000)
})
</script>

<template>
    <div class="flex justify-center" v-if="mdcontent == null">
        <Home :appInfo="appInfo" />
    </div>
    <article v-if="mdcontent != null" class="prose prose-base sm:prose-base md:prose-lg prose-zinc max-w-full
	prose-h1:!mt-2
	prose-h2:!mt-3
	prose-h3:!mt-3
	prose-h4:!mt-4
	prose-h5:!mt-3
	prose-h6:!mt-3
	
	prose-h1:!mb-2
	prose-h2:!mb-1
	prose-h3:!mb-1
	prose-h4:!mb-1
	prose-h5:!mb-1
	prose-h6:!mb-1
	prose-li:!m-0

	prose-ol:!mt-0
	prose-ul:!mt-0
	prose-ol:!mb-2
	prose-ul:!mb-2
	prose-p:!mt-2
	prose-p:!mb-1

	prose-img:!mt-1
	prose-img:!mb-0
	prose-th:!border
	prose-td:!border
	prose-th:!p-2
	prose-td:!p-2

	prose-pre:!mt-1
	prose-pre:!mb-2
	prose-pre:!p-2
	prose-pre:!px-4

	prose-pre:whitespace-pre-wrap
	selection:bg-blue-100 selection:text-black justify-center flex">

        <div v-html="mdcontent" class="m-auto p-2 sm:m-5 md:m-10 lg:my-10 lg:mx-40 max-w-[1300px]">
        </div>
    </article>
    <Search :openMarkdown="openMarkdown"/>
</template>
