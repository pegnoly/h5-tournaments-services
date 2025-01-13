import { create } from "zustand"
import { Hero } from "../common/types"
import { invoke } from "@tauri-apps/api/core"

type HeroesStoreData = {
    heroes: Hero[]
}

type HeroesStoreActions = {
    load: (mod_type: number) => void
}

export const useHeroesStore = create<HeroesStoreData & HeroesStoreActions>((set) => ({
    heroes: [],
    async load(mod_type: number) {
        await invoke("load_heroes", {modType: mod_type})
            .then((heroes_data) => set({heroes: heroes_data as Hero[]}))
    },
}))