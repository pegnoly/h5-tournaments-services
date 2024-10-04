import { create } from "zustand"
import { Race } from "../common/types"
import { invoke } from "@tauri-apps/api/core"

type RacesStoreType = {
    races: Race[]
}

type RacesStoreActions = {
    load: () => void
}

export const useRacesStore = create<RacesStoreType & RacesStoreActions>((set) => ({
    races: [],
    async load() {
        await invoke("load_races")
            .then((races_data) => set({races: races_data as Race[]}))
    },
}))