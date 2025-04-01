<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const mdcontent = ref("");
const name = ref("");

async function getMarkdown() {
    mdcontent.value = await invoke("load_markdown_as_html", { path: "/home/sam/HEIG/year3/HPC/HPC-labs/lab02/report/report.md" });
}
onMounted(() => { getMarkdown() })
</script>

<template>
    <article class="prose prose-base sm:prose-base md:prose-lg prose-zinc max-w-full
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

<style>
:root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;

    color: black;
    background-color: white;

    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    -webkit-text-size-adjust: 100%;
}

a {
    font-weight: 500;
    color: #646cff;
    text-decoration: inherit;
}

a:hover {
    color: #535bf2;
}
</style>
