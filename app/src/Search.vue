<script setup lang="ts">
// A basic Search modal accessible by pressing on `p`
import { invoke } from '@tauri-apps/api/core';
import { onKeyStroke } from '@vueuse/core'
import { ref, onMounted, Ref } from "vue";

interface Props {
  openMarkdown: (path: string|null) => any
}
const props = defineProps<Props>()
const searchOpened = ref(false);

type ResearchResult = {
    path: string,
    title: string | null,
    priority: number
}

const searchInput = ref(null)
const results: Ref<ResearchResult[]> = ref([])
const search = ref("")
const error = ref("")
const selectedEntry = ref(0)
const isFocused = ref(false)

async function runSearch() {
    try {
        const result = await invoke("run_search", {search: search.value}) as ResearchResult[];
        results.value = result
        error.value = ""
    } catch (err) {
        error.value = err
    }
}

function escapeOnInput(){
    if (isFocused.value) {
        searchInput.value?.blur()
        isFocused.value = false
    } else {
        searchOpened.value = false
    }   
}
 async function onEnter() {
    const new_path = results.value[selectedEntry.value].path
    if (await props.openMarkdown(new_path)) {
        searchOpened.value = false
    }
}
function onKeyDown(e) {
    if (e.key == 'Escape') escapeOnInput();
    else if (e.key == 'Enter') onEnter();
    else runSearch()
}
onMounted(() => {
    onKeyStroke(['p'], () => {
        console.log("isFocused", isFocused.value)
        searchOpened.value = true
        searchInput.value?.focus() // try to focus, even if this is not always successful
    })
    onKeyStroke(['j'], () => {
        console.log("isFocused", isFocused.value)
        if (!isFocused.value) {
            if (selectedEntry.value < 20) {
                selectedEntry.value = selectedEntry.value + 1
            }
        }
    })
    onKeyStroke(['k'], () => {
        console.log("isFocused", isFocused.value)
        if (!isFocused.value) {
            if (selectedEntry.value > 0) {
                selectedEntry.value = selectedEntry.value - 1
            }
        }
    })
   
    onKeyStroke(['Enter'], onEnter)
    onKeyStroke(['Escape'], () => {
      escapeOnInput()
    })
})

const MAX_SIZE = 60;
function short(text: string) {
    if (text.length > MAX_SIZE) {
        return "..." + text.substring(text.length - MAX_SIZE)
    }
}

</script>

<template>
    <div>
    <div v-if="searchOpened"
        class="fixed flex items-center justify-center inset-0 border bg-gray-200/40 overflow-hidden">
        <div class="prose p-5 max-w-[90vh] max-h-[60vh] w-full h-full border rounded-sm  bg-gray-100/80 overflow-hidden">
            <h2>Search</h2>
            <input @focusin="isFocused = true"
                v-model="search" ref="searchInput"
                tabindex="1"
                @keydown.stop="onKeyDown"
                class="text-xl focus:bg-orange-300 w-full rounded-sm p-3 border border-orange-300"
                placeholder="Starting typing file names or heading keywords..." autofocus />
            {{ search }}
            
            <div v-if="error != ''" class="text-red-400">{{ error }}</div>
            
            <div v-for="(result, idx) in results">
                <div :class="idx == selectedEntry ?'bg-orange-300/70': 'cursor-pointer hover:bg-orange-300/60 bg-orange-300/20' " :key="result.path" class="my-1 p-1 w-full overflow-hidden"
                @click="() => {openMarkdown(result.path); searchOpened = false}">
                    <div class="flex justify-between mx-3">
                        <span v-if="result.title != null" class="font-bold">{{ result.title }}</span>
                        <span class="italic">{{ short(result.path) }} <span class="ml-2 font-bold not-italic">{{ result.priority }}</span></span>
                    </div>
                </div>
            </div>
        </div>
    </div>
    </div>
</template>
