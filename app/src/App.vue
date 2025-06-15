<script setup lang="ts">
import Home from "./Home.vue"
import Grammars from "./Grammars.vue"
import Search from "./Search.vue"
import { onKeyStroke } from '@vueuse/core'
import { ref, onMounted } from "vue";
import type { Ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ResearchResult } from "./types";

// All used icons must be defined here
import { addIcons } from "oh-vue-icons";
import { BiTrashFill, CoSearch, IoHelp, CoPaint } from "oh-vue-icons/icons";
addIcons(BiTrashFill, CoSearch, IoHelp, CoPaint);

type Page = "Home" | "Grammars" | "Preview" | "Help"
const mdcontent: Ref<string | null> = ref(null)
const lastPathUsed: Ref<string | null> = ref(null)
const page: Ref<Page> = ref("Home")
const lastPage: Ref<Page> = ref("Home")

function switchToPage(newPage: Page) {
    lastPage.value = page.value
    page.value = newPage
}

function backToLastPage() {
    const nextPage = lastPage.value
    lastPage.value = page.value
    page.value = nextPage
}

// Open a Markdown file from disk, send null if unknown by the frontend yet
// It's possible to give a heading to select once opened (useful during search)
// Returns true if it has been opened, false in case of error
async function openMarkdown(path: string | null, selectedHeading: string | null) {
    lastPathUsed.value = path
    try {
        const result = await invoke("open_markdown_file", { path: path ?? "" }) as string;
        if (!result) return
        mdcontent.value = result
        switchToPage("Preview")

        // Try to scroll to the selected heading if present, after a small timeout
        // to let the content to render, no headings are found otherwise
        if (selectedHeading) {
            setTimeout(() => {
                const allHeadings = document.querySelectorAll('.prose h1, .prose h2, .prose h3, .prose h4, .prose h5, .prose h6') as unknown as HTMLElement[]
                for (const heading of allHeadings) {
                    if (heading.innerText == selectedHeading) {
                        heading.scrollIntoView({ behavior: 'smooth', block: 'start' });
                    }
                }
            }, 50)
        }
        return true
    } catch (err) {
        mdcontent.value = "<h2 class='text-red-300'>" + err + "</h2>"
        switchToPage("Preview")
        return false
    }
}

async function openSearchEntry(entry: ResearchResult) {
    return openMarkdown(entry.path, entry.title)
}


onMounted(() => {
    openMarkdown(lastPathUsed.value, null)
    onKeyStroke(['r'], () => { openMarkdown(lastPathUsed.value, null) })
    onKeyStroke(['Escape'], () => {
        if (page.value == "Grammars") {
            backToLastPage()
        }
        if (page.value == "Help") {
            backToLastPage()
        }
    })
    onKeyStroke((e) => { // Ctrl + G
        if (e.ctrlKey && e.key == 'g') {
            switchToPage("Grammars")
        }
    })
})
</script>

<template>
    <div class="flex justify-center">
        <div class="w-full h-full mx-2 sm:mx-5 lg:my-10 md:mx-[5vw] lg:mx-[10vw] max-w-[1000px]
prose prose-xl prose-zinc

prose-h1:!mt-2 prose-h2:!mt-3 prose-h3:!mt-3 prose-h4:!mt-4 prose-h5:!mt-3 prose-h6:!mt-3

prose-h1:!mb-2 prose-h2:!mb-1 prose-h3:!mb-1 prose-h4:!mb-1 prose-h5:!mb-1 prose-h6:!mb-1 prose-li:!m-0

prose-ol:!mt-0 prose-ul:!mt-0 prose-ol:!mb-2 prose-ul:!mb-2 prose-p:!mt-2 prose-p:!mb-1

prose-img:!mt-1 prose-img:!mb-0 prose-th:!border prose-td:!border prose-th:!p-2 prose-td:!p-2

prose-pre:!mt-1 prose-pre:!mb-2 prose-pre:!p-2 prose-pre:!px-4

prose-pre:whitespace-pre-wrap
selection:bg-blue-100 selection:text-black">
            <article v-if="page == 'Preview'" v-html="mdcontent" class="w-full"> </article>

            <div class="flex justify-center items-center h-[100vh]" v-if="page == 'Home'">
                <Home />
            </div>

            <div v-if="page == 'Grammars'">
                <Grammars :close-page="backToLastPage" />
            </div>

            <Search :openSearchEntry="openSearchEntry" />
        </div>
    </div>
</template>
