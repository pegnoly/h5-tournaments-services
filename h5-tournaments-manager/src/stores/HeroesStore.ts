import { create } from "zustand"
import { Hero } from "../common/types"
import { invoke } from "@tauri-apps/api/core"

type HeroesStoreData = {
    heroes: Hero[]
}

type HeroesStoreActions = {
    load: () => void
}

export const useHeroesStore = create<HeroesStoreData & HeroesStoreActions>((set) => ({
    heroes: [],
    async load() {
        await invoke("load_heroes")
            .then((heroes_data) => set({heroes: heroes_data as Hero[]}))
    },
}))