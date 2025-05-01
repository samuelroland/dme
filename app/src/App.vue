<script setup lang="ts">
import Home from "./Home.vue"
import { ref, onMounted } from "vue";
import type { Ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export type AppInfo = {
    version: string
}
const mdcontent = ref(null);
const appInfo: Ref<AppInfo> = ref({ version: "??" });

//type Some<T> = T
//type None = null
//type Option<T> = Some<T> | None

type Result<T> = {
    Ok?: T
    Err?: string
}

async function getMarkdown() {
    const result = await invoke("get_file_to_show") as Result<string>;
    if (result) {
        if (result.Err) {
            mdcontent.value = "<h2 class='text-red-300'>" + result.Err + "</h2>"
        } else {
            mdcontent.value = result.Ok
            console.log("inserted file", mdcontent.value)
        }
    }
    console.log("got a result", result)
    console.log("got a result", JSON.stringify(result))
}

async function getAppInfo() {
    const result = await invoke("get_app_info");
    console.log("got app_info", result)
    if (result != null) {
        appInfo.value = result as AppInfo
    }
}

onMounted(() => {
    getAppInfo()
    getMarkdown()
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
</template>
