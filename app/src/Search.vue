<script setup lang="ts">
// A basic Search modal accessible by pressing on `p`
import { invoke } from '@tauri-apps/api/core';
import { onKeyStroke } from '@vueuse/core'
import { ref, onMounted, Ref } from "vue";
import { ResearchResult } from './types';

interface Props {
    openSearchEntry: (e: ResearchResult) => Promise<boolean>
}
const props = defineProps<Props>()

// State of this search modal
const searchOpened = ref(false);
const searchInput = ref(null)
const searchResults: Ref<ResearchResult[]> = ref([])
const search = ref("")
const error = ref("")
const selectedEntry = ref(0)
const isFocused = ref(false)

async function runSearch() {
    try {
        const result = await invoke("run_search", { search: search.value }) as ResearchResult[];
        if (searchResults.value != result) {
            selectedEntry.value = 0 // reset selection entry on each results change
        }
        searchResults.value = result
        error.value = ""
    } catch (err) {
        error.value = err
    }
}

// If it is focused, we just want to loose focus, otherwise we want to close the search
function escapeFromSearchInput() {
    if (isFocused.value) {
        searchInput.value?.blur()
        isFocused.value = false
    } else {
        searchOpened.value = false
    }
}

// When pressing enter during search, we want to open the selected entry,
// by default this is the first one
async function onEnterDuringSearch() {
    if (!searchOpened.value) return;

    const entry = searchResults.value[selectedEntry.value]
    const new_path = entry.path
    if (new_path && new_path != "" && await props.openSearchEntry(entry)) {
        searchOpened.value = false
    }
}

// React to enter, escape or any other key that trigger running the search again
function onKeyDownOnSearchInput(e) {
    if (e.key == 'Escape') escapeFromSearchInput();
    else if (e.key == 'Enter') onEnterDuringSearch();
    else runSearch()
}

// Change the selected entry by incrementing the internal counter
function changeSelectedEntry(increment: number) {
    if (isFocused.value) return;
    // Check bounds defined by length of the results
    if (increment > 0 && selectedEntry.value < searchResults.value.length || selectedEntry.value > 0) {
        selectedEntry.value = selectedEntry.value + increment
    }
}

function insertSearchInputDuringSearch(e) {
    if (isFocused.value || !searchOpened.value) return;
    searchInput.value?.focus()
    e.preventDefault() // to avoid entering "i" in the input
}

function onEntryClick(result: ResearchResult) {
    if (props.openSearchEntry(result)) searchOpened.value = false
}

onMounted(() => {
    // Toggle this search modal on key p
    onKeyStroke(['p'], () => {
        searchOpened.value = true
        searchInput.value?.focus() // try to focus, even if this is not always successful
    })

    // Define up and down actions on selection
    onKeyStroke(['j', 'ArrowDown'], (e) => {
        if (searchOpened.value) {
            e.preventDefault()
            changeSelectedEntry(1)
        }
    })
    onKeyStroke(['k', 'ArrowUp'], (e) =>{
        if (searchOpened.value) {
            e.preventDefault()
            changeSelectedEntry(-1)
        }
    })

    // Declare escape and enter listener globally too to catch it when the output is not focused
    onKeyStroke(['Enter'], onEnterDuringSearch)
    onKeyStroke(['Escape'], escapeFromSearchInput)
    onKeyStroke(['i'], insertSearchInputDuringSearch)
})

// Only show the end of the paths when too long to be able to see the filenames
const MAX_PATH_SIZE = 60;
function cutLongPathAtLeft(text: string) {
    if (text.length > MAX_PATH_SIZE) {
        return "..." + text.substring(text.length - MAX_PATH_SIZE)
    }
}

</script>

<template>
    <div>
        <div v-if="searchOpened"
            class="fixed flex items-center justify-center inset-0 border bg-gray-200/40 overflow-hidden">
            <div
                class="p-5 max-w-[90vh] max-h-[60vh] w-full h-full border rounded-sm  bg-gray-100/80 overflow-hidden">
                <h2 class="text-3xl font-bold">Search</h2>
                <input @focusin="isFocused = true" v-model="search" ref="searchInput" tabindex="1"
                    @keydown.stop="onKeyDownOnSearchInput"
                    class="text-2xl font-bold focus:bg-orange-100 w-full rounded-sm p-3 border border-orange-300"
                    placeholder="Starting typing file names or heading keywords..." autofocus />

                <div v-if="error != ''" class="text-red-400">{{ error }}</div>

                <div v-for="(result, idx) in searchResults">
                    <div :class="idx == selectedEntry ? 'bg-orange-300/70' : 'cursor-pointer hover:bg-orange-300/60 bg-orange-300/20'"
                        :key="result.path" class="my-1 p-1 w-full overflow-hidden" @click="() => onEntryClick(result)">
                        <div class="flex justify-between mx-3">
                            <span v-if="result.title != null" class="font-bold text-xl">{{ result.title }}</span>
                            <span class="italic">{{ cutLongPathAtLeft(result.path) }}
                                <span class="ml-2 font-bold not-italic">{{ result.priority }}</span>
                            </span>
                        </div>
                    </div>
                </div>

                <div v-if="searchResults.length == 0 && search.length > 0" class="text-gray-400 my-5">
                    No results for this search
                </div>
            </div>
        </div>
    </div>
</template>
