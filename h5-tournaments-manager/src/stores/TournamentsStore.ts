import { create } from "zustand"
import { Tournament } from "../common/types"
import { invoke } from "@tauri-apps/api/core"

type TournamentsStoreData = {
    tournaments: Tournament[]
}

type TournamentsStoreActions = {
    load: () => void
}

export const useTournamentsStore = create<TournamentsStoreData & TournamentsStoreActions>((set) => ({
    tournaments: [],
    async load() {
        await invoke("load_tournaments")
            .then((tournaments_data) => set({tournaments: tournaments_data as Tournament[]}))
    },
}))