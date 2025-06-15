<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { onMounted, Ref, ref } from 'vue';
import { OhVueIcon } from "oh-vue-icons";

enum InstalledStatus {
    NotInstalled = "NotInstalled",
    Installed = "Installed",
}

type GrammarState = {
    id: string,
    link: string,
    status: InstalledStatus,
}

interface Props {
    closePage: () => any
}
const props = defineProps<Props>()

const grammars: Ref<GrammarState[]> = ref([])
const grammarsFolder = ref("")
const lastError = ref("")

// A set of grammars currently in installation
// This is the easiest way to manage this state, on the frontend only
// It will not always be 100% correct (if user go back to home and come back this set will be lost)
const installingIds: Ref<Set<string>> = ref(new Set())

async function reloadGrammarsList() {
    grammars.value = await invoke("get_grammars_list") as unknown as GrammarState[]
    console.log(grammars.value)
}

async function installById(id: string) {
    lastError.value = ""
    try {
        installingIds.value.add(id)
        console.log("start install", installingIds.value)
        const promise = invoke("install_grammar", { id })
        await promise
        installingIds.value.delete(id)
        console.log("end install", installingIds.value)
        reloadGrammarsList()
    } catch (error) {
        lastError.value = error
        installingIds.value.delete(id)
    }
}

// The installation is very quick, we don't need to display "Uninstalling"
async function removeById(id: string) {
    await invoke("remove_grammar", { id })
    reloadGrammarsList()
}


onMounted(async () => {
    reloadGrammarsList()
    grammarsFolder.value = await invoke("grammars_folder")
    grammars.value = await invoke("get_grammars_list") as unknown as GrammarState[]
})

</script>

<template>
    <div class="flex">
        <h2 class="flex-1">Grammars installation</h2>
        <button class="btn" @click="props.closePage">Close</button>
    </div>
    <p>In this page, you can add support for syntax highlighting for the languages you use. Any programming languages or
        data format that has a Tree-Sitter is supported.</p>
    <p class="text-gray-500">Note: Tree-Sitter grammars are installed in folder: <em>{{ grammarsFolder }}</em></p>

    <div class="text-red-600">{{ lastError }}</div>
    <div>
        <table>
            <thead>
                <tr>
                    <th>ID</th>
                    <th>Link</th>
                    <th>Status</th>
                </tr>
            </thead>
            <tbody>
                <tr v-for="grammar in grammars" :key="grammar.id">
                    <td><code>{{ grammar.id }}</code></td>
                    <td><a :href="grammar.link">{{ grammar.link }}</a></td>
                    <td class="text-center">
                        <span v-if="grammar.status == InstalledStatus.NotInstalled">
                            <span v-if="installingIds.has(grammar.id)">Installing...</span>
                            <button v-else @click="() => installById(grammar.id)"
                                class="btn border-sea-light text-sea">Install</button>
                        </span>
                        <span v-if="grammar.status == InstalledStatus.Installed">
                            <span class="mr-3">Installed</span>
                            <button @click="() => removeById(grammar.id)" class="btn hover:bg-red-200">
                                <OhVueIcon name="bi-trash-fill" />
                            </button>
                        </span>

                    </td>
                </tr>
            </tbody>
        </table>
    </div>
</template>
