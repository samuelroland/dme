<script setup lang="ts">
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
    console.log("starting search of", search.value)
    try {
        const result = await invoke("run_search", {search: search.value}) as ResearchResult[];
        console.log("got a result", result)
        console.log("got a result", JSON.stringify(result))
        results.value = result
        error.value = ""
    } catch (err) {
        error.value = err
    }
}

function escapeOnInput(){
   console.log("isFocused", isFocused.value)
        if (isFocused.value) {
            searchInput.value?.blur()
            isFocused.value = false
        } else {
            searchOpened.value = false
        }   
}
 async function onEnter() {
    const new_path = results.value[selectedEntry.value].path
    console.log("opening new path ", new_path)
    if (await props.openMarkdown(new_path)) {
        searchOpened.value = false
    }
}
function onKeyDown(e) {
    if (e.key == 'Escape') escapeOnInput();
    if (e.key == 'Enter') onEnter();
}
onMounted(() => {
    onKeyStroke(['p'], () => {
        console.log("isFocused", isFocused.value)
        searchOpened.value = true
        searchInput.value?.focus()
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
        class="fixed flex items-center justify-center inset-0 border bg-gray-200/40">
        <div class="prose p-5 max-w-[90vh] max-h-[60vh] w-full h-full border rounded-sm  bg-gray-100/80">
            <h2>Search</h2>
            {{ selectedEntry }}
            <input @focusin="isFocused = true"
                @keypress="runSearch" v-model="search" ref="searchInput"
                @keydown.stop="onKeyDown" @keyup.stop=""
                class="focus:bg-orange-300 w-full rounded-sm p-3 border border-orange-300"
                placeholder="Starting typing file names or heading keywords..." autofocus />
            {{ search }}
            
            <div v-if="error != ''" class="text-red-400">{{ error }}</div>
            
            <div v-for="(result, idx) in results">
                <div :class="idx == selectedEntry ?'bg-orange-300/70': 'bg-orange-300/20' " :key="result.path" class="my-1 p-1  w-full overflow-hidden">
                    <span class="whitespace-nowrap [direction:rtl] text-ellipsis text-left">{{ short(result.path) }}</span>
                </div>
            </div>
        </div>
    </div>
    </div>
</template>
